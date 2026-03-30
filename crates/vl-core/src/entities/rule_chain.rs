use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RuleNode — a single node in a rule chain.
/// Java: org.thingsboard.server.common.data.rule.RuleNode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleNode {
    pub id: Option<Uuid>,
    pub created_time: Option<i64>,
    pub rule_chain_id: Option<Uuid>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub configuration: Option<serde_json::Value>,
    pub additional_info: Option<serde_json::Value>,
    pub debug_mode: bool,
    pub singleton_mode: bool,
    pub queue_name: Option<String>,
    pub version: Option<i64>,
}

/// Connection between two nodes in a rule chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConnectionInfo {
    pub from_index: i32,
    pub to_index: i32,
    #[serde(rename = "type")]
    pub type_: String,
}

/// Connection from a rule node to another rule chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleChainConnectionInfo {
    pub from_index: i32,
    pub target_rule_chain_id: Uuid,
    #[serde(rename = "type")]
    pub type_: String,
}

/// Full metadata for a rule chain: nodes + connections.
/// Java: org.thingsboard.server.common.data.rule.RuleChainMetaData
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleChainMetaData {
    pub rule_chain_id: Uuid,
    pub first_node_index: Option<i32>,
    pub nodes: Vec<RuleNode>,
    pub connections: Vec<NodeConnectionInfo>,
    pub rule_chain_connections: Option<Vec<RuleChainConnectionInfo>>,
}

/// Khớp Java: RuleChain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleChain {
    pub id:                 Uuid,
    pub created_time:       i64,
    pub tenant_id:          Uuid,
    pub name:               String,
    pub chain_type:         String,   // 'CORE' | 'EDGE'
    pub first_rule_node_id: Option<Uuid>,
    pub root:               bool,
    pub debug_mode:         bool,
    pub configuration:      Option<String>,
    pub additional_info:    Option<String>,
    pub external_id:        Option<Uuid>,
    pub version:            i64,
}
