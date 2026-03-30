use uuid::Uuid;

/// Trait cho phép rule nodes và các service khác push message xuống Edge gateway.
/// Implemented by `EdgeSessionRegistry` trong `vl-cluster`.
///
/// Dùng fire-and-forget semantics (try_send) để không block async execution.
pub trait EdgeSender: Send + Sync {
    /// Push JSON payload tới một Edge cụ thể (by edge entity UUID).
    fn push_to_edge(&self, edge_id: Uuid, payload: serde_json::Value);

    /// Push JSON payload tới tất cả Edges đang kết nối của một tenant.
    fn push_to_tenant_edges(&self, tenant_id: Uuid, payload: serde_json::Value);
}
