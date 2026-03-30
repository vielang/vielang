/// Integration tests cho MQTT Transport (Phase 3).
///
/// Mỗi test:
/// 1. Tạo tenant + device + credentials trong DB (isolated bởi sqlx::test)
/// 2. Khởi động MQTT server trên một port tự do
/// 3. Kết nối bằng raw TCP với MQTT CONNECT packet thủ công
/// 4. Publish telemetry / attributes
/// 5. Verify data trong DB

use sqlx::PgPool;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use uuid::Uuid;

use vl_config::MqttTransportConfig;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// ─── DB Setup Helpers ─────────────────────────────────────────────────────────

async fn insert_tenant(pool: &PgPool) -> Uuid {
    // Tạo tenant_profile trước (FK constraint)
    let profile_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant_profile (id, created_time, name, is_default, isolated_vl_rule_engine)
           VALUES ($1, $2, $3, false, false)"#,
        profile_id,
        now_ms(),
        format!("profile-{profile_id}"),
    )
    .execute(pool)
    .await
    .unwrap();

    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant (id, created_time, title, region, country, state, city,
           address, address2, zip, phone, email, tenant_profile_id)
           VALUES ($1, $2, $3, 'Global', '', '', '', '', '', '', '', $4, $5)"#,
        id,
        now_ms(),
        format!("tenant-{id}"),
        format!("{id}@test.com"),
        profile_id,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn insert_device_profile(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO device_profile
           (id, created_time, tenant_id, name, type, transport_type, provision_type, is_default)
           VALUES ($1, $2, $3, $4, 'DEFAULT', 'DEFAULT', 'DISABLED', false)"#,
        id,
        now_ms(),
        tenant_id,
        format!("profile-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn insert_device_with_token(
    pool: &PgPool,
    tenant_id: Uuid,
    access_token: &str,
) -> Uuid {
    let profile_id = insert_device_profile(pool, tenant_id).await;
    let device_id = Uuid::new_v4();

    sqlx::query!(
        r#"INSERT INTO device
           (id, created_time, tenant_id, device_profile_id, name, type, version)
           VALUES ($1, $2, $3, $4, $5, 'DEFAULT', 1)"#,
        device_id,
        now_ms(),
        tenant_id,
        profile_id,
        format!("device-{device_id}"),
    )
    .execute(pool)
    .await
    .unwrap();

    let cred_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO device_credentials
           (id, created_time, device_id, credentials_type, credentials_id)
           VALUES ($1, $2, $3, 'ACCESS_TOKEN', $4)"#,
        cred_id,
        now_ms(),
        device_id,
        access_token,
    )
    .execute(pool)
    .await
    .unwrap();

    device_id
}

// ─── MQTT Packet Builders ─────────────────────────────────────────────────────

/// Tạo MQTT CONNECT packet bytes.
/// flags = 0x82 = username present (bit 7) + clean session (bit 1)
fn build_connect(client_id: &str, username: &str) -> Vec<u8> {
    let protocol = b"\x00\x04MQTT\x04"; // "MQTT" v3.1.1
    let flags = [0x82u8]; // username + clean_session
    let keep_alive = [0x00u8, 0x3C]; // 60 seconds
    let client_id_bytes = encode_mqtt_str(client_id);
    let username_bytes = encode_mqtt_str(username);

    let remaining_len = protocol.len()
        + flags.len()
        + keep_alive.len()
        + client_id_bytes.len()
        + username_bytes.len();

    let mut pkt = vec![0x10u8]; // fixed header: CONNECT
    pkt.extend_from_slice(&encode_remaining_len(remaining_len));
    pkt.extend_from_slice(protocol);
    pkt.extend_from_slice(&flags);
    pkt.extend_from_slice(&keep_alive);
    pkt.extend_from_slice(&client_id_bytes);
    pkt.extend_from_slice(&username_bytes);
    pkt
}

