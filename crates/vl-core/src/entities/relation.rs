use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::alarm::EntityType;

/// Khớp với bảng `relation`.
/// Java: org.thingsboard.server.common.data.relation.EntityRelation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelation {
    pub from_id: Uuid,
    pub from_type: EntityType,
    pub to_id: Uuid,
    pub to_type: EntityType,
    pub relation_type: String,
    pub relation_type_group: RelationTypeGroup,
    pub additional_info: Option<serde_json::Value>,
}

/// Khớp Java: RelationTypeGroup
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationTypeGroup {
    Common,
    Alarm,
    DashboardLink,
    RuleChain,
    RuleNode,
    EdgeAutoAssignDefault,
}

/// Built-in relation types — Java: EntityRelation constants
pub mod relation_types {
    pub const CONTAINS: &str = "Contains";
    pub const MANAGES: &str = "Manages";
}
