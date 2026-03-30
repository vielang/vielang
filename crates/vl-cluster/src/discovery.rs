use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::ClusterError;
use crate::etcd_http::EtcdClient;
use crate::node::{NodeEvent, NodeInfo};

/// Prefix dùng để lưu node registry trong etcd
const NODE_PREFIX: &str = "/vielang/nodes/";
/// TTL của etcd lease (giây) — node tự động bị xóa nếu heartbeat dừng
const LEASE_TTL_SECS: i64 = 30;
/// Interval để refresh lease
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
/// Interval để poll node list (thay cho watch streaming)
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// etcd-backed service discovery.
///
/// Dùng etcd v3 HTTP/JSON gateway (không cần protoc).
/// Watch được implement bằng poll đơn giản (không dùng streaming gRPC watch).
pub struct ServiceDiscovery {
    client:     EtcdClient,
    local_node: NodeInfo,
}

impl ServiceDiscovery {
    pub async fn new(etcd_url: &str, local_node: NodeInfo) -> Result<Self, ClusterError> {
        let client = EtcdClient::new(etcd_url);
        Ok(Self { client, local_node })
    }

    /// Đăng ký node vào etcd với lease TTL, trả về lease_id.
    pub async fn register(&self) -> Result<i64, ClusterError> {
        let lease_id = self.client.lease_grant(LEASE_TTL_SECS).await?;
        let key   = node_key(self.local_node.node_id);
        let value = serde_json::to_vec(&self.local_node)?;
        self.client.put(&key, &value, Some(lease_id)).await?;
        info!(node_id = %self.local_node.node_id, "Registered in etcd (lease {})", lease_id);
        Ok(lease_id)
    }

    /// Xóa node khỏi etcd (graceful shutdown).
    pub async fn deregister(&self) -> Result<(), ClusterError> {
        let key = node_key(self.local_node.node_id);
        self.client.delete(&key).await?;
        info!(node_id = %self.local_node.node_id, "Deregistered from etcd");
        Ok(())
    }

    /// Spawn background task refresh etcd lease mỗi 10s.
    pub fn start_heartbeat(&self, lease_id: i64) -> tokio::task::JoinHandle<()> {
        let client  = self.client.clone();
        let node_id = self.local_node.node_id;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(HEARTBEAT_INTERVAL).await;
                if let Err(e) = client.lease_keepalive(lease_id).await {
                    warn!(node_id = %node_id, "Lease keep-alive failed: {}", e);
                }
            }
        })
    }

    /// Liệt kê tất cả nodes hiện tại trong cluster.
    pub async fn list_nodes(&self) -> Result<Vec<NodeInfo>, ClusterError> {
        let kvs = self.client.get_prefix(NODE_PREFIX).await?;
        let mut nodes = Vec::new();
        for (_key, value) in kvs {
            match serde_json::from_slice::<NodeInfo>(&value) {
                Ok(node) => nodes.push(node),
                Err(e)   => warn!("Failed to deserialize node info: {}", e),
            }
        }
        Ok(nodes)
    }

    /// Spawn background task poll cluster nodes → phát NodeEvent qua tx khi có thay đổi.
    ///
    /// Dùng polling thay vì streaming watch để tránh phụ thuộc vào gRPC streaming.
    pub fn watch_nodes(&self, tx: mpsc::Sender<NodeEvent>) -> tokio::task::JoinHandle<()> {
        let client     = self.client.clone();
        let local_id   = self.local_node.node_id;

        tokio::spawn(async move {
            let mut known: std::collections::HashMap<Uuid, NodeInfo> = std::collections::HashMap::new();

            loop {
                tokio::time::sleep(POLL_INTERVAL).await;

                let current = match client.get_prefix(NODE_PREFIX).await {
                    Ok(kvs) => {
                        let mut map = std::collections::HashMap::new();
                        for (_k, v) in kvs {
                            if let Ok(node) = serde_json::from_slice::<NodeInfo>(&v) {
                                map.insert(node.node_id, node);
                            }
                        }
                        map
                    }
                    Err(e) => {
                        warn!("Failed to poll etcd nodes: {}", e);
                        continue;
                    }
                };

                // Detect joins
                for (id, node) in &current {
                    if !known.contains_key(id) && *id != local_id {
                        info!(node_id = %id, "Node joined cluster");
                        let _ = tx.send(NodeEvent::Joined(node.clone())).await;
                    }
                }
                // Detect leaves
                for (id, _) in &known {
                    if !current.contains_key(id) {
                        info!(node_id = %id, "Node left cluster");
                        let _ = tx.send(NodeEvent::Left(*id)).await;
                    }
                }

                known = current;
            }
        })
    }
}

fn node_key(node_id: Uuid) -> String {
    format!("{}{}", NODE_PREFIX, node_id)
}
