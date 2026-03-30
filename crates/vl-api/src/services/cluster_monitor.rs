use std::sync::Arc;
use std::time::Duration;

use vl_cluster::PartitionService;
use vl_cluster::NodeInfo;
use vl_dao::{ClusterNodeDao, ClusterPartitionDao};
use tracing::{error, info, warn};

/// ClusterMonitorService — runs every 15s to maintain cluster health.
///
/// Responsibilities:
/// - Refresh heartbeat for the local node.
/// - Promote nodes from ACTIVE → SUSPECT when heartbeat > 30s stale.
/// - Promote nodes from SUSPECT → DOWN when heartbeat > 60s stale.
/// - **Auto-failover**: when a node is marked DOWN, reassign its partitions
///   to the remaining ACTIVE nodes (only when `partition_dao` is set).
/// - Clean up DOWN nodes whose heartbeat is > 5 minutes stale.
pub struct ClusterMonitorService {
    dao:           Arc<ClusterNodeDao>,
    partition_dao: Option<Arc<ClusterPartitionDao>>,
    partition_svc: Option<Arc<PartitionService>>,
    local_node_id: String,
}

impl ClusterMonitorService {
    /// Minimal constructor (single-node mode, no partition failover).
    pub fn new(dao: Arc<ClusterNodeDao>, local_node_id: String) -> Self {
        Self { dao, partition_dao: None, partition_svc: None, local_node_id }
    }

    /// Full constructor with partition failover support (distributed mode).
    pub fn with_partition_dao(
        dao:           Arc<ClusterNodeDao>,
        partition_dao: Arc<ClusterPartitionDao>,
        partition_svc: Arc<PartitionService>,
        local_node_id: String,
    ) -> Self {
        Self { dao, partition_dao: Some(partition_dao), partition_svc: Some(partition_svc), local_node_id }
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            interval.tick().await; // skip first immediate tick
            loop {
                interval.tick().await;
                if let Err(e) = self.run_cycle().await {
                    error!("ClusterMonitor cycle failed: {}", e);
                }
            }
        })
    }

    async fn run_cycle(&self) -> anyhow::Result<()> {
        // 1. Refresh local node heartbeat.
        if let Err(e) = self.dao.heartbeat(&self.local_node_id).await {
            warn!("Failed to update local node heartbeat: {}", e);
        }

        let now_ms = chrono::Utc::now().timestamp_millis();
        let suspect_threshold = now_ms - 30_000;   // 30s
        let down_threshold    = now_ms - 60_000;   // 60s
        let cleanup_threshold = now_ms - 300_000;  // 5 minutes

        let all_nodes = self.dao.find_all_nodes().await?;
        let mut newly_down: Vec<String> = vec![];

        for node in &all_nodes {
            // Skip the local node — we just refreshed its heartbeat.
            if node.node_id == self.local_node_id {
                continue;
            }

            if node.last_heartbeat < down_threshold && node.status != "DOWN" {
                warn!(
                    node_id = %node.node_id,
                    last_heartbeat = node.last_heartbeat,
                    "Marking cluster node as DOWN (no heartbeat > 60s)"
                );
                if let Err(e) = self.dao.mark_down(&node.node_id).await {
                    error!("Failed to mark node {} DOWN: {}", node.node_id, e);
                } else {
                    newly_down.push(node.node_id.clone());
                }
            } else if node.last_heartbeat < suspect_threshold && node.status == "ACTIVE" {
                warn!(
                    node_id = %node.node_id,
                    last_heartbeat = node.last_heartbeat,
                    "Marking cluster node as SUSPECT (no heartbeat > 30s)"
                );
                if let Err(e) = self.dao.mark_suspect(&node.node_id).await {
                    error!("Failed to mark node {} SUSPECT: {}", node.node_id, e);
                }
            }
        }

        // 2. Auto-failover: reassign partitions from newly-dead nodes.
        if !newly_down.is_empty() {
            if let (Some(part_dao), Some(part_svc)) = (&self.partition_dao, &self.partition_svc) {
                let live_nodes: Vec<String> = all_nodes.iter()
                    .filter(|n| n.status == "ACTIVE" && !newly_down.contains(&n.node_id))
                    .map(|n| n.node_id.clone())
                    .collect();

                for dead_node in &newly_down {
                    match part_dao.failover(dead_node, &live_nodes).await {
                        Ok(count) if count > 0 => {
                            info!(dead_node = %dead_node, reassigned = count, "Partition failover completed");
                        }
                        Ok(_)    => {}
                        Err(e)   => error!("Partition failover for {dead_node} failed: {e}"),
                    }
                }

                // Rebalance the in-memory partition ring to match DB state.
                let live_infos: Vec<NodeInfo> = all_nodes.iter()
                    .filter(|n| n.status == "ACTIVE" && !newly_down.contains(&n.node_id))
                    .map(|n| NodeInfo::new(
                        n.node_id.parse().unwrap_or_default(),
                        &n.host,
                        n.grpc_port as u16,
                    ))
                    .collect();
                part_svc.rebalance(live_infos).await;
            }
        }

        // 3. Clean up DOWN nodes older than 5 minutes.
        match self.dao.cleanup_dead_nodes(cleanup_threshold).await {
            Ok(count) if count > 0 => info!("Cleaned up {} dead cluster nodes", count),
            Err(e)                 => error!("cleanup_dead_nodes failed: {}", e),
            _                      => {}
        }

        Ok(())
    }
}
