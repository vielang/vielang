use std::collections::HashMap;
use std::sync::Arc;

use bytes::BytesMut;
use mqttbytes::v5::{ConnAck, ConnAckProperties, ConnectReturnCode, Packet};
use mqttbytes::QoS;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};
use uuid::Uuid;

use vl_cache::TbCache;
use vl_core::entities::{ActivityEvent, TbMsg, msg_type};
use vl_dao::{
    postgres::{device::DeviceDao, kv::KvDao},
    DbPool, TimeseriesDao,
};
use vl_queue::{TbProducer, topics};

use super::auth::authenticate;
use super::codec::{
    encode_connack, encode_pingresp, encode_puback, encode_pubcomp, encode_pubrec,
    encode_publish, encode_suback, read_packet,
};
use super::ota::OtaHandler;
use super::session_store::PersistentSessionStore;
use super::telemetry::{save_client_attributes, save_telemetry};
use crate::error::TransportError;

pub async fn handle_connection<S>(
    stream:          S,
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    rule_engine_tx:  Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
    device_registry: Arc<crate::DeviceWriteRegistry>,
    rpc_pending:     Arc<crate::RpcPendingRegistry>,
    session_store:   Arc<PersistentSessionStore>,
    chunk_size_kb:   usize,
) where S: AsyncRead + AsyncWrite + Unpin + Send + 'static {
    let peer = None::<std::net::SocketAddr>;
    if let Err(e) = do_handle(
        stream, pool, ts_dao, rule_engine_tx, queue_producer, cache,
        ws_tx, activity_tx, device_registry, rpc_pending, session_store, chunk_size_kb,
    ).await {
        if !is_disconnect_error(&e) {
            warn!("MQTT error from {:?}: {}", peer, e);
        } else {
            debug!("MQTT client {:?} disconnected", peer);
        }
    }
}

