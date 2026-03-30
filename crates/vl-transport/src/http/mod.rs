pub mod handler;

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

use vl_cache::TbCache;
use vl_config::HttpTransportConfig;
use vl_core::entities::{ActivityEvent, TbMsg};
use vl_dao::{DbPool, TimeseriesDao};
use vl_queue::TbProducer;

use handler::HttpTransportState;

pub async fn run(
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    config:          HttpTransportConfig,
    rule_engine_tx:  Option<mpsc::Sender<TbMsg>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
    rpc_pending:     Arc<crate::RpcPendingRegistry>,
) {
    let addr = format!("{}:{}", config.bind, config.port);

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("HTTP transport listening on {}", addr);
            l
        }
        Err(e) => {
            error!("Failed to bind HTTP transport on {}: {}", addr, e);
            return;
        }
    };

    let state = HttpTransportState {
        pool,
        ts_dao,
        rule_engine_tx: Arc::new(rule_engine_tx),
        queue_producer,
        cache,
        ws_tx,
        activity_tx,
        rpc_pending,
    };

    let app = handler::router(state);

    if let Err(e) = axum::serve(listener, app).await {
        error!("HTTP transport server error: {}", e);
    }
}
