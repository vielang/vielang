use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};
use tracing::{debug, warn};

use vl_cluster::{ClusterMsg, ClusterMsgHandler, ClusterResponse};
use vl_core::entities::TbMsg;

/// Real cluster RPC handler wired into the application layer.
///
/// Handles messages forwarded from other cluster nodes:
/// - `RULE_ENGINE_MSG` — deserialize TbMsg, submit to local rule engine
/// - `WS_PUSH`         — deserialize TbMsg, broadcast to local WebSocket sessions
///
/// Built after AppState so it can hold references to app-level channels.
pub struct TbRpcHandler {
    re_tx:  mpsc::Sender<TbMsg>,
    ws_tx:  broadcast::Sender<TbMsg>,
}

impl TbRpcHandler {
    pub fn new(
        re_tx:  mpsc::Sender<TbMsg>,
        ws_tx:  broadcast::Sender<TbMsg>,
    ) -> Arc<Self> {
        Arc::new(Self { re_tx, ws_tx })
    }
}

#[async_trait::async_trait]
impl ClusterMsgHandler for TbRpcHandler {
    async fn handle(&self, msg: ClusterMsg) -> ClusterResponse {
        match msg.msg_type.as_str() {
            "RULE_ENGINE_MSG" => {
                match serde_json::from_slice::<TbMsg>(&msg.payload) {
                    Ok(tb_msg) => {
                        let _ = self.re_tx.send(tb_msg).await;
                        ClusterResponse { success: true, payload: vec![] }
                    }
                    Err(e) => {
                        warn!("ClusterHandler RULE_ENGINE_MSG deserialize error: {}", e);
                        ClusterResponse { success: false, payload: e.to_string().into_bytes() }
                    }
                }
            }
            "WS_PUSH" => {
                match serde_json::from_slice::<TbMsg>(&msg.payload) {
                    Ok(tb_msg) => {
                        let _ = self.ws_tx.send(tb_msg);
                        ClusterResponse { success: true, payload: vec![] }
                    }
                    Err(e) => {
                        warn!("ClusterHandler WS_PUSH deserialize error: {}", e);
                        ClusterResponse { success: false, payload: e.to_string().into_bytes() }
                    }
                }
            }
            other => {
                debug!("ClusterHandler: unhandled msg_type '{}'", other);
                ClusterResponse { success: true, payload: vec![] }
            }
        }
    }
}