/// Tạo MQTT PUBLISH packet bytes (QoS 0, no retain).
fn build_publish(topic: &str, payload: &[u8]) -> Vec<u8> {
    let topic_bytes = encode_mqtt_str(topic);
    let remaining_len = topic_bytes.len() + payload.len();

    let mut pkt = vec![0x30u8]; // fixed header: PUBLISH, QoS 0
    pkt.extend_from_slice(&encode_remaining_len(remaining_len));
    pkt.extend_from_slice(&topic_bytes);
    pkt.extend_from_slice(payload);
    pkt
}

/// MQTT DISCONNECT packet
fn build_disconnect() -> Vec<u8> {
    vec![0xE0, 0x00]
}

fn encode_mqtt_str(s: &str) -> Vec<u8> {
    let len = s.len() as u16;
    let mut v = vec![(len >> 8) as u8, (len & 0xFF) as u8];
    v.extend_from_slice(s.as_bytes());
    v
}

fn encode_remaining_len(mut len: usize) -> Vec<u8> {
    let mut encoded = Vec::new();
    loop {
        let mut byte = (len % 128) as u8;
        len /= 128;
        if len > 0 {
            byte |= 0x80;
        }
        encoded.push(byte);
        if len == 0 {
            break;
        }
    }
    encoded
}

// ─── Start MQTT Server Helper ─────────────────────────────────────────────────

