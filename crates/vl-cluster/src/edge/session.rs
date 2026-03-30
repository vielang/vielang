use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// Trạng thái của một Edge session đang kết nối.
/// Mỗi session tương ứng với một Edge instance đang duy trì gRPC bidirectional stream.
#[derive(Clone)]
pub struct EdgeSession {
    pub edge_id:    Uuid,
    pub tenant_id:  Uuid,
    /// Channel để push DownlinkMsg (JSON) xuống Edge gRPC stream.
    /// Receiver ở gRPC handler task convert JSON → DownlinkMsg protobuf.
    pub downlink_tx: mpsc::Sender<serde_json::Value>,
    pub connected_at: Instant,
    pub last_seen:   Arc<Mutex<Instant>>,
}

impl EdgeSession {
    pub fn new(
        edge_id:    Uuid,
        tenant_id:  Uuid,
        downlink_tx: mpsc::Sender<serde_json::Value>,
    ) -> Self {
        let now = Instant::now();
        Self {
            edge_id,
            tenant_id,
            downlink_tx,
            connected_at: now,
            last_seen: Arc::new(Mutex::new(now)),
        }
    }
}