async fn do_handle<S>(
    stream:          S,
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    rule_engine_tx:  Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
    device_registry: Arc<crate::DeviceWriteRegistry>,
    rpc_pending:     Arc<crate::RpcPendingRegistry>,
    session_store:   Arc<PersistentSessionStore>,
    chunk_size_kb:   usize,
) -> Result<(), TransportError> where S: AsyncRead + AsyncWrite + Unpin + Send + 'static {
    let (mut reader, writer) = tokio::io::split(stream);

    // ── Step 1: CONNECT ───────────────────────────────────────────────────────
    let mut buf = BytesMut::with_capacity(4096);
    let connect = match read_packet(&mut reader, &mut buf).await? {
        Packet::Connect(c) => c,
        _ => return Err(TransportError::Protocol("Expected CONNECT packet".into())),
    };
    let client_id     = connect.client_id.clone();
    let clean_session = connect.clean_session;
    let will_message  = connect.last_will.clone();
    let username      = connect.login.as_ref().map(|l| l.username.as_str());
    let password      = connect.login.as_ref().map(|l| l.password.as_str());

    // ── Step 2: Authenticate ─────────────────────────────────────────────────
    let device_dao = DeviceDao::new(pool.clone());
    let device = match authenticate(&device_dao, &cache, username, password).await {
        Some(d) => d,
        None => {
            warn!(client_id = %client_id, "MQTT auth failed");
            let (write_tx, mut write_rx) = mpsc::channel::<bytes::Bytes>(4);
            let mut w = writer;
            tokio::spawn(async move {
                while let Some(data) = write_rx.recv().await {
                    w.write_all(&data).await.ok();
                }
            });
            write_tx.send(encode_connack(ConnAck {
                session_present: false,
                code: ConnectReturnCode::BadUserNamePassword,
                properties: None,
            })).await.ok();
            return Ok(());
        }
    };

    // ── Step 3: Spawn write task — owns the TCP writer ────────────────────────
    let (write_tx, mut write_rx) = mpsc::channel::<bytes::Bytes>(64);
    {
        let mut w = writer;
        tokio::spawn(async move {
            while let Some(data) = write_rx.recv().await {
                if w.write_all(&data).await.is_err() {
                    break;
                }
            }
        });
    }

    // ── Step 4: Persistent session handling ───────────────────────────────────
    let had_session = session_store.has_session(device.device_id);
    if clean_session {
        session_store.clear_session(device.device_id);
    }
    let session_present = !clean_session && had_session;

    // ── Step 5: Send CONNACK ──────────────────────────────────────────────────
    info!(device_id = %device.device_id, client_id = %client_id, "MQTT device connected");
    write_tx.send(encode_connack(ConnAck {
        session_present,
        code: ConnectReturnCode::Success,
        properties: {
            let mut props = ConnAckProperties::new();
            props.session_expiry_interval = Some(3600);       // 1 hour session
            props.receive_max = Some(65535);
            props.max_qos = Some(2);                          // Support QoS 0, 1, 2
            props.retain_available = Some(1);                 // 1 = true
            props.max_packet_size = Some(10 * 1024 * 1024);   // 10 MB
            props.topic_alias_max = Some(65535);
            props.wildcard_subscription_available = Some(1);   // 1 = true
            props.subscription_identifiers_available = Some(1);
            props.shared_subscription_available = Some(1);
            Some(props)
        },
    })).await.ok();

    // ── Step 6: Deliver pending messages (persistent session) ─────────────────
    if !clean_session {
        let pending = session_store.drain_pending(device.device_id);
        if !pending.is_empty() {
            debug!(device_id = %device.device_id, count = pending.len(), "Delivering queued messages");
            for msg in pending {
                write_tx.send(encode_publish(&msg.topic, &msg.payload)).await.ok();
            }
        }
    }

    // Register device in write registry
    device_registry.insert(device.device_id, write_tx.clone());

    // MQTT connection metrics
    metrics::counter!("vielang_mqtt_connections_total").increment(1);
    metrics::gauge!("vielang_mqtt_active_connections").increment(1.0);

    // Emit connect activity event
    let now = now_ms();
    activity_tx.send(ActivityEvent::Connected { device_id: device.device_id, ts: now }).await.ok();

    // Publish CONNECT_EVENT to rule engine + transport requests topic
    let connect_msg = TbMsg::new(msg_type::CONNECT_EVENT, device.device_id, "DEVICE", "{}")
        .with_tenant(device.tenant_id);
    send_direct(&rule_engine_tx, connect_msg.clone());
    publish_to_queue(&queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &connect_msg).await;

    // ── Step 7: Message loop ──────────────────────────────────────────────────
    let kv_dao      = KvDao::new(pool.clone());
    let ota_handler = OtaHandler::new(pool.clone(), chunk_size_kb);
    let mut qos2_in_flight: HashMap<u16, (String, bytes::Bytes)> = HashMap::new();
    let mut graceful_disconnect = false;

    loop {
        match read_packet(&mut reader, &mut buf).await {
            Ok(Packet::Publish(publish)) => {
                match publish.qos {
                    QoS::AtLeastOnce => {
                        write_tx.send(encode_puback(publish.pkid)).await.ok();
                        on_publish(
                            &ts_dao, &kv_dao, &write_tx,
                            device.device_id, device.tenant_id, &publish,
                            &rule_engine_tx, &queue_producer, &ws_tx,
                            &activity_tx, &rpc_pending, &ota_handler, &pool,
                        ).await;
                    }
                    QoS::ExactlyOnce => {
                        let pkid = publish.pkid;
                        if !qos2_in_flight.contains_key(&pkid) {
                            qos2_in_flight.insert(pkid, (publish.topic.clone(), publish.payload.clone()));
                        }
                        write_tx.send(encode_pubrec(pkid)).await.ok();
                        debug!(device_id = %device.device_id, pkid = pkid, "QoS2 PUBREC sent");
                    }
                    QoS::AtMostOnce => {
                        on_publish(
                            &ts_dao, &kv_dao, &write_tx,
                            device.device_id, device.tenant_id, &publish,
                            &rule_engine_tx, &queue_producer, &ws_tx,
                            &activity_tx, &rpc_pending, &ota_handler, &pool,
                        ).await;
                    }
                }
            }
            Ok(Packet::PubRel(pubrel)) => {
                let pkid = pubrel.pkid;
                if let Some((topic, payload)) = qos2_in_flight.remove(&pkid) {
                    debug!(device_id = %device.device_id, pkid = pkid, "QoS2 PUBREL: processing stored payload");
                    let mut synthetic = mqttbytes::v5::Publish::new(
                        &topic,
                        QoS::AtMostOnce,
                        payload.to_vec(),
                    );
                    synthetic.pkid = pkid;
                    on_publish(
                        &ts_dao, &kv_dao, &write_tx,
                        device.device_id, device.tenant_id, &synthetic,
                        &rule_engine_tx, &queue_producer, &ws_tx,
                        &activity_tx, &rpc_pending, &ota_handler, &pool,
                    ).await;
                } else {
                    debug!(device_id = %device.device_id, pkid = pkid, "QoS2 PUBREL for unknown pkid");
                }
                write_tx.send(encode_pubcomp(pkid)).await.ok();
                debug!(device_id = %device.device_id, pkid = pkid, "QoS2 PUBCOMP sent");
            }
            Ok(Packet::PingReq) => {
                write_tx.send(encode_pingresp()).await.ok();
            }
            Ok(Packet::Subscribe(sub)) => {
                let n = sub.filters.len();
                write_tx.send(encode_suback(sub.pkid, n)).await.ok();
                for filter in &sub.filters {
                    debug!(device_id = %device.device_id, filter = %filter.path, "MQTT subscribe");
                }
            }
            Ok(Packet::Unsubscribe(_)) => {}
            Ok(Packet::Disconnect(_)) => {
                info!(device_id = %device.device_id, "MQTT device disconnected cleanly");
                graceful_disconnect = true;
                device_registry.remove(&device.device_id);
                metrics::gauge!("vielang_mqtt_active_connections").decrement(1.0);
                activity_tx.send(ActivityEvent::Disconnected { device_id: device.device_id, ts: now_ms() }).await.ok();
                let disc_msg = TbMsg::new(msg_type::DISCONNECT_EVENT, device.device_id, "DEVICE", "{}")
                    .with_tenant(device.tenant_id);
                send_direct(&rule_engine_tx, disc_msg.clone());
                publish_to_queue(&queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &disc_msg).await;
                break;
            }
            Ok(_) => {}
            Err(e) if is_disconnect_error(&e) => {
                info!(device_id = %device.device_id, "MQTT connection closed");
                device_registry.remove(&device.device_id);
                metrics::gauge!("vielang_mqtt_active_connections").decrement(1.0);
                activity_tx.send(ActivityEvent::Disconnected { device_id: device.device_id, ts: now_ms() }).await.ok();
                let disc_msg = TbMsg::new(msg_type::DISCONNECT_EVENT, device.device_id, "DEVICE", "{}")
                    .with_tenant(device.tenant_id);
                send_direct(&rule_engine_tx, disc_msg.clone());
                publish_to_queue(&queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &disc_msg).await;
                break;
            }
            Err(e) => {
                device_registry.remove(&device.device_id);
                // Publish will before returning error
                if let Some(ref will) = will_message {
                    publish_will(
                        &ts_dao, &kv_dao, &write_tx,
                        device.device_id, device.tenant_id, will,
                        &rule_engine_tx, &queue_producer, &ws_tx,
                        &activity_tx, &rpc_pending, &ota_handler, &pool,
                    ).await;
                }
                return Err(e);
            }
        }
    }

    // ── Will message on abnormal disconnect ───────────────────────────────────
    if !graceful_disconnect {
        if let Some(ref will) = will_message {
            publish_will(
                &ts_dao, &kv_dao, &write_tx,
                device.device_id, device.tenant_id, will,
                &rule_engine_tx, &queue_producer, &ws_tx,
                &activity_tx, &rpc_pending, &ota_handler, &pool,
            ).await;
        }
    }

    Ok(())
}