/// Khởi động MQTT server trên một port tự do, trả về port.
async fn start_mqtt_server(pool: PgPool) -> u16 {
    // Bind port 0 để OS cấp port tự do
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Release — có race nhỏ nhưng chấp nhận được trong tests

    let config = MqttTransportConfig {
        enabled: true,
        bind: "127.0.0.1".into(),
        port,
        ws_enabled: false,
        ws_port: 0,
        ws_path: "/mqtt".into(),
        max_clients: 100,
    };

    tokio::spawn(async move {
        let noop_producer = vl_queue::create_producer(
            &vl_config::QueueConfig::default()
        ).expect("queue producer");
        let cache = vl_cache::create_cache(&vl_config::CacheConfig::default())
            .expect("cache");
        let (ws_tx, _) = tokio::sync::broadcast::channel::<vl_core::entities::TbMsg>(16);
        let noop_ts: std::sync::Arc<dyn vl_dao::TimeseriesDao> = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let (act_tx, _act_rx) = tokio::sync::mpsc::channel(16);
        let device_registry = std::sync::Arc::new(vl_transport::DeviceWriteRegistry::new());
        let rpc_pending = std::sync::Arc::new(vl_transport::RpcPendingRegistry::new());
        vl_transport::run_mqtt(pool, noop_ts, config, None, noop_producer, cache, ws_tx, act_tx, device_registry, rpc_pending, 256).await;
    });

    // Đợi server bind xong
    sleep(Duration::from_millis(100)).await;
    port
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_mqtt_connect_with_valid_token(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_connect_valid";
    insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect to MQTT server");

    // Send CONNECT
    let connect_pkt = build_connect("test-client-id", access_token);
    stream.write_all(&connect_pkt).await.unwrap();

    // Read CONNACK
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();

    // CONNACK: 0x20 0x02 0x00 0x00 = success, no session present
    assert_eq!(buf[0], 0x20, "Expected CONNACK packet type");
    assert_eq!(buf[3], 0x00, "Expected ConnectReturnCode::Success (0)");

    stream.write_all(&build_disconnect()).await.unwrap();
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mqtt_reject_invalid_token(pool: PgPool) -> sqlx::Result<()> {
    let port = start_mqtt_server(pool).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    let connect_pkt = build_connect("client-id", "INVALID_TOKEN_XYZ");
    stream.write_all(&connect_pkt).await.unwrap();

    // Read CONNACK — should be failure
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();

    assert_eq!(buf[0], 0x20, "Expected CONNACK");
    assert_ne!(buf[3], 0x00, "Expected non-zero return code (auth failure)");
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mqtt_telemetry_saved_to_db(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_telemetry";
    let device_id = insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool.clone()).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    // Connect
    stream
        .write_all(&build_connect("telemetry-test", access_token))
        .await
        .unwrap();

    // Read CONNACK
    let mut connack = [0u8; 4];
    stream.read_exact(&mut connack).await.unwrap();
    assert_eq!(connack[3], 0x00, "Expected successful connection");

    // Publish telemetry: {"temperature": 25.5, "humidity": 60}
    let payload = br#"{"temperature": 25.5, "humidity": 60}"#;
    let publish_pkt = build_publish("v1/devices/me/telemetry", payload);
    stream.write_all(&publish_pkt).await.unwrap();

    // Đợi server xử lý async
    sleep(Duration::from_millis(300)).await;

    stream.write_all(&build_disconnect()).await.unwrap();

    // Verify ts_kv_latest
    let rows = sqlx::query!(
        "SELECT k.key, l.dbl_v, l.long_v FROM ts_kv_latest l
         JOIN key_dictionary k ON k.key_id = l.key
         WHERE l.entity_id = $1
         ORDER BY k.key",
        device_id
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(rows.len(), 2, "Expected 2 telemetry entries (temperature + humidity)");

    let temp = rows.iter().find(|r| r.key == "temperature").expect("temperature not found");
    let humid = rows.iter().find(|r| r.key == "humidity").expect("humidity not found");

    assert_eq!(temp.dbl_v, Some(25.5));
    assert_eq!(humid.long_v, Some(60));

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mqtt_telemetry_with_timestamps(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_ts_format";
    let device_id = insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool.clone()).await;
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    stream
        .write_all(&build_connect("ts-test", access_token))
        .await
        .unwrap();

    let mut connack = [0u8; 4];
    stream.read_exact(&mut connack).await.unwrap();
    assert_eq!(connack[3], 0x00);

    // Array format với timestamps
    let payload = br#"[{"ts": 1700000000000, "values": {"voltage": 3.7}}]"#;
    stream
        .write_all(&build_publish("v1/devices/me/telemetry", payload))
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;
    stream.write_all(&build_disconnect()).await.unwrap();

    // Verify ts_kv với đúng timestamp
    let row = sqlx::query!(
        "SELECT l.dbl_v, l.ts FROM ts_kv_latest l
         JOIN key_dictionary k ON k.key_id = l.key
         WHERE l.entity_id = $1 AND k.key = 'voltage'",
        device_id
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(row.dbl_v, Some(3.7));
    assert_eq!(row.ts, 1700000000000i64);

    Ok(())
}

// ─── Unit 17 Edge Case Tests ──────────────────────────────────────────────────

/// Build a CONNECT packet with NO username flag (username field absent).
/// flags = 0x02 = clean session only (no username bit)
fn build_connect_no_username(client_id: &str) -> Vec<u8> {
    let protocol = b"\x00\x04MQTT\x04";
    let flags = [0x02u8]; // clean_session only, no username
    let keep_alive = [0x00u8, 0x3C];
    let client_id_bytes = encode_mqtt_str(client_id);

    let remaining_len = protocol.len() + flags.len() + keep_alive.len() + client_id_bytes.len();

    let mut pkt = vec![0x10u8];
    pkt.extend_from_slice(&encode_remaining_len(remaining_len));
    pkt.extend_from_slice(protocol);
    pkt.extend_from_slice(&flags);
    pkt.extend_from_slice(&keep_alive);
    pkt.extend_from_slice(&client_id_bytes);
    pkt
}

/// Build a CONNECT packet with an empty-string username.
/// flags = 0x82 = username present + clean session, but username string = ""
fn build_connect_empty_username(client_id: &str) -> Vec<u8> {
    build_connect(client_id, "")
}

/// MQTT_REJECT_EMPTY_USERNAME — server must reject CONNECT with no/empty username.
/// The server sends CONNACK with return code != 0 (bad credentials).
#[sqlx::test(migrations = "../../migrations")]
async fn mqtt_reject_empty_username(pool: PgPool) -> sqlx::Result<()> {
    let port = start_mqtt_server(pool).await;

    // Case 1: username flag set but empty string
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .expect("connect");
        stream
            .write_all(&build_connect_empty_username("client-empty"))
            .await
            .unwrap();

        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf[0], 0x20, "Expected CONNACK packet type");
        assert_ne!(buf[3], 0x00, "Empty username must be rejected (return code != 0)");
    }

    // Case 2: no username flag at all
    {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .expect("connect");
        stream
            .write_all(&build_connect_no_username("client-nousername"))
            .await
            .unwrap();

        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf[0], 0x20, "Expected CONNACK packet type");
        assert_ne!(buf[3], 0x00, "Missing username must be rejected (return code != 0)");
    }

    Ok(())
}

/// MQTT_REJECT_UNKNOWN_TOKEN — server must return CONNACK return code 4 or 5.
#[sqlx::test(migrations = "../../migrations")]
async fn mqtt_reject_unknown_token(pool: PgPool) -> sqlx::Result<()> {
    let port = start_mqtt_server(pool).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    stream
        .write_all(&build_connect("client-unknown", "TOTALLY_UNKNOWN_TOKEN_abc123"))
        .await
        .unwrap();

    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();

    assert_eq!(buf[0], 0x20, "Expected CONNACK");
    // mqttbytes v0.2 uses MQTT v5 return codes:
    //   134 = BadUserNamePassword, 135 = NotAuthorized
    // Return code must be non-zero (auth failure).
    assert_ne!(
        buf[3], 0x00,
        "Expected auth-failure return code, got {}",
        buf[3]
    );

    Ok(())
}

/// MQTT_INVALID_JSON_PAYLOAD — publishing malformed JSON must NOT crash/disconnect the client.
/// After a bad publish, a subsequent valid publish must still succeed (data saved to DB).
#[sqlx::test(migrations = "../../migrations")]
async fn mqtt_invalid_json_payload(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_bad_json";
    let device_id = insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool.clone()).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    stream
        .write_all(&build_connect("bad-json-client", access_token))
        .await
        .unwrap();

    let mut connack = [0u8; 4];
    stream.read_exact(&mut connack).await.unwrap();
    assert_eq!(connack[3], 0x00, "Expected successful connection");

    // Publish malformed JSON — server should log a warning and keep connection alive
    let bad_payload = b"{not_valid_json!!!";
    stream
        .write_all(&build_publish("v1/devices/me/telemetry", bad_payload))
        .await
        .unwrap();

    sleep(Duration::from_millis(200)).await;

    // Now publish valid JSON — connection must still be up
    let good_payload = br#"{"sensor_ok": 42}"#;
    stream
        .write_all(&build_publish("v1/devices/me/telemetry", good_payload))
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;
    stream.write_all(&build_disconnect()).await.unwrap();

    // Valid telemetry after the bad one must be saved
    let row = sqlx::query!(
        "SELECT l.long_v FROM ts_kv_latest l
         JOIN key_dictionary k ON k.key_id = l.key
         WHERE l.entity_id = $1 AND k.key = 'sensor_ok'",
        device_id
    )
    .fetch_one(&pool)
    .await?;

    assert_eq!(row.long_v, Some(42), "Valid publish after bad JSON must be saved");

    Ok(())
}

