mod handler;
pub mod bootstrap;
pub mod codec;
pub mod credentials;
pub mod ipso;
pub mod object_registry;
pub mod registration;
pub mod senml;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{error, info};

use vl_cache::TbCache;
use vl_config::Lwm2mTransportConfig;
use vl_core::entities::{ActivityEvent, TbMsg};
use vl_dao::{DbPool, TimeseriesDao};
use vl_queue::TbProducer;

use handler::Lwm2mContext;

/// Start LwM2M transport — binds a UDP socket and processes registration,
/// update, deregistration, and notify messages per OMA LwM2M 1.0/1.1.
pub async fn run(
    pool:           DbPool,
    ts_dao:         Arc<dyn TimeseriesDao>,
    config:         Lwm2mTransportConfig,
    rule_engine_tx: Option<mpsc::Sender<TbMsg>>,
    queue_producer: Arc<dyn TbProducer>,
    cache:          Arc<dyn TbCache>,
    ws_tx:          broadcast::Sender<TbMsg>,
    activity_tx:    mpsc::Sender<ActivityEvent>,
) {
    let addr = format!("{}:{}", config.bind, config.port);

    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => {
            info!("LwM2M transport listening on udp:{}", addr);
            Arc::new(s)
        }
        Err(e) => {
            error!("Failed to bind LwM2M transport on {}: {}", addr, e);
            return;
        }
    };

    let ctx = Arc::new(Lwm2mContext {
        pool,
        ts_dao,
        rule_engine_tx: Arc::new(rule_engine_tx),
        queue_producer,
        cache,
        ws_tx,
        activity_tx,
        registry: Arc::new(RwLock::new(HashMap::new())),
    });

    let mut buf = [0u8; 2048];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, peer)) => {
                let raw    = buf[..len].to_vec();
                let ctx    = ctx.clone();
                let socket = socket.clone();
                tokio::spawn(async move {
                    if let Some(response) = handler::handle_packet(&raw, peer, &ctx).await {
                        if let Err(e) = socket.send_to(&response, peer).await {
                            tracing::debug!("LwM2M send error to {}: {}", peer, e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("LwM2M recv error: {}", e);
            }
        }
    }
}
