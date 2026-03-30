use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ComponentDescriptor — mirrors ThingsBoard Java ComponentDescriptor.
/// Represents a rule node component that can be used in rule chains.
/// In ThingsBoard Java these are discovered at runtime from classpath annotations
/// and persisted to the component_descriptor table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDescriptor {
    pub id: Uuid,
    pub created_time: i64,

    /// Component type (e.g. ENRICHMENT, FILTER, TRANSFORMATION, ACTION, EXTERNAL)
    #[serde(rename = "type")]
    pub type_: Option<String>,

    /// Scope: TENANT or SYSTEM
    pub scope: Option<String>,

    /// Clustering mode: USER_PREFERENCE, ENABLED, or SINGLETON
    pub clustering_mode: Option<String>,

    /// Human-readable name of the rule node
    pub name: Option<String>,

    /// Fully-qualified Java class name
    pub clazz: Option<String>,

    /// JSON configuration schema descriptor
    pub configuration_descriptor: Option<serde_json::Value>,

    /// Configuration schema version
    pub configuration_version: Option<i32>,

    /// Deprecated — always null
    pub actions: Option<String>,

    /// Whether the node supports queue name configuration
    pub has_queue_name: Option<bool>,
}
