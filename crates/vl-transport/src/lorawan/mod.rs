/// LoRaWAN bridge — subscribes to a ChirpStack MQTT broker and ingests
/// uplink / join / status events into the VieLang rule engine.
///
/// Design:
///   LoRa Device → LoRa Gateway → ChirpStack NS → MQTT → LoRaWanBridge → Rule Engine
///
/// Device lookup: devices register their `lora_dev_eui` via
/// `POST /api/device/{id}/lorawan`; the bridge resolves incoming uplinks to
/// the matching VieLang device.
pub mod chirpstack;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde_json::json;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use vl_cache::TbCache;
use vl_config::LoRaWanConfig;
use vl_core::entities::{ActivityEvent, TbMsg, msg_type};
use vl_dao::{
    postgres::{device::DeviceDao, kv::KvDao},
    DbPool, TimeseriesDao,
};
use vl_queue::{TbProducer, topics};

use crate::mqtt::telemetry::save_telemetry;
use chirpstack::{parse_topic, ChirpStackJoin, ChirpStackStatus, ChirpStackUplink};

// ── Entry point ───────────────────────────────────────────────────────────────

/// Start the LoRaWAN bridge. Returns immediately if `config.enabled = false`.
pub async fn run(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          LoRaWanConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    _cache:          Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
) {
    // Parse broker URL: "mqtt://host:port" or "mqtt://host"
    let url = config.chirpstack_url.trim_start_matches("mqtt://");
    let (host, port) = if let Some((h, p)) = url.split_once(':') {
        (h.to_string(), p.parse::<u16>().unwrap_or(1883))
    } else {
        (url.to_string(), 1883u16)
    };

    let client_id = format!("vielang-lorawan-{}", uuid::Uuid::new_v4().simple());
    let mut opts = MqttOptions::new(client_id, &host, port);
    opts.set_keep_alive(Duration::from_secs(30));
    opts.set_clean_session(true);

    if !config.username.is_empty() {
        opts.set_credentials(&config.username, &config.password);
    }

    let (client, mut event_loop) = AsyncClient::new(opts, 128);

    // Build subscription topics — filter by application_ids or use wildcard.
    let topics_to_sub: Vec<String> = if config.application_ids.is_empty() {
        vec![
            "application/+/device/+/event/up".into(),
            "application/+/device/+/event/join".into(),
            "application/+/device/+/event/status".into(),
        ]
    } else {
        config.application_ids.iter().flat_map(|app_id| {
            vec![
                format!("application/{app_id}/device/+/event/up"),
                format!("application/{app_id}/device/+/event/join"),
                format!("application/{app_id}/device/+/event/status"),
            ]
        }).collect()
    };

    // Subscribe after connection is established (first ConnAck).
    let client_clone       = client.clone();
    let topics_clone       = topics_to_sub.clone();
    let pool_clone         = pool.clone();
    let ts_dao_clone       = ts_dao.clone();
    let rule_engine_clone  = Arc::new(rule_engine_tx);
    let producer_clone     = queue_producer.clone();
    let ws_clone           = ws_tx.clone();
    let activity_clone     = activity_tx.clone();

    info!(broker = %config.chirpstack_url, "LoRaWAN bridge starting");

    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                info!("LoRaWAN: connected to ChirpStack broker {}", config.chirpstack_url);
                for topic in &topics_clone {
                    if let Err(e) = client_clone.subscribe(topic, QoS::AtLeastOnce).await {
                        error!("LoRaWAN: subscribe to '{}' failed: {}", topic, e);
                    } else {
                        debug!("LoRaWAN: subscribed to '{}'", topic);
                    }
                }
            }

            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                let mqtt_topic = &publish.topic;
                let payload    = publish.payload.to_vec();

                let (dev_eui, event) = match parse_topic(mqtt_topic) {
                    Some(t) => t,
                    None    => {
                        debug!("LoRaWAN: ignoring unknown topic '{}'", mqtt_topic);
                        continue;
                    }
                };

                let pool_h         = pool_clone.clone();
                let ts_dao_h       = ts_dao_clone.clone();
                let re_h           = rule_engine_clone.clone();
                let producer_h     = producer_clone.clone();
                let ws_h           = ws_clone.clone();
                let activity_h     = activity_clone.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_event(
                        &dev_eui, &event, &payload,
                        &pool_h, &ts_dao_h, re_h, &producer_h, ws_h, activity_h,
                    ).await {
                        warn!(dev_eui = %dev_eui, event = %event, "LoRaWAN event error: {}", e);
                    }
                });
            }

            Ok(Event::Incoming(Incoming::Disconnect)) => {
                warn!("LoRaWAN: disconnected from broker — will reconnect");
            }

            Err(e) => {
                error!("LoRaWAN: MQTT event loop error: {}  — retrying in 10s", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
            }

            _ => {}
        }
    }
}

