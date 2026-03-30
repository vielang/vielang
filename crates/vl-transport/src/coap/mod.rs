mod handler;
pub mod observe;

use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

use vl_cache::TbCache;
use vl_config::CoapTransportConfig;
use vl_core::entities::{ActivityEvent, TbMsg, msg_type};
use vl_dao::{DbPool, TimeseriesDao};
use vl_queue::TbProducer;

use handler::CoapContext;
use observe::ObserveRegistry;

pub async fn run(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          CoapTransportConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    _activity_tx:    mpsc::Sender<ActivityEvent>,
) {
    let addr = format!("{}:{}", config.bind, config.port);

    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => {
            info!("CoAP transport listening on udp:{}", addr);
            Arc::new(s)
        }
        Err(e) => {
            error!("Failed to bind CoAP transport on {}: {}", addr, e);
            return;
        }
    };

    let observe_registry = ObserveRegistry::new(socket.clone());

    // ── Background task: notify CoAP observers when attributes change ─────────
    if config.observe_enabled {
        let registry = observe_registry.clone();
        let mut notify_rx = ws_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(msg) = notify_rx.recv().await {
                // Notify observers for any message type that indicates attribute change.
                if msg.msg_type == msg_type::POST_ATTRIBUTES_REQUEST
                    || msg.msg_type == msg_type::ATTRIBUTE_UPDATED
                {
                    let device_id = msg.originator_id;
                    let payload   = msg.data.as_bytes().to_vec();
                    let reg       = registry.clone();
                    tokio::spawn(async move {
                        if let Err(e) = reg.notify_device(device_id, &payload).await {
                            tracing::debug!(
                                device_id = %device_id,
                                error = %e,
                                "CoAP Observe notification error"
                            );
                        }
                    });
                }
            }
        });
    }

    let ctx = Arc::new(CoapContext {
        pool,
        ts_dao,
        rule_engine_tx: Arc::new(rule_engine_tx),
        queue_producer,
        cache,
        ws_tx,
        observe_registry,
    });

    let mut buf = [0u8; 1500];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, peer)) => {
                let raw    = buf[..len].to_vec();
                let ctx    = ctx.clone();
                let socket = socket.clone();
                tokio::spawn(async move {
                    if let Some(response) = handler::handle_packet(&raw, peer, &ctx).await {
                        if let Err(e) = socket.send_to(&response, peer).await {
                            tracing::debug!("CoAP send error to {}: {}", peer, e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("CoAP recv error: {}", e);
            }
        }
    }
}
