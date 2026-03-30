use serde::{Deserialize, Serialize};

/// Represents a node in the VieLang cluster.
/// Maps to the `cluster_node` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterNode {
    pub node_id:        String,
    pub host:           String,
    pub port:           i32,
    pub status:         String,
    pub service_type:   String,
    pub last_heartbeat: i64,
    pub joined_at:      i64,
    pub metadata:       serde_json::Value,
    // P15: leader election fields
    pub is_leader:      bool,
    pub grpc_port:      i32,
    pub leader_epoch:   i64,
}

/// Cluster topology as seen by the local node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterTopology {
    pub local_node:          ClusterNode,
    pub nodes:               Vec<ClusterNode>,
    pub total_partitions:    i32,
    pub assigned_partitions: Vec<i32>,
}

/// Maps partition_id → responsible node_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterPartition {
    pub partition_id: i32,
    pub node_id:      Option<String>,
    pub assigned_at:  i64,
}

/// Leader info returned by GET /api/cluster/leader.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterLeaderInfo {
    pub leader_node_id: Option<String>,
    pub host:           Option<String>,
    pub grpc_port:      Option<i32>,
    pub leader_epoch:   i64,
    pub is_local:       bool,
}
