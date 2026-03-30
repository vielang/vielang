use std::sync::Arc;

use tracing::{debug, warn};
use uuid::Uuid;

use vl_cluster::{ClusterManager, PartitionService};
use vl_core::entities::TbMsg;
use vl_rule_engine::RuleEngine;

use crate::error::ApiError;

/// Routes TbMsg to the correct cluster node based on the entity (device) ID.
///
/// In single-node mode every message is processed locally.
/// In distributed mode messages destined for another node are forwarded
/// over the cluster RPC channel.
#[derive(Clone)]
pub struct ClusterMessageRouter {
    cluster:        Arc<ClusterManager>,
    partition_svc:  Arc<PartitionService>,
    rule_engine:    Arc<RuleEngine>,
    local_node_id:  Uuid,
}

impl ClusterMessageRouter {
    pub fn new(
        cluster:       Arc<ClusterManager>,
        partition_svc: Arc<PartitionService>,
        rule_engine:   Arc<RuleEngine>,
    ) -> Self {
        let local_node_id = cluster.local_node_id();
        Self { cluster, partition_svc, rule_engine, local_node_id }
    }

    /// Route a rule engine message to the responsible node for `entity_id`.
    ///
    /// - If local → process directly in the rule engine.
    /// - If remote → forward via cluster RPC.
    /// - If no nodes available → log warning and process locally (graceful degradation).
    pub async fn route(&self, entity_id: Uuid, msg: TbMsg) -> Result<(), ApiError> {
        if !self.cluster.is_distributed() {
            // Fast path: single-node mode — always local.
            return self.process_local(msg).await;
        }

        let target = self.partition_svc.route(entity_id).await;

        match target {
            Some(node_id) if node_id == self.local_node_id => {
                debug!(entity_id = %entity_id, "Routing msg to local rule engine");
                self.process_local(msg).await
            }
            Some(node_id) => {
                debug!(entity_id = %entity_id, target = %node_id, "Forwarding msg to remote node");
                self.forward_remote(node_id, entity_id, msg).await
            }
            None => {
                warn!(entity_id = %entity_id, "No partition assignment — processing locally");
                self.process_local(msg).await
            }
        }
    }

    async fn process_local(&self, msg: TbMsg) -> Result<(), ApiError> {
        self.rule_engine.send_async(msg).await;
        Ok(())
    }

    async fn forward_remote(
        &self,
        node_id:   Uuid,
        entity_id: Uuid,
        msg:       TbMsg,
    ) -> Result<(), ApiError> {
        let payload = serde_json::to_vec(&msg)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let forwarded = self.cluster
            .forward_to_node(node_id, "RULE_ENGINE_MSG", payload)
            .await
            .map_err(|e| ApiError::Internal(format!("cluster forward: {e}")))?;

        if !forwarded {
            // Node was unreachable — fall back to local processing.
            warn!(
                entity_id = %entity_id,
                target_node = %node_id,
                "Remote node unreachable, processing locally as fallback"
            );
            return self.process_local(msg).await;
        }

        Ok(())
    }
}
