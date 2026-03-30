/// Seed script: tạo dữ liệu test cho VieLang + ThingsBoard UI
/// Chạy: DATABASE_URL=... cargo run -p vl-seed
/// Replay: DATABASE_URL=... REPLAY_NODE_ID=<uuid> cargo run -p vl-seed -- replay
mod replay;
use sqlx::PgPool;
use uuid::Uuid;
use vl_core::entities::{Authority, User};
use vl_dao::postgres::user::UserDao;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// UUID cố định để seed reproducible
const TENANT_PROFILE_ID: &str = "00000000-0000-0000-0000-000000000001";
const TENANT_ID: &str         = "00000000-0000-0000-0000-000000000002";
const CUSTOMER_ID: &str       = "00000000-0000-0000-0000-000000000003";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://vielang:vielang@localhost:5432/vielang".into());

    let pool = PgPool::connect(&db_url).await?;

    // Subcommand: `cargo run -p vl-seed -- replay`
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("replay") {
        return replay::run(&pool).await;
    }

    println!("🌱 Seeding VieLang test data...");

    // ── 1. Tenant Profile ────────────────────────────────────────────────────
    // Look up existing "Default" profile or create one
    let tp_row = sqlx::query!(
        "SELECT id FROM tenant_profile WHERE name = 'Default' LIMIT 1"
    ).fetch_optional(&pool).await?;
    let tp_id = if let Some(row) = tp_row {
        println!("  ✅ Tenant Profile: Default (existing)");
        row.id
    } else {
        let tp_id = Uuid::parse_str(TENANT_PROFILE_ID)?;
        sqlx::query!(
            r#"INSERT INTO tenant_profile
               (id, created_time, name, is_default, isolated_vl_rule_engine, profile_data, version)
               VALUES ($1, $2, 'Default', true, false, $3, 1)"#,
            tp_id, now_ms(),
            serde_json::json!({"configuration": {"type": "DEFAULT"}})
        ).execute(&pool).await?;
        println!("  ✅ Tenant Profile: Default (created)");
        tp_id
    };

    // ── 2. Tenant ────────────────────────────────────────────────────────────
    let tenant_id = Uuid::parse_str(TENANT_ID)?;
    let result = sqlx::query!(
        r#"INSERT INTO tenant
           (id, created_time, title, tenant_profile_id, region, country, state, city, address,
            address2, zip, phone, email, additional_info, version)
           VALUES ($1,$2,$3,$4,'','Vietnam','','Hanoi','123 Test St',
                   '','100000','0912345678','admin@vielang.dev','{}',1)
           ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title"#,
        tenant_id, now_ms(), "VieLang Demo Tenant", tp_id
    ).execute(&pool).await;
    match result {
        Ok(_) => println!("  ✅ Tenant: VieLang Demo Tenant (id={tenant_id})"),
        Err(e) => println!("  ⚠️  Tenant: {e}"),
    }

    // ── 3. Customer ──────────────────────────────────────────────────────────
    let customer_id = Uuid::parse_str(CUSTOMER_ID)?;
    let result = sqlx::query!(
        r#"INSERT INTO customer
           (id, created_time, tenant_id, title, country, city, phone, email,
            is_public, additional_info, version)
           VALUES ($1,$2,$3,$4,'Vietnam','Hanoi','0987654321','customer@vielang.dev',
                   false,'{}',1)
           ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title"#,
        customer_id, now_ms(), tenant_id, "Demo Customer Co."
    ).execute(&pool).await;
    match result {
        Ok(_) => println!("  ✅ Customer: Demo Customer Co. (id={customer_id})"),
        Err(e) => println!("  ⚠️  Customer: {e}"),
    }

    let user_dao = UserDao::new(pool.clone());

    // Helper: upsert user + credentials by email
    async fn upsert_user_with_creds(
        pool: &PgPool,
        user_dao: &UserDao,
        email: &str,
        password_plain: &str,
        authority: Authority,
        tenant_id: Uuid,
        customer_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        // Find or create user
        let existing = sqlx::query!("SELECT id FROM tb_user WHERE email = $1", email)
            .fetch_optional(pool).await?;
        let user_id = if let Some(row) = existing {
            row.id
        } else {
            let u = User {
                id: Uuid::new_v4(), created_time: crate::now_ms(),
                tenant_id, customer_id,
                email: email.into(), authority,
                first_name: None, last_name: None,
                phone: None, additional_info: None, version: 1,
            };
            user_dao.save(&u).await.map_err(|e| anyhow::anyhow!("{e}"))?;
            u.id
        };

        // Upsert credentials
        let hash = vl_auth::password::hash_password(password_plain)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        sqlx::query!(
            r#"INSERT INTO user_credentials (id, created_time, user_id, enabled, password)
               VALUES ($1,$2,$3,true,$4)
               ON CONFLICT (user_id) DO UPDATE SET password = EXCLUDED.password, enabled = true"#,
            Uuid::new_v4(), crate::now_ms(), user_id, hash
        ).execute(pool).await?;
        Ok(())
    }

    // ── 4. SYS_ADMIN ─────────────────────────────────────────────────────────
    match upsert_user_with_creds(&pool, &user_dao, "sysadmin@thingsboard.org", "sysadmin",
        Authority::SysAdmin, Uuid::nil(), None).await {
        Ok(_) => println!("  ✅ SysAdmin: sysadmin@thingsboard.org / sysadmin"),
        Err(e) => println!("  ⚠️  SysAdmin: {e}"),
    }

    // ── 5. TENANT_ADMIN ───────────────────────────────────────────────────────
    match upsert_user_with_creds(&pool, &user_dao, "tenant@vielang.dev", "tenant123",
        Authority::TenantAdmin, tenant_id, None).await {
        Ok(_) => println!("  ✅ TenantAdmin: tenant@vielang.dev / tenant123"),
        Err(e) => println!("  ⚠️  TenantAdmin: {e}"),
    }

    // ── 6. CUSTOMER_USER ─────────────────────────────────────────────────────
    match upsert_user_with_creds(&pool, &user_dao, "customer@vielang.dev", "customer123",
        Authority::CustomerUser, tenant_id, Some(customer_id)).await {
        Ok(_) => println!("  ✅ CustomerUser: customer@vielang.dev / customer123"),
        Err(e) => println!("  ⚠️  CustomerUser: {e}"),
    }

    // ── 7. Device Profiles ────────────────────────────────────────────────────
    let profiles = vec![
        ("00000000-0000-0000-0000-000000000040", "Default", "DEFAULT"),
        ("00000000-0000-0000-0000-000000000041", "Temperature Sensor", "TEMPERATURE"),
        ("00000000-0000-0000-0000-000000000042", "Smart Meter", "METER"),
    ];
    for (id_str, name, ptype) in &profiles {
        let pid = Uuid::parse_str(id_str)?;
        let is_default = *name == "Default";
        let result = sqlx::query!(
            r#"INSERT INTO device_profile
               (id, created_time, tenant_id, name, type, transport_type,
                provision_type, profile_data, is_default, version)
               VALUES ($1,$2,$3,$4,$5,'DEFAULT','DISABLED',$6,$7,1)
               ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name"#,
            pid, now_ms(), tenant_id, name, ptype,
            serde_json::json!({
                "configuration": {"type": "DEFAULT"},
                "transportConfiguration": {"type": "DEFAULT"},
                "provisionConfiguration": {"type": "DISABLED"},
                "alarms": []
            }),
            is_default
        ).execute(&pool).await;
        match result {
            Ok(_) => println!("  ✅ DeviceProfile: {name}"),
            Err(e) => println!("  ⚠️  DeviceProfile {name}: {e}"),
        }
    }

    // ── 8. Devices ────────────────────────────────────────────────────────────
    let default_profile_id = Uuid::parse_str("00000000-0000-0000-0000-000000000040")?;
    let temp_profile_id = Uuid::parse_str("00000000-0000-0000-0000-000000000041")?;
    let meter_profile_id = Uuid::parse_str("00000000-0000-0000-0000-000000000042")?;

    // (id, name, type, profile_id, customer_id, is_gateway)
    let devices: Vec<(&str, &str, &str, Uuid, Option<Uuid>, bool)> = vec![
        ("00000000-0000-0000-0000-000000000050", "Gateway-01", "GATEWAY", default_profile_id, None, true),
        ("00000000-0000-0000-0000-000000000051", "TempSensor-01", "TEMPERATURE", temp_profile_id, Some(customer_id), false),
        ("00000000-0000-0000-0000-000000000052", "TempSensor-02", "TEMPERATURE", temp_profile_id, Some(customer_id), false),
        ("00000000-0000-0000-0000-000000000053", "SmartMeter-01", "METER", meter_profile_id, Some(customer_id), false),
        ("00000000-0000-0000-0000-000000000054", "SmartMeter-02", "METER", meter_profile_id, None, false),
        ("00000000-0000-0000-0000-000000000055", "AirQuality-01", "AIR", default_profile_id, None, false),
    ];

    for (id_str, name, dtype, profile_id, cust_id, is_gateway) in &devices {
        let dev_id = Uuid::parse_str(id_str)?;
        let token = format!("token-{}", &id_str[id_str.len()-2..]);
        let result = sqlx::query!(
            r#"INSERT INTO device
               (id, created_time, tenant_id, customer_id, name, type,
                device_profile_id, additional_info, version)
               VALUES ($1,$2,$3,$4,$5,$6,$7,'{}',1)
               ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name"#,
            dev_id, now_ms(), tenant_id, *cust_id, name, dtype, profile_id
        ).execute(&pool).await;
        // Set gateway flag in additional_info
        if *is_gateway {
            let _ = sqlx::query!(
                "UPDATE device SET additional_info = '{\"gateway\":true}'::jsonb WHERE id = $1",
                dev_id
            ).execute(&pool).await;
        }
        match result {
            Ok(_) => println!("  ✅ Device: {name}"),
            Err(e) => println!("  ⚠️  Device {name}: {e}"),
        }

        // Device credentials (ACCESS_TOKEN)
        let cred_result = sqlx::query!(
            r#"INSERT INTO device_credentials
               (id, created_time, device_id, credentials_type, credentials_id)
               VALUES ($1,$2,$3,'ACCESS_TOKEN',$4)
               ON CONFLICT (device_id) DO NOTHING"#,
            Uuid::new_v4(), now_ms(), dev_id, token
        ).execute(&pool).await;
        if let Err(e) = cred_result {
            println!("  ⚠️  DeviceCreds {name}: {e}");
        }
    }

    // ── 9. Assets ─────────────────────────────────────────────────────────────
    // Ensure a "Default" asset profile exists and is marked is_default=true
    let default_ap_id = Uuid::parse_str("00000000-0000-0000-0000-000000000090")?;
    let _ = sqlx::query!(
        r#"INSERT INTO asset_profile (id, created_time, tenant_id, name, is_default)
           VALUES ($1,$2,$3,'Default',true)
           ON CONFLICT (id) DO UPDATE SET is_default = true"#,
        default_ap_id, now_ms(), tenant_id
    ).execute(&pool).await;
    println!("  ✅ AssetProfile: Default (is_default=true)");

    let assets = vec![
        ("00000000-0000-0000-0000-000000000060", "Building A", "BUILDING"),
        ("00000000-0000-0000-0000-000000000061", "Floor 1",    "FLOOR"),
        ("00000000-0000-0000-0000-000000000062", "Room 101",   "ROOM"),
    ];
    for (id_str, name, atype) in &assets {
        let aid = Uuid::parse_str(id_str)?;
        let default_asset_profile = ensure_asset_profile(&pool, tenant_id, atype).await;
        let result = sqlx::query!(
            r#"INSERT INTO asset
               (id, created_time, tenant_id, customer_id, name, type,
                asset_profile_id, additional_info, version)
               VALUES ($1,$2,$3,NULL,$4,$5,$6,'{}',1)
               ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name"#,
            aid, now_ms(), tenant_id, name, atype, default_asset_profile
        ).execute(&pool).await;
        match result {
            Ok(_) => println!("  ✅ Asset: {name} ({atype})"),
            Err(e) => println!("  ⚠️  Asset {name}: {e}"),
        }
    }

    // ── 10. Dashboards ────────────────────────────────────────────────────────
    let dashboards = vec![
        ("00000000-0000-0000-0000-000000000070", "Main Dashboard"),
        ("00000000-0000-0000-0000-000000000071", "Temperature Overview"),
        ("00000000-0000-0000-0000-000000000072", "Energy Monitoring"),
    ];
    for (id_str, title) in &dashboards {
        let did = Uuid::parse_str(id_str)?;
        let result = sqlx::query!(
            r#"INSERT INTO dashboard
               (id, created_time, tenant_id, title, configuration, mobile_hide,
                mobile_order, assigned_customers, version)
               VALUES ($1,$2,$3,$4,'{}',false,0,'[]',1)
               ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title"#,
            did, now_ms(), tenant_id, title
        ).execute(&pool).await;
        match result {
            Ok(_) => println!("  ✅ Dashboard: {title}"),
            Err(e) => println!("  ⚠️  Dashboard {title}: {e}"),
        }
    }

    // ── 11. Alarms ────────────────────────────────────────────────────────────
    let alarms_data = vec![
        ("00000000-0000-0000-0000-000000000080", "HighTemperature", "CRITICAL",
         "00000000-0000-0000-0000-000000000051", "DEVICE"),
        ("00000000-0000-0000-0000-000000000081", "OverVoltage", "MAJOR",
         "00000000-0000-0000-0000-000000000053", "DEVICE"),
        ("00000000-0000-0000-0000-000000000082", "LowBattery", "MINOR",
         "00000000-0000-0000-0000-000000000052", "DEVICE"),
    ];
    let ts_now = now_ms();
    // originator_type: 0=DEVICE,1=ASSET,2=TENANT,3=CUSTOMER,4=USER,5=DASHBOARD,6=RULE_CHAIN,7=RULE_NODE,8=WIDGET_TYPE,9=WIDGET_BUNDLE
    for (id_str, alarm_type, severity, orig_id, _) in &alarms_data {
        let aid = Uuid::parse_str(id_str)?;
        let oid = Uuid::parse_str(orig_id)?;
        let result = sqlx::query!(
            r#"INSERT INTO alarm
               (id, created_time, tenant_id, type, originator_id, originator_type,
                severity, acknowledged, cleared, start_ts, end_ts, assign_ts, details)
               VALUES ($1,$2,$3,$4,$5,0,$6,false,false,$7,0,0,'{}')
               ON CONFLICT (id) DO UPDATE SET type = EXCLUDED.type"#,
            aid, ts_now, tenant_id, alarm_type, oid,
            severity, ts_now
        ).execute(&pool).await;
        match result {
            Ok(_) => println!("  ✅ Alarm: {alarm_type} ({severity})"),
            Err(e) => println!("  ⚠️  Alarm {alarm_type}: {e}"),
        }
    }

    // ── 12. Telemetry (ts_kv_latest) ─────────────────────────────────────────
    // (entity_id, key, bool_v, long_v, dbl_v, str_v)
    let telemetry: Vec<(&str, &str, Option<bool>, Option<i64>, Option<f64>, Option<&str>)> = vec![
        ("00000000-0000-0000-0000-000000000051", "temperature", None, None, Some(24.5_f64), None),
        ("00000000-0000-0000-0000-000000000051", "humidity",    None, None, Some(65.0_f64), None),
        ("00000000-0000-0000-0000-000000000052", "temperature", None, None, Some(31.2_f64), None),
        ("00000000-0000-0000-0000-000000000053", "voltage",     None, Some(220_i64), None,  None),
        ("00000000-0000-0000-0000-000000000053", "power_kw",    None, None, Some(5.8_f64), None),
        ("00000000-0000-0000-0000-000000000054", "voltage",     None, Some(221_i64), None,  None),
        ("00000000-0000-0000-0000-000000000055", "pm25",        None, Some(42_i64),  None,  None),
        ("00000000-0000-0000-0000-000000000055", "co2",         None, Some(412_i64), None,  None),
    ];

    for (entity_id_str, key, bool_v, long_v, dbl_v, str_v) in &telemetry {
        let entity_id = Uuid::parse_str(entity_id_str)?;

        let _ = sqlx::query!(
            "INSERT INTO key_dictionary (key) VALUES ($1) ON CONFLICT (key) DO NOTHING", key
        ).execute(&pool).await;
        let key_row = sqlx::query!(
            "SELECT key_id FROM key_dictionary WHERE key = $1", key
        ).fetch_one(&pool).await;

        match key_row {
            Ok(r) => {
                let key_id = r.key_id;
                let _ = sqlx::query!(
                    r#"INSERT INTO ts_kv_latest
                       (entity_id, key, ts, bool_v, long_v, dbl_v, str_v, json_v, version)
                       VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,0)
                       ON CONFLICT (entity_id, key) DO UPDATE
                       SET ts=$3, bool_v=$4, long_v=$5, dbl_v=$6, str_v=$7"#,
                    entity_id, key_id, ts_now,
                    *bool_v, *long_v, *dbl_v,
                    *str_v
                ).execute(&pool).await;
                println!("  ✅ Telemetry: entity={entity_id_str} key={key}");
            }
            Err(e) => println!("  ⚠️  Telemetry key {key}: {e}"),
        }
    }

    // ── 13. Simulator Configs ──────────────────────────────────────────────────
    let sim_configs: Vec<(&str, &str, &str, &str)> = vec![
        (
            "00000000-0000-0000-0000-000000000091",
            "TempSensor Simulator",
            "00000000-0000-0000-0000-000000000051",
            r#"[
                {"key":"temperature","dataType":"DOUBLE","generator":{"type":"SINE_WAVE","amplitude":15.0,"offset":22.0,"periodMs":60000}},
                {"key":"humidity","dataType":"DOUBLE","generator":{"type":"RANDOM","min":30.0,"max":70.0}}
            ]"#,
        ),
        (
            "00000000-0000-0000-0000-000000000092",
            "SmartMeter Simulator",
            "00000000-0000-0000-0000-000000000053",
            r#"[
                {"key":"powerConsumption","dataType":"DOUBLE","generator":{"type":"LINEAR","start":0.0,"step":0.5,"max":100.0}},
                {"key":"voltage","dataType":"DOUBLE","generator":{"type":"RANDOM","min":218.0,"max":242.0}}
            ]"#,
        ),
    ];
    for (id_str, name, device_id_str, schema) in &sim_configs {
        let sim_id = Uuid::parse_str(id_str)?;
        let device_id = Uuid::parse_str(device_id_str)?;
        let schema_val: serde_json::Value = serde_json::from_str(schema)?;
        let result = sqlx::query!(
            r#"INSERT INTO simulator_config
               (id, tenant_id, device_id, name, enabled, interval_ms,
                telemetry_schema, created_time, updated_time)
               VALUES ($1,$2,$3,$4,false,5000,$5,$6,$7)
               ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name"#,
            sim_id, tenant_id, device_id, name, schema_val, ts_now, ts_now
        ).execute(&pool).await;
        match result {
            Ok(_) => println!("  ✅ SimulatorConfig: {name}"),
            Err(e) => println!("  ⚠️  SimulatorConfig {name}: {e}"),
        }
    }

    println!("\n✨ Seed hoàn tất!");
    println!("\n📋 Tài khoản test:");
    println!("  SYS_ADMIN    : sysadmin@thingsboard.org / sysadmin");
    println!("  TENANT_ADMIN : tenant@vielang.dev / tenant123");
    println!("  CUSTOMER_USER: customer@vielang.dev / customer123");
    println!("\n📊 Data đã tạo:");
    println!("  - 1 Tenant Profile, 1 Tenant, 1 Customer");
    println!("  - 3 users (SYS_ADMIN, TENANT_ADMIN, CUSTOMER_USER)");
    println!("  - 3 Device Profiles, 6 Devices");
    println!("  - 3 Assets, 3 Dashboards, 3 Alarms");
    println!("  - 8 telemetry data points (latest)");
    println!("  - 2 Simulator Configs (disabled, ready to start)");

    Ok(())
}

async fn ensure_asset_profile(pool: &PgPool, tenant_id: Uuid, asset_type: &str) -> Uuid {
    let pid = Uuid::new_v4();
    let _ = sqlx::query!(
        r#"INSERT INTO asset_profile
           (id, created_time, tenant_id, name, is_default)
           VALUES ($1,$2,$3,$4,false)
           ON CONFLICT DO NOTHING"#,
        pid, now_ms(), tenant_id, asset_type
    ).execute(pool).await;

    let row = sqlx::query!(
        "SELECT id FROM asset_profile WHERE tenant_id=$1 AND name=$2 LIMIT 1",
        tenant_id, asset_type
    ).fetch_optional(pool).await.ok().flatten();

    row.map(|r| r.id).unwrap_or(pid)
}
