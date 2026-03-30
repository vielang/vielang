use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use uuid::Uuid;
use tracing::{info, warn};
use tokio::task::JoinHandle;

use vl_core::entities::EdgeSender;
use crate::edge::session::EdgeSession;

/// Registry các Edge session đang connected.
/// Thread-safe: wraps DashMap for concurrent read/write.
/// Implements `EdgeSender` để rule nodes và services có thể push data xuống Edge.
pub struct EdgeSessionRegistry {
    sessions: DashMap<Uuid, EdgeSession>,
}

impl EdgeSessionRegistry {
    pub fn new() -> Self {
        Self { sessions: DashMap::new() }
    }

    /// Đăng ký một Edge session mới (gọi khi Edge kết nối).
    pub fn register(&self, session: EdgeSession) {
        self.sessions.insert(session.edge_id, session);
    }

    /// Xoá Edge session (gọi khi Edge ngắt kết nối).
    pub fn remove(&self, edge_id: Uuid) {
        self.sessions.remove(&edge_id);
    }

    /// Số lượng Edge đang kết nối.
    pub fn connected_count(&self) -> usize {
        self.sessions.len()
    }

    /// List tất cả Edge IDs đang connected.
    pub fn connected_edge_ids(&self) -> Vec<Uuid> {
        self.sessions.iter().map(|s| s.edge_id).collect()
    }

    /// Kiểm tra một Edge có đang connected không.
    pub fn is_connected(&self, edge_id: Uuid) -> bool {
        self.sessions.contains_key(&edge_id)
    }

    /// Get a cloned session (để update last_seen, v.v.)
    pub fn get_session(&self, edge_id: Uuid) -> Option<EdgeSession> {
        self.sessions.get(&edge_id).map(|s| s.clone())
    }

    /// Push JSON payload trực tiếp (không qua EdgeSender trait — dùng nội bộ trong vl-cluster).
    pub fn push_to_edge_raw(&self, edge_id: Uuid, payload: serde_json::Value) {
        self.push_to_edge(edge_id, payload);
    }

    /// Start background task that removes stale sessions (no activity for `timeout`).
    /// Returns JoinHandle for graceful shutdown.
    pub fn start_session_cleanup(self: &Arc<Self>, timeout: Duration) -> JoinHandle<()> {
        let registry = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            interval.tick().await; // skip immediate tick
            loop {
                interval.tick().await;
                let mut stale = Vec::new();
                for entry in registry.sessions.iter() {
                    let last = *entry.last_seen.lock().await;
                    if last.elapsed() > timeout {
                        stale.push(entry.edge_id);
                    }
                }
                for edge_id in stale {
                    registry.remove(edge_id);
                    info!(edge_id = %edge_id, "Removed stale edge session (timeout)");
                }
            }
        })
    }
}

impl EdgeSender for EdgeSessionRegistry {
    fn push_to_edge(&self, edge_id: Uuid, payload: serde_json::Value) {
        if let Some(session) = self.sessions.get(&edge_id) {
            if let Err(e) = session.downlink_tx.try_send(payload) {
                warn!(edge_id = %edge_id, "Failed to push downlink to edge: {}", e);
            }
        }
    }

    fn push_to_tenant_edges(&self, tenant_id: Uuid, payload: serde_json::Value) {
        // Collect senders first to avoid holding DashMap ref across await
        let senders: Vec<_> = self.sessions
            .iter()
            .filter(|s| s.tenant_id == tenant_id)
            .map(|s| s.downlink_tx.clone())
            .collect();

        for tx in senders {
            if let Err(e) = tx.try_send(payload.clone()) {
                warn!(tenant_id = %tenant_id, "Failed to push downlink to edge: {}", e);
            }
        }
    }
}

/// Trait dành cho đối tượng cần sync xuống Edge (device, rule chain, dashboard, ...).
pub trait EdgeSyncable: Send + Sync {
    /// Tên entity type, khớp với EdgeEntityType enum trong proto.
    fn edge_entity_type() -> &'static str;
    /// Serialize self thành JSON body để đưa vào DownlinkMsg.
    fn to_edge_body(&self) -> serde_json::Value;
    /// Entity UUID.
    fn entity_id(&self) -> Uuid;
    /// Tenant UUID.
    fn tenant_id(&self) -> Uuid;
}

/// Helper: tạo downlink payload chuẩn để push qua EdgeSender.
/// Format: `{ "entityType": "DEVICE", "entityId": "...", "tenantId": "...", "body": {...} }`
pub fn make_entity_update_payload<T: EdgeSyncable>(entity: &T) -> serde_json::Value {
    serde_json::json!({
        "entityType": T::edge_entity_type(),
        "entityId":   entity.entity_id().to_string(),
        "tenantId":   entity.tenant_id().to_string(),
        "body":       entity.to_edge_body(),
    })
}

impl<T: EdgeSyncable + 'static> EdgeSyncable for Arc<T> {
    fn edge_entity_type() -> &'static str { T::edge_entity_type() }
    fn to_edge_body(&self)  -> serde_json::Value { self.as_ref().to_edge_body() }
    fn entity_id(&self)     -> Uuid { self.as_ref().entity_id() }
    fn tenant_id(&self)     -> Uuid { self.as_ref().tenant_id() }
}