// ── Event dispatcher ──────────────────────────────────────────────────────────

async fn handle_event(
    dev_eui:        &str,
    event:          &str,
    payload:        &[u8],
    pool:           &DbPool,
    ts_dao:         &Arc<dyn TimeseriesDao>,
    rule_engine_tx: Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer: &Arc<dyn TbProducer>,
    ws_tx:          broadcast::Sender<TbMsg>,
    activity_tx:    mpsc::Sender<ActivityEvent>,
) -> anyhow::Result<()> {
    match event {
        "up" => {
            let uplink: ChirpStackUplink = serde_json::from_slice(payload)?;
            handle_uplink(dev_eui, uplink, pool, ts_dao, rule_engine_tx, queue_producer, ws_tx, activity_tx).await
        }
        "join" => {
            let join: ChirpStackJoin = serde_json::from_slice(payload)?;
            handle_join(dev_eui, join, pool, rule_engine_tx, ws_tx, activity_tx).await
        }
        "status" => {
            let status: ChirpStackStatus = serde_json::from_slice(payload)?;
            handle_status(dev_eui, status, pool, ts_dao, rule_engine_tx, queue_producer, ws_tx).await
        }
        other => {
            debug!("LoRaWAN: unhandled event type '{}' for dev_eui '{}'", other, dev_eui);
            Ok(())
        }
    }
}

// ── Uplink handler ────────────────────────────────────────────────────────────

