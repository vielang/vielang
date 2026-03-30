use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use vl_config::ClusterConfig;

use crate::discovery::ServiceDiscovery;
use crate::error::ClusterError;
use crate::node::{NodeEvent, NodeInfo};
use crate::partitioner::RendezvousHasher;
use crate::rpc::{ClusterMsgHandler, ClusterRpcClient, start_rpc_server};

/// ClusterManager — điểm trung tâm của cluster subsystem.
///
/// - Single-node mode (`config.enabled = false`): không kết nối etcd, không gRPC.
///   `is_local()` luôn trả về true, `get_responsible_node()` trả về local node.
/// - Distributed mode (`config.enabled = true`): register etcd, heartbeat, watch peers,
///   dùng rendezvous hashing để route entities.
#[derive(Clone)]
pub struct ClusterManager {
    pub local_node: NodeInfo,
    nodes:          Arc<RwLock<Vec<NodeInfo>>>,
    rpc_client:     Arc<ClusterRpcClient>,
    distributed:    bool,
    /// Bind address for the RPC server — set in distributed mode, None otherwise.
    rpc_bind_addr:  Option<String>,
}

impl ClusterManager {
    /// Khởi tạo ClusterManager từ config.
    /// Nếu `config.enabled = false` → single-node mode, không connect etcd.
    pub async fn new(config: &ClusterConfig) -> Result<Self, ClusterError> {
        let node_id = if config.node_id.is_empty() {
            Uuid::new_v4()
        } else {
            config.node_id.parse().unwrap_or_else(|_| Uuid::new_v4())
        };

        let local_node = NodeInfo::new(node_id, &config.rpc_host, config.rpc_port);
        let rpc_client = Arc::new(ClusterRpcClient::new(node_id));
        let nodes      = Arc::new(RwLock::new(vec![local_node.clone()]));

        if !config.distributed() {
            info!(node_id = %node_id, "Cluster in single-node mode");
            return Ok(Self { local_node, nodes, rpc_client, distributed: false, rpc_bind_addr: None });
        }

        // ── Distributed mode ──────────────────────────────────────────────────
        info!(node_id = %node_id, "Cluster distributed mode — connecting to {}", config.etcd_url);

        let discovery = ServiceDiscovery::new(&config.etcd_url, local_node.clone()).await?;

        // Register this node and get a lease
        let lease_id = discovery.register().await?;

        // Seed the node list with existing peers
        let existing = discovery.list_nodes().await?;
        {
            let mut guard = nodes.write().await;
            *guard = existing;
            // Ensure local node is always present
            if !guard.iter().any(|n| n.node_id == node_id) {
                guard.push(local_node.clone());
            }
        }

        // Heartbeat task
        discovery.start_heartbeat(lease_id);

        // Watch task — update nodes list when peers join/leave
        let (event_tx, mut event_rx) = mpsc::channel::<NodeEvent>(64);
        discovery.watch_nodes(event_tx);

        let nodes_for_watcher = nodes.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    NodeEvent::Joined(node) => {
                        let mut guard = nodes_for_watcher.write().await;
                        if !guard.iter().any(|n| n.node_id == node.node_id) {
                            info!(node_id = %node.node_id, "Peer joined cluster");
                            guard.push(node);
                        }
                    }
                    NodeEvent::Left(nid) => {
                        let mut guard = nodes_for_watcher.write().await;
                        guard.retain(|n| n.node_id != nid);
                        warn!(node_id = %nid, "Peer left cluster");
                    }
                }
            }
        });

        // RPC server is started lazily via start_rpc_with_handler() so that
        // the caller (vl-api) can inject an app-level handler without circular deps.
        let bind = format!("{}:{}", config.rpc_host, config.rpc_port);

        Ok(Self { local_node, nodes, rpc_client, distributed: true, rpc_bind_addr: Some(bind) })
    }

    /// Node ID của node hiện tại.
    pub fn local_node_id(&self) -> Uuid {
        self.local_node.node_id
    }

    /// Trả về node chịu trách nhiệm cho entity_id.
    /// Trong single-node mode luôn trả về local node.
    pub async fn get_responsible_node(&self, entity_id: Uuid) -> NodeInfo {
        if !self.distributed {
            return self.local_node.clone();
        }
        let guard = self.nodes.read().await;
        RendezvousHasher::get_node(entity_id, &guard)
            .cloned()
            .unwrap_or_else(|| self.local_node.clone())
    }

    /// Kiểm tra node hiện tại có phải responsible cho entity_id không.
    /// Trong single-node mode luôn true.
    pub async fn is_local(&self, entity_id: Uuid) -> bool {
        if !self.distributed {
            return true;
        }
        let responsible = self.get_responsible_node(entity_id).await;
        responsible.node_id == self.local_node.node_id
    }

    /// Số lượng nodes trong cluster (bao gồm local).
    pub async fn node_count(&self) -> usize {
        self.nodes.read().await.len()
    }

    /// Danh sách tất cả nodes.
    pub async fn all_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.read().await.clone()
    }

    /// Start the cluster RPC server with a custom message handler.
    /// Must be called in distributed mode after the app-level handler is built.
    /// In single-node mode this is a no-op.
    pub fn start_rpc_with_handler(&self, handler: Arc<dyn ClusterMsgHandler>) {
        if let Some(ref bind) = self.rpc_bind_addr {
            info!("Starting cluster RPC server on {}", bind);
            start_rpc_server(bind.clone(), handler);
        }
    }

    /// True nếu cluster đang chạy ở distributed mode.
    pub fn is_distributed(&self) -> bool {
        self.distributed
    }

    /// Forward message tới một node cụ thể theo node_id.
    /// Trả về false nếu node không tìm thấy (caller fallback to local).
    pub async fn forward_to_node(
        &self,
        target_node_id: Uuid,
        msg_type:       &str,
        payload:        Vec<u8>,
    ) -> Result<bool, ClusterError> {
        let guard = self.nodes.read().await;
        let target = guard.iter().find(|n| n.node_id == target_node_id).cloned();
        drop(guard);
        match target {
            Some(node) => {
                self.rpc_client.send_msg(&node, msg_type, payload).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Forward một message tới node chịu trách nhiệm cho entity.
    /// Trong single-node mode: không-op (xử lý local).
    pub async fn forward_to_responsible(
        &self,
        entity_id: Uuid,
        msg_type: &str,
        payload: Vec<u8>,
    ) -> Result<bool, ClusterError> {
        let responsible = self.get_responsible_node(entity_id).await;
        if responsible.node_id == self.local_node.node_id {
            // Process locally
            return Ok(false);
        }
        self.rpc_client
            .send_msg(&responsible, msg_type, payload)
            .await?;
        Ok(true) // forwarded
    }
}

// Extension trait cho ClusterConfig để tránh circular dependency
trait ClusterConfigExt {
    fn distributed(&self) -> bool;
}

impl ClusterConfigExt for vl_config::ClusterConfig {
    fn distributed(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vl_config::ClusterConfig;

    #[tokio::test]
    async fn single_node_mode_always_local() {
        let config = ClusterConfig::default(); // enabled = false
        let manager = ClusterManager::new(&config).await.unwrap();
        let entity_id = Uuid::new_v4();
        assert!(manager.is_local(entity_id).await);
        assert_eq!(manager.node_count().await, 1);
    }

    #[tokio::test]
    async fn local_node_id_matches_config_when_set() {
        let fixed_id = Uuid::from_u128(0x1234);
        let config = ClusterConfig {
            enabled:             false,
            node_id:             fixed_id.to_string(),
            etcd_url:            "http://localhost:2379".into(),
            rpc_host:            "localhost".into(),
            rpc_port:            9090,
            num_partitions:      12,
            election_timeout_ms: 10_000,
        };
        let manager = ClusterManager::new(&config).await.unwrap();
        assert_eq!(manager.local_node_id(), fixed_id);
    }

    #[tokio::test]
    async fn auto_generates_node_id_when_empty() {
        let config = ClusterConfig::default();
        let manager = ClusterManager::new(&config).await.unwrap();
        // Should have a valid (non-nil) UUID
        assert_ne!(manager.local_node_id(), Uuid::nil());
    }

    #[tokio::test]
    async fn get_responsible_node_single_mode_is_local() {
        let config = ClusterManager::new(&ClusterConfig::default()).await.unwrap();
        let node = config.get_responsible_node(Uuid::new_v4()).await;
        assert_eq!(node.node_id, config.local_node_id());
    }
}
