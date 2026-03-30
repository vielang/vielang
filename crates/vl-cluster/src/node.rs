use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Thông tin một cluster node — được lưu trong etcd và trao đổi giữa các node.
/// Khớp ThingsBoard Java: ServerAddress + NodeInfo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    /// Unique node identifier (UUID v4, auto-generated at startup)
    pub node_id:  Uuid,
    /// Host/IP mà các node khác dùng để connect RPC
    pub rpc_host: String,
    /// Port cho node-to-node RPC
    pub rpc_port: u16,
}

impl NodeInfo {
    pub fn new(node_id: Uuid, rpc_host: impl Into<String>, rpc_port: u16) -> Self {
        Self { node_id, rpc_host: rpc_host.into(), rpc_port }
    }

    /// Địa chỉ RPC dưới dạng "host:port"
    pub fn rpc_addr(&self) -> String {
        format!("{}:{}", self.rpc_host, self.rpc_port)
    }
}

/// Sự kiện node join/leave — phát ra bởi ServiceDiscovery::watch_nodes()
#[derive(Debug, Clone)]
pub enum NodeEvent {
    Joined(NodeInfo),
    Left(Uuid),
}
