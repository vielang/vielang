use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

use vl_cache::TbCache;
use vl_config::MqttTransportConfig;
use vl_core::entities::{ActivityEvent, TbMsg};
use vl_dao::{DbPool, TimeseriesDao};
use vl_queue::TbProducer;

mod auth;
pub mod codec;
pub mod gateway;
mod handler;
pub mod sparkplug;
pub mod ota;
pub mod session_store;
pub mod telemetry;
pub mod websocket;

pub use session_store::PersistentSessionStore;

pub async fn run(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          MqttTransportConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
    device_registry: Arc<crate::DeviceWriteRegistry>,
    rpc_pending:     Arc<crate::RpcPendingRegistry>,
    chunk_size_kb:   usize,
) {
    let addr: SocketAddr = match format!("{}:{}", config.bind, config.port).parse() {
        Ok(a) => a,
        Err(e) => {
            error!("Invalid MQTT bind address {}:{}: {}", config.bind, config.port, e);
            return;
        }
    };

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => {
            info!("MQTT server listening on {}", addr);
            l
        }
        Err(e) => {
            error!("Failed to bind MQTT server on {}: {}", addr, e);
            return;
        }
    };

    let rule_engine_tx = Arc::new(rule_engine_tx);
    let session_store  = Arc::new(PersistentSessionStore::new(100));

    // ── Optionally start MQTT-over-WebSocket server ────────────────────────────
    if config.ws_enabled {
        let ws_server = websocket::MqttWebSocketServer::new(
            config.clone(),
            pool.clone(),
            ts_dao.clone(),
            rule_engine_tx.clone(),
            queue_producer.clone(),
            cache.clone(),
            ws_tx.clone(),
            activity_tx.clone(),
            device_registry.clone(),
            rpc_pending.clone(),
            session_store.clone(),
            chunk_size_kb,
        );
        tokio::spawn(ws_server.run());
    }

    // ── TCP MQTT server ───────────────────────────────────────────────────────
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                let pool       = pool.clone();
                let ts_dao     = ts_dao.clone();
                let re_tx      = rule_engine_tx.clone();
                let producer   = queue_producer.clone();
                let cache      = cache.clone();
                let ws_tx      = ws_tx.clone();
                let act_tx     = activity_tx.clone();
                let dev_reg    = device_registry.clone();
                let rpc_pend   = rpc_pending.clone();
                let sess_store = session_store.clone();
                tracing::debug!("MQTT connection from {}", peer);
                tokio::spawn(handler::handle_connection(
                    stream, pool, ts_dao, re_tx, producer, cache, ws_tx,
                    act_tx, dev_reg, rpc_pend, sess_store, chunk_size_kb,
                ));
            }
            Err(e) => {
                error!("MQTT accept error: {}", e);
            }
        }
    }
}