async fn handle_uplink(
    dev_eui:        &str,
    uplink:         ChirpStackUplink,
    pool:           &DbPool,
    ts_dao:         &Arc<dyn TimeseriesDao>,
    rule_engine_tx: Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer: &Arc<dyn TbProducer>,
    ws_tx:          broadcast::Sender<TbMsg>,
    activity_tx:    mpsc::Sender<ActivityEvent>,
) -> anyhow::Result<()> {
    let device_dao = DeviceDao::new(pool.clone());
    let device = match device_dao.find_by_lora_dev_eui(dev_eui).await? {
        Some(d) => d,
        None => {
            warn!(dev_eui = %dev_eui, "LoRaWAN: unknown dev_eui — no linked device");
            return Ok(());
        }
    };

    // Decode base64 application payload
    let raw_bytes = base64::engine::general_purpose::STANDARD
        .decode(&uplink.data)
        .unwrap_or_default();

    // Try to parse payload as JSON telemetry; fall back to hex string
    let mut telemetry: HashMap<String, serde_json::Value> = if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&raw_bytes) {
        if let Some(obj) = v.as_object() {
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    // If payload couldn't be decoded as JSON, store raw hex
    if telemetry.is_empty() && !raw_bytes.is_empty() {
        let hex = raw_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        telemetry.insert("rawPayload".to_string(), json!(hex));
    }

    // Add LoRa signal metadata
    if let Some(rx) = uplink.rx_info.first() {
        telemetry.insert("rssi".to_string(), json!(rx.rssi));
        telemetry.insert("snr".to_string(),  json!(rx.snr));
    }
    telemetry.insert("fCnt".to_string(),    json!(uplink.f_cnt));
    telemetry.insert("fPort".to_string(),   json!(uplink.f_port));
    telemetry.insert("confirmed".to_string(), json!(uplink.confirmed));

    let kv_json  = serde_json::Value::Object(telemetry.into_iter().collect());
    let kv_bytes = serde_json::to_vec(&kv_json)?;

    let saved = save_telemetry(ts_dao, "DEVICE", device.id, &kv_bytes).await?;
    debug!(dev_eui = %dev_eui, device_id = %device.id, entries = saved, "LoRaWAN telemetry saved");

    let ts = chrono::Utc::now().timestamp_millis();
    let _ = activity_tx.try_send(ActivityEvent::Telemetry { device_id: device.id, ts });

    let data_str = String::from_utf8_lossy(&kv_bytes).to_string();
    let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, device.id, "DEVICE", &data_str);
    send_to_rule_engine(&rule_engine_tx, msg.clone());
    publish_to_queue(queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
    let _ = ws_tx.send(msg);

    Ok(())
}

// ── Join handler ──────────────────────────────────────────────────────────────

async fn handle_join(
    dev_eui:        &str,
    join:           ChirpStackJoin,
    pool:           &DbPool,
    rule_engine_tx: Arc<Option<mpsc::Sender<TbMsg>>>,
    ws_tx:          broadcast::Sender<TbMsg>,
    activity_tx:    mpsc::Sender<ActivityEvent>,
) -> anyhow::Result<()> {
    let device_dao = DeviceDao::new(pool.clone());
    let device = match device_dao.find_by_lora_dev_eui(dev_eui).await? {
        Some(d) => d,
        None => {
            debug!(dev_eui = %dev_eui, "LoRaWAN: join from unknown dev_eui — ignoring");
            return Ok(());
        }
    };

    let ts = chrono::Utc::now().timestamp_millis();
    let _ = activity_tx.try_send(ActivityEvent::Connected { device_id: device.id, ts });

    info!(dev_eui = %dev_eui, device_id = %device.id, dev_addr = %join.dev_addr, "LoRaWAN: device joined");

    let msg = TbMsg::new(
        msg_type::CONNECT_EVENT,
        device.id,
        "DEVICE",
        &format!(r#"{{"devAddr":"{}"}}"#, join.dev_addr),
    );
    send_to_rule_engine(&rule_engine_tx, msg.clone());
    let _ = ws_tx.send(msg);
    Ok(())
}

// ── Status handler ────────────────────────────────────────────────────────────

async fn handle_status(
    dev_eui:        &str,
    status:         ChirpStackStatus,
    pool:           &DbPool,
    ts_dao:         &Arc<dyn TimeseriesDao>,
    rule_engine_tx: Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer: &Arc<dyn TbProducer>,
    ws_tx:          broadcast::Sender<TbMsg>,
) -> anyhow::Result<()> {
    let device_dao = DeviceDao::new(pool.clone());
    let device = match device_dao.find_by_lora_dev_eui(dev_eui).await? {
        Some(d) => d,
        None => return Ok(()),
    };

    let mut telemetry: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    if !status.battery_level_unavailable && !status.external_power_source {
        // battery_level 0–254 maps to 0–100%
        let pct = (status.battery_level as f64 / 254.0 * 100.0).round() as u8;
        telemetry.insert("batteryLevel".into(), json!(pct));
    }
    telemetry.insert("loraMargin".into(), json!(status.margin));

    if telemetry.is_empty() { return Ok(()); }

    let kv_bytes = serde_json::to_vec(&telemetry)?;
    save_telemetry(ts_dao, "DEVICE", device.id, &kv_bytes).await?;

    let data_str = String::from_utf8_lossy(&kv_bytes).to_string();
    let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, device.id, "DEVICE", &data_str);
    send_to_rule_engine(&rule_engine_tx, msg.clone());
    publish_to_queue(queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
    let _ = ws_tx.send(msg);
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn send_to_rule_engine(tx: &Arc<Option<mpsc::Sender<TbMsg>>>, msg: TbMsg) {
    if let Some(sender) = tx.as_ref() {
        if let Err(e) = sender.try_send(msg) {
            debug!("LoRaWAN: rule engine channel: {}", e);
        }
    }
}

async fn publish_to_queue(producer: &Arc<dyn TbProducer>, topic: &str, msg: &TbMsg) {
    if let Err(e) = producer.send_tb_msg(topic, msg).await {
        debug!("LoRaWAN: queue publish error: {}", e);
    }
}