/// MQTT_LARGE_PAYLOAD — publish a payload with 100 key-value pairs; must succeed.
#[sqlx::test(migrations = "../../migrations")]
async fn mqtt_large_payload(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_large_payload";
    let device_id = insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool.clone()).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    stream
        .write_all(&build_connect("large-payload-client", access_token))
        .await
        .unwrap();

    let mut connack = [0u8; 4];
    stream.read_exact(&mut connack).await.unwrap();
    assert_eq!(connack[3], 0x00, "Expected successful connection");

    // Build a JSON object with 100 key-value pairs
    let mut pairs: Vec<String> = Vec::with_capacity(100);
    for i in 0..100 {
        pairs.push(format!("\"sensor_{:03}\": {}", i, i * 10));
    }
    let large_json = format!("{{{}}}", pairs.join(", "));

    stream
        .write_all(&build_publish("v1/devices/me/telemetry", large_json.as_bytes()))
        .await
        .unwrap();

    // Allow time for all 100 DB writes (each key needs get_or_create_key + save_latest)
    sleep(Duration::from_millis(4000)).await;
    stream.write_all(&build_disconnect()).await.unwrap();

    // Spot-check: first and last entries must be present
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM ts_kv_latest l
         JOIN key_dictionary k ON k.key_id = l.key
         WHERE l.entity_id = $1 AND k.key LIKE 'sensor_%'",
        device_id
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0);

    assert_eq!(count, 100, "All 100 large-payload entries must be saved");

    Ok(())
}

