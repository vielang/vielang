use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `edge`.
/// Java: org.thingsboard.server.common.data.edge.Edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub root_rule_chain_id: Option<Uuid>,
    pub name: String,
    pub edge_type: String,
    pub label: Option<String>,
    pub routing_key: String,
    pub secret: String,
    pub additional_info: Option<serde_json::Value>,
    pub external_id: Option<Uuid>,
    pub version: i64,
}

/// Lightweight info — Edge + customer title/public.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    #[serde(flatten)]
    pub edge: Edge,
    pub customer_title: Option<String>,
    pub customer_is_public: bool,
}

/// Khớp với bảng `edge_event`.
/// Java: org.thingsboard.server.common.data.edge.EdgeEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeEvent {
    pub id: Uuid,
    pub created_time: i64,
    pub seq_id: i64,
    pub tenant_id: Uuid,
    pub edge_id: Uuid,
    /// DASHBOARD | DEVICE | ALARM | RULE_CHAIN | etc.
    pub edge_event_type: String,
    /// ADDED | DELETED | UPDATED | etc.
    pub edge_event_action: String,
    pub entity_id: Option<Uuid>,
    pub body: Option<serde_json::Value>,
    pub uid: Option<String>,
}
