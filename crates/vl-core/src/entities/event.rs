use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event type for lifecycle and debug events
/// Java: org.thingsboard.server.common.data.event.EventType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    /// Entity lifecycle events (created, updated, deleted)
    LcEvent,
    /// Statistics events
    Stats,
    /// Rule node debug events
    DebugRuleNode,
    /// Rule chain debug events
    DebugRuleChain,
    /// Error events
    Error,
    /// Calculated field debug events
    DebugCf,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::LcEvent => "LC_EVENT",
            EventType::Stats => "STATS",
            EventType::DebugRuleNode => "DEBUG_RULE_NODE",
            EventType::DebugRuleChain => "DEBUG_RULE_CHAIN",
            EventType::Error => "ERROR",
            EventType::DebugCf => "DEBUG_CF",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "LC_EVENT" => Some(EventType::LcEvent),
            "STATS" => Some(EventType::Stats),
            "DEBUG_RULE_NODE" => Some(EventType::DebugRuleNode),
            "DEBUG_RULE_CHAIN" => Some(EventType::DebugRuleChain),
            "ERROR" => Some(EventType::Error),
            "DEBUG_CF" => Some(EventType::DebugCf),
            _ => None,
        }
    }
}

/// Event — lifecycle, statistics, or debug events
/// Java: org.thingsboard.server.common.data.event.Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,

    /// Entity this event is associated with
    pub entity_id: Uuid,
    pub entity_type: String,

    /// Event type
    pub event_type: EventType,

    /// Event uid for deduplication
    pub event_uid: String,

    /// Event body (JSON)
    pub body: serde_json::Value,
}

/// Lifecycle event body structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEventBody {
    /// Event name: "created", "updated", "deleted"
    pub event: String,
    /// Whether the event was successful
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
}

/// Error event body structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEventBody {
    /// Method where error occurred
    pub method: String,
    /// Error message
    pub error: String,
}

/// Statistics event body structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEventBody {
    /// Messages processed
    pub messages_processed: i64,
    /// Errors count
    pub errors_occurred: i64,
}

/// Debug rule node event body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRuleNodeEventBody {
    /// Rule chain id
    pub rule_chain_id: Uuid,
    /// Rule node id
    pub rule_node_id: Uuid,
    /// Message type
    pub msg_type: String,
    /// Relation type (Success, Failure, etc.)
    pub relation_type: String,
    /// Data in JSON format
    pub data: String,
    /// Metadata in JSON format
    pub metadata: String,
    /// Error if any
    pub error: Option<String>,
}

/// Calculated field debug event body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfDebugEventBody {
    pub cf_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub msg_id: Option<Uuid>,
    pub msg_type: Option<String>,
    pub args: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// Event filter for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub event_type: Option<EventType>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
}