/// Publish a will message by creating a synthetic Publish and routing through on_publish.
async fn publish_will(
    ts_dao:          &Arc<dyn TimeseriesDao>,
    kv_dao:          &KvDao,
    write_tx:        &mpsc::Sender<bytes::Bytes>,
    device_id:       Uuid,
    tenant_id:       Uuid,
    will:            &mqttbytes::v5::LastWill,
    rule_engine_tx:  &Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer:  &Arc<dyn TbProducer>,
    ws_tx:           &broadcast::Sender<TbMsg>,
    activity_tx:     &mpsc::Sender<ActivityEvent>,
    rpc_pending:     &Arc<crate::RpcPendingRegistry>,
    ota_handler:     &OtaHandler,
    pool:            &DbPool,
) {
    debug!(device_id = %device_id, topic = %will.topic, "Publishing MQTT will message");
    let synthetic = mqttbytes::v5::Publish::new(&will.topic, will.qos, will.message.to_vec());
    on_publish(
        ts_dao, kv_dao, write_tx,
        device_id, tenant_id, &synthetic,
        rule_engine_tx, queue_producer, ws_tx,
        activity_tx, rpc_pending, ota_handler, pool,
    ).await;
}

#[allow(clippy::too_many_arguments)]
async fn on_publish(
    ts_dao:         &Arc<dyn TimeseriesDao>,
    kv_dao:         &KvDao,
    write_tx:       &mpsc::Sender<bytes::Bytes>,
    device_id:      Uuid,
    tenant_id:      Uuid,
    publish:        &mqttbytes::v5::Publish,
    rule_engine_tx: &Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer: &Arc<dyn TbProducer>,
    ws_tx:          &broadcast::Sender<TbMsg>,
    activity_tx:    &mpsc::Sender<ActivityEvent>,
    rpc_pending:    &Arc<crate::RpcPendingRegistry>,
    ota_handler:    &OtaHandler,
    pool:           &DbPool,
) {
    let topic   = &publish.topic;
    let payload = &publish.payload;

    metrics::counter!("vielang_mqtt_messages_received_total",
        "type" => topic.as_str().to_owned()
    ).increment(1);

    // ── OTA firmware chunk request: v2/fw/request/{requestId}/chunk/{chunkIndex} ──
    if let Some(rest) = topic.strip_prefix("v2/fw/request/") {
        if let Some((request_id, chunk_part)) = rest.split_once('/') {
            if let Some(chunk_str) = chunk_part.strip_prefix("chunk/") {
                if let Ok(chunk_index) = chunk_str.parse::<u32>() {
                    debug!(
                        device_id = %device_id,
                        request_id = request_id,
                        chunk_index = chunk_index,
                        "OTA chunk request"
                    );
                    ota_handler.handle_chunk_request(write_tx, device_id, request_id, chunk_index).await;
                    return;
                }
            }
        }
    }

    if topic == "v1/devices/me/telemetry" {
        match save_telemetry(ts_dao, "DEVICE", device_id, payload).await {
            Ok(n) => {
                debug!(device_id = %device_id, entries = n, "Saved telemetry");
                activity_tx.send(ActivityEvent::Telemetry { device_id, ts: now_ms() }).await.ok();
                // Check for OTA state update in telemetry payload
                if let Ok(telemetry) = serde_json::from_slice::<serde_json::Value>(payload) {
                    if let Some(fw_state) = telemetry.get("current_fw_state").and_then(|v| v.as_str()) {
                        let error_msg = telemetry.get("current_fw_error").and_then(|v| v.as_str());
                        ota_handler.handle_telemetry_fw_state(device_id, fw_state, error_msg).await;
                    }
                }
                let data = String::from_utf8_lossy(payload).to_string();
                let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, device_id, "DEVICE", &data)
                    .with_tenant(tenant_id);
                send_direct(rule_engine_tx, msg.clone());
                publish_to_queue(queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
                let _ = ws_tx.send(msg);
            }
            Err(e) => warn!(device_id = %device_id, error = %e, "Failed to save telemetry"),
        }
    } else if topic == "v1/devices/me/attributes" {
        match save_client_attributes(kv_dao, device_id, payload).await {
            Ok(n) => {
                debug!(device_id = %device_id, entries = n, "Saved client attributes");
                let data = String::from_utf8_lossy(payload).to_string();
                let msg = TbMsg::new(msg_type::POST_ATTRIBUTES_REQUEST, device_id, "DEVICE", &data)
                    .with_tenant(tenant_id);
                send_direct(rule_engine_tx, msg.clone());
                publish_to_queue(queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
                let _ = ws_tx.send(msg);
            }
            Err(e) => warn!(device_id = %device_id, error = %e, "Failed to save attributes"),
        }
    } else if let Some(request_id) = topic.strip_prefix("v1/devices/me/attributes/request/") {
        on_attribute_request(write_tx, device_id, request_id).await;
    } else if let Some(request_id) = topic.strip_prefix("v1/devices/me/rpc/request/") {
        info!(device_id = %device_id, request_id = request_id, "MQTT RPC request");
    } else if let Some(response_id) = topic.strip_prefix("v1/devices/me/rpc/response/") {
        if let Ok(req_id) = response_id.parse::<i32>() {
            if let Some((_, tx)) = rpc_pending.remove(&(device_id, req_id)) {
                let response: serde_json::Value = serde_json::from_slice(payload)
                    .unwrap_or(serde_json::Value::Null);
                tx.send(response).ok();
            }
        }
        info!(device_id = %device_id, response_id = response_id, "MQTT RPC response received");
    } else if topic == "v1/devices/me/provision" || topic == "/provision" {
        // ── MQTT Device Provisioning ──────────────────────────────────────
        on_provision(write_tx, pool, payload).await;
    } else if topic.starts_with("v1/gateway/") {
        // ── MQTT Gateway Mode ─────────────────────────────────────────────
        on_gateway_msg(write_tx, pool, ts_dao, device_id, tenant_id, topic, payload).await;
    } else if super::sparkplug::SparkplugTopic::is_sparkplug(topic) {
        // ── Sparkplug B Protocol ──────────────────────────────────────────
        on_sparkplug(ts_dao, device_id, topic, payload).await;
    } else {
        debug!(device_id = %device_id, topic = %topic, "Unknown MQTT topic");
    }
}

async fn on_attribute_request(
    write_tx:   &mpsc::Sender<bytes::Bytes>,
    device_id:  Uuid,
    request_id: &str,
) {
    let response_topic = format!("v1/devices/me/attributes/response/{}", request_id);
    if let Err(e) = write_tx.send(encode_publish(&response_topic, b"{}")).await {
        warn!(device_id = %device_id, error = %e, "Failed to send attribute response");
    }
}

/// Handle Sparkplug B messages (spBv1.0/* topics).
async fn on_sparkplug(
    ts_dao:    &Arc<dyn TimeseriesDao>,
    device_id: Uuid,
    topic:     &str,
    payload:   &bytes::Bytes,
) {
    let Some(sp_topic) = super::sparkplug::SparkplugTopic::parse(topic) else {
        debug!(device_id = %device_id, topic = %topic, "Invalid Sparkplug topic");
        return;
    };

    debug!(
        device_id = %device_id,
        group = %sp_topic.group_id,
        msg_type = %sp_topic.message_type.as_str(),
        node = %sp_topic.edge_node_id,
        sp_device = ?sp_topic.device_id,
        "Sparkplug B message"
    );

    if sp_topic.message_type.has_metrics() {
        let metrics = super::sparkplug::parse_sparkplug_json(payload);
        if !metrics.is_empty() {
            let records = super::sparkplug::metrics_to_ts_records(device_id, &metrics);
            if let Err(e) = ts_dao.save_batch("DEVICE", &records).await {
                warn!(device_id = %device_id, "Sparkplug telemetry save failed: {e}");
            }
            if let Err(e) = ts_dao.save_latest_batch("DEVICE", &records).await {
                warn!(device_id = %device_id, "Sparkplug latest save failed: {e}");
            }
            info!(
                device_id = %device_id,
                metrics = records.len(),
                msg_type = %sp_topic.message_type.as_str(),
                "Sparkplug B metrics saved"
            );
        }
    }

    if sp_topic.message_type.is_birth() {
        info!(device_id = %device_id, node = %sp_topic.edge_node_id, "Sparkplug birth certificate");
    } else if sp_topic.message_type.is_death() {
        info!(device_id = %device_id, node = %sp_topic.edge_node_id, "Sparkplug death certificate");
    }
}

/// Handle MQTT gateway messages (v1/gateway/* topics).
/// Uses a per-connection GatewaySession (created lazily on first gateway msg).
async fn on_gateway_msg(
    write_tx:  &mpsc::Sender<bytes::Bytes>,
    pool:      &DbPool,
    ts_dao:    &Arc<dyn TimeseriesDao>,
    device_id: Uuid,
    tenant_id: Uuid,
    topic:     &str,
    payload:   &bytes::Bytes,
) {
    // Lazy gateway session — in production, this would be stored per-connection.
    // For now, create a fresh session per message (stateless gateway).
    let mut gw = super::gateway::GatewaySession::new(
        device_id, tenant_id, pool.clone(), ts_dao.clone(),
    );

    let sub_topic = topic.strip_prefix("v1/gateway/").unwrap_or("");
    match sub_topic {
        "connect" => {
            if let Some(child_id) = gw.on_connect(payload).await {
                let resp = serde_json::json!({"device": child_id.to_string()});
                write_tx.send(encode_publish("v1/gateway/connect/response", resp.to_string().as_bytes())).await.ok();
            }
        }
        "disconnect" => {
            gw.on_disconnect(payload);
        }
        "telemetry" => {
            gw.on_telemetry(payload).await;
        }
        "attributes" => {
            gw.on_attributes(payload).await;
        }
        "claim" => {
            debug!(gateway = %device_id, "Gateway claim request (forwarded)");
        }
        "rpc" => {
            debug!(gateway = %device_id, "Gateway RPC response (forwarded)");
        }
        _ => {
            debug!(gateway = %device_id, topic = %topic, "Unknown gateway sub-topic");
        }
    }
}

/// Handle MQTT device provisioning request.
/// Topic: v1/devices/me/provision
/// Publishes response to /provision/response
async fn on_provision(
    write_tx: &mpsc::Sender<bytes::Bytes>,
    pool: &DbPool,
    payload: &bytes::Bytes,
) {
    let response_topic = "/provision/response";

    let body: serde_json::Value = match serde_json::from_slice(payload) {
        Ok(v) => v,
        Err(_) => {
            let resp = serde_json::json!({"status": "FAILURE", "errorMsg": "Invalid JSON"});
            write_tx.send(encode_publish(response_topic, resp.to_string().as_bytes())).await.ok();
            return;
        }
    };

    let device_name = body.get("deviceName").and_then(|v| v.as_str()).unwrap_or("");
    let provision_key = body.get("provisionDeviceKey").and_then(|v| v.as_str()).unwrap_or("");
    let provision_secret = body.get("provisionDeviceSecret").and_then(|v| v.as_str()).unwrap_or("");

    if device_name.is_empty() || provision_key.is_empty() {
        let resp = serde_json::json!({"status": "FAILURE", "errorMsg": "deviceName and provisionDeviceKey required"});
        write_tx.send(encode_publish(response_topic, resp.to_string().as_bytes())).await.ok();
        return;
    }

    let profile_dao = vl_dao::postgres::device_profile::DeviceProfileDao::new(pool.clone());
    let profile = match profile_dao.find_by_provision_key(provision_key).await {
        Ok(Some(p)) => p,
        _ => {
            let resp = serde_json::json!({"status": "FAILURE", "errorMsg": "Invalid provision key"});
            write_tx.send(encode_publish(response_topic, resp.to_string().as_bytes())).await.ok();
            return;
        }
    };

    // Validate secret
    let expected_secret = profile.profile_data.as_ref()
        .and_then(|d| d.get("provisionConfiguration"))
        .and_then(|c| c.get("provisionDeviceSecret"))
        .and_then(|s| s.as_str())
        .unwrap_or("");
    if !expected_secret.is_empty() && provision_secret != expected_secret {
        let resp = serde_json::json!({"status": "FAILURE", "errorMsg": "Invalid provision secret"});
        write_tx.send(encode_publish(response_topic, resp.to_string().as_bytes())).await.ok();
        return;
    }

    use vl_core::entities::device_profile::DeviceProvisionType;
    let device_dao = vl_dao::postgres::device::DeviceDao::new(pool.clone());

    let resp = match profile.provision_type {
        DeviceProvisionType::AllowCreateNewDevices => {
            // Check if device exists
            if let Ok(Some(existing)) = device_dao.find_by_name(profile.tenant_id, device_name).await {
                if let Ok(Some(creds)) = device_dao.get_credentials(existing.id).await {
                    serde_json::json!({
                        "status": "SUCCESS",
                        "credentialsType": creds.credentials_type,
                        "credentialsValue": creds.credentials_id
                    })
                } else {
                    serde_json::json!({"status": "FAILURE", "errorMsg": "Credentials not found"})
                }
            } else {
                // Create new device
                let now = chrono::Utc::now().timestamp_millis();
                let device = vl_core::entities::Device {
                    id: uuid::Uuid::new_v4(),
                    created_time: now,
                    tenant_id: profile.tenant_id,
                    customer_id: None,
                    device_profile_id: profile.id,
                    name: device_name.to_string(),
                    device_type: profile.name.clone(),
                    label: None, device_data: None,
                    firmware_id: profile.firmware_id, software_id: profile.software_id,
                    external_id: None, additional_info: None, version: 1,
                };
                if device_dao.save(&device).await.is_err() {
                    serde_json::json!({"status": "FAILURE", "errorMsg": "Failed to create device"})
                } else {
                    let token = uuid::Uuid::new_v4().to_string().replace('-', "");
                    let creds = vl_core::entities::DeviceCredentials {
                        id: uuid::Uuid::new_v4(), created_time: now,
                        device_id: device.id,
                        credentials_type: vl_core::entities::DeviceCredentialsType::AccessToken,
                        credentials_id: token.clone(), credentials_value: None,
                    };
                    if device_dao.save_credentials(&creds).await.is_err() {
                        serde_json::json!({"status": "FAILURE", "errorMsg": "Failed to save credentials"})
                    } else {
                        info!(device_name = %device_name, device_id = %device.id, "MQTT device provisioned");
                        serde_json::json!({
                            "status": "SUCCESS",
                            "credentialsType": "ACCESS_TOKEN",
                            "credentialsValue": token
                        })
                    }
                }
            }
        }
        DeviceProvisionType::CheckPreProvisionedDevices => {
            match device_dao.find_by_name(profile.tenant_id, device_name).await {
                Ok(Some(d)) => match device_dao.get_credentials(d.id).await {
                    Ok(Some(c)) => serde_json::json!({
                        "status": "SUCCESS",
                        "credentialsType": c.credentials_type,
                        "credentialsValue": c.credentials_id
                    }),
                    _ => serde_json::json!({"status": "FAILURE", "errorMsg": "Credentials not found"}),
                },
                _ => serde_json::json!({"status": "FAILURE", "errorMsg": "Device not found"}),
            }
        }
        _ => serde_json::json!({"status": "FAILURE", "errorMsg": "Provisioning disabled for this profile"}),
    };

    write_tx.send(encode_publish(response_topic, resp.to_string().as_bytes())).await.ok();
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn send_direct(tx: &Arc<Option<mpsc::Sender<TbMsg>>>, msg: TbMsg) {
    if let Some(sender) = tx.as_ref() {
        if let Err(e) = sender.try_send(msg) {
            debug!("Rule engine direct channel: {}", e);
        }
    }
}

async fn publish_to_queue(producer: &Arc<dyn TbProducer>, topic: &str, msg: &TbMsg) {
    if let Err(e) = producer.send_tb_msg(topic, msg).await {
        debug!("Queue publish error on {}: {}", topic, e);
    }
}

fn is_disconnect_error(e: &TransportError) -> bool {
    match e {
        TransportError::Io(io) => matches!(
            io.kind(),
            std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::UnexpectedEof
        ),
        _ => false,
    }
}