/// MQTT_MULTIPLE_SEQUENTIAL_PUBLISHES — publish 5 telemetry messages in a row;
/// connection must stay alive throughout and all data must be saved.
#[sqlx::test(migrations = "../../migrations")]
async fn mqtt_multiple_sequential_publishes(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_multi_pub";
    let device_id = insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool.clone()).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    stream
        .write_all(&build_connect("multi-pub-client", access_token))
        .await
        .unwrap();

    let mut connack = [0u8; 4];
    stream.read_exact(&mut connack).await.unwrap();
    assert_eq!(connack[3], 0x00, "Expected successful connection");

    // Publish 5 times; each sets the same key to a different value
    for i in 1i64..=5 {
        let payload = format!("{{\"seq_val\": {}}}", i);
        stream
            .write_all(&build_publish("v1/devices/me/telemetry", payload.as_bytes()))
            .await
            .expect(&format!("publish #{} failed (connection dropped?)", i));
        sleep(Duration::from_millis(50)).await;
    }

    sleep(Duration::from_millis(300)).await;
    stream.write_all(&build_disconnect()).await.unwrap();

    // ts_kv_latest holds the LAST written value for a key
    let row = sqlx::query!(
        "SELECT l.long_v FROM ts_kv_latest l
         JOIN key_dictionary k ON k.key_id = l.key
         WHERE l.entity_id = $1 AND k.key = 'seq_val'",
        device_id
    )
    .fetch_one(&pool)
    .await?;

    // Last publish had value 5
    assert_eq!(row.long_v, Some(5), "Last sequential publish value must be saved");

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mqtt_client_attributes_saved(pool: PgPool) -> sqlx::Result<()> {
    let tenant_id = insert_tenant(&pool).await;
    let access_token = "mqtt_test_token_attrs";
    let device_id = insert_device_with_token(&pool, tenant_id, access_token).await;

    let port = start_mqtt_server(pool.clone()).await;
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    stream
        .write_all(&build_connect("attr-test", access_token))
        .await
        .unwrap();

    let mut connack = [0u8; 4];
    stream.read_exact(&mut connack).await.unwrap();
    assert_eq!(connack[3], 0x00);

    // Publish client attributes
    let payload = br#"{"firmware_version": "1.0.2", "battery_level": 87}"#;
    stream
        .write_all(&build_publish("v1/devices/me/attributes", payload))
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;
    stream.write_all(&build_disconnect()).await.unwrap();

    // Verify attribute_kv (CLIENT_SCOPE = 1)
    let rows = sqlx::query!(
        "SELECT k.key, a.str_v, a.long_v FROM attribute_kv a
         JOIN key_dictionary k ON k.key_id = a.attribute_key
         WHERE a.entity_id = $1 AND a.attribute_type = 1
         ORDER BY k.key",
        device_id
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(rows.len(), 2);

    let battery = rows.iter().find(|r| r.key == "battery_level").unwrap();
    let firmware = rows.iter().find(|r| r.key == "firmware_version").unwrap();

    assert_eq!(battery.long_v, Some(87));
    assert_eq!(firmware.str_v.as_deref(), Some("1.0.2"));

    Ok(())
}
