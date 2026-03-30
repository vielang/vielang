pub mod auth;
pub mod error;
pub mod mqtt;
#[cfg(feature = "http")]
pub mod http;
pub mod coap;
pub mod lwm2m;
pub mod snmp;
pub mod lorawan;

pub use error::TransportError;

use std::sync::Arc;
use vl_cache::TbCache;
use vl_config::{CoapTransportConfig, HttpTransportConfig, LoRaWanConfig, Lwm2mTransportConfig, MqttTransportConfig, SnmpConfig};
use vl_core::entities::{ActivityEvent, TbMsg};
use vl_dao::{DbPool, TimeseriesDao};
use vl_queue::TbProducer;
use tokio::sync::{broadcast, mpsc};

/// Shared registry: device_id → mpsc sender for writing raw MQTT packet bytes to device.
/// Populated on MQTT CONNECT, removed on DISCONNECT.
pub type DeviceWriteRegistry = dashmap::DashMap<uuid::Uuid, tokio::sync::mpsc::Sender<bytes::Bytes>>;

/// Pending two-way RPC responses: (device_id, request_id) → oneshot sender.
pub type RpcPendingRegistry = dashmap::DashMap<(uuid::Uuid, i32), tokio::sync::oneshot::Sender<serde_json::Value>>;

/// Start MQTT transport — call from main.rs via tokio::spawn.
pub async fn run_mqtt(
    pool: DbPool,
    ts_dao: Arc<dyn TimeseriesDao>,
    config: MqttTransportConfig,
    rule_engine_tx: Option<mpsc::Sender<TbMsg>>,
    queue_producer: Arc<dyn TbProducer>,
    cache: Arc<dyn TbCache>,
    ws_tx: broadcast::Sender<TbMsg>,
    activity_tx: mpsc::Sender<ActivityEvent>,
    device_registry: Arc<DeviceWriteRegistry>,
    rpc_pending: Arc<RpcPendingRegistry>,
    chunk_size_kb: usize,
) {
    mqtt::run(pool, ts_dao, config, rule_engine_tx, queue_producer, cache, ws_tx, activity_tx, device_registry, rpc_pending, chunk_size_kb).await;
}

/// Start HTTP Device API transport — call from main.rs via tokio::spawn.
#[cfg(feature = "http")]
pub async fn run_http(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          HttpTransportConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
    rpc_pending:     Arc<RpcPendingRegistry>,
) {
    http::run(pool, ts_dao, config, rule_engine_tx, queue_producer, cache, ws_tx, activity_tx, rpc_pending).await;
}

/// Start CoAP transport — call from main.rs via tokio::spawn.
pub async fn run_coap(
    pool: DbPool,
    ts_dao: Arc<dyn TimeseriesDao>,
    config: CoapTransportConfig,
    rule_engine_tx: Option<mpsc::Sender<TbMsg>>,
    queue_producer: Arc<dyn TbProducer>,
    cache: Arc<dyn TbCache>,
    ws_tx: broadcast::Sender<TbMsg>,
    activity_tx: mpsc::Sender<ActivityEvent>,
) {
    coap::run(pool, ts_dao, config, rule_engine_tx, queue_producer, cache, ws_tx, activity_tx).await;
}

/// Start LwM2M transport — call from main.rs via tokio::spawn.
pub async fn run_lwm2m(
    pool: DbPool,
    ts_dao: Arc<dyn TimeseriesDao>,
    config: Lwm2mTransportConfig,
    rule_engine_tx: Option<mpsc::Sender<TbMsg>>,
    queue_producer: Arc<dyn TbProducer>,
    cache: Arc<dyn TbCache>,
    ws_tx: broadcast::Sender<TbMsg>,
    activity_tx: mpsc::Sender<ActivityEvent>,
) {
    lwm2m::run(pool, ts_dao, config, rule_engine_tx, queue_producer, cache, ws_tx, activity_tx).await;
}

/// Start SNMP trap receiver transport — call from main.rs via tokio::spawn.
pub async fn run_snmp(
    pool: DbPool,
    ts_dao: Arc<dyn TimeseriesDao>,
    config: SnmpConfig,
    rule_engine_tx: Option<mpsc::Sender<TbMsg>>,
    queue_producer: Arc<dyn TbProducer>,
    cache: Arc<dyn TbCache>,
    ws_tx: broadcast::Sender<TbMsg>,
    activity_tx: mpsc::Sender<ActivityEvent>,
) {
    snmp::run(pool, ts_dao, config, rule_engine_tx, queue_producer, cache, ws_tx, activity_tx).await;
}

/// Start LoRaWAN ChirpStack bridge — call from main.rs via tokio::spawn.
pub async fn run_lorawan(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          LoRaWanConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
) {
    lorawan::run(pool, ts_dao, config, rule_engine_tx, queue_producer, cache, ws_tx, activity_tx).await;
}
