//! Actor framework for VíeLang, modeled after ThingsBoard's actor system.
//!
//! # Architecture
//!
//! - **`TbActorSystem`** — global registry managing all actors, dispatchers,
//!   parent-child relationships, and message routing.
//! - **`TbActorMailbox`** — per-actor mailbox with dual-priority queues
//!   (high & normal), throughput batching, and atomic busy/ready flags.
//! - **`TbActor`** trait — implement for domain actors (AppActor, TenantActor, etc.).
//! - **`TbActorCtx`** — context provided to actors for child management,
//!   broadcasting, and self-reference.
//!
//! # Actor Hierarchy (ThingsBoard-compatible)
//!
//! ```text
//! AppActor
//!   └── TenantActor (per tenant)
//!       ├── RuleChainActor (per rule chain)
//!       │   └── RuleNodeActor (per rule node)
//!       ├── DeviceActor (per device, lazy)
//!       └── CalculatedFieldManagerActor
//!           └── CalculatedFieldEntityActor
//! ```

mod actor_id;
mod actor_ref;
mod actor_system;
mod error;
mod mailbox;
mod strategy;

pub use actor_id::*;
pub use actor_ref::*;
pub use actor_system::*;
pub use error::*;
pub use mailbox::MsgPriority;
pub use strategy::*;

use async_trait::async_trait;

/// Core actor trait — implement for each domain actor.
///
/// Mirrors ThingsBoard's `TbActor` interface:
/// - `process()` handles a single message, returning `true` if handled.
/// - `init()` is called once before message processing begins.
/// - `destroy()` is called when the actor is stopping.
/// - `on_init_failure()` / `on_process_failure()` control supervision behavior.
#[async_trait]
pub trait TbActor: Send + 'static {
    /// Process a single message. Return `true` if handled.
    async fn process(&mut self, msg: ActorMsg) -> bool;

    /// Called once when the actor is started (before message processing).
    async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError>;

    /// Called when the actor is stopping.
    async fn destroy(&mut self, reason: StopReason);

    /// Supervision strategy when `init()` fails.
    fn on_init_failure(&self, attempt: u32, error: &ActorError) -> InitFailureStrategy {
        let _ = error;
        InitFailureStrategy::RetryWithDelay {
            delay_ms: 5000 * attempt as u64,
        }
    }

    /// Supervision strategy when `process()` panics or returns error.
    fn on_process_failure(&self, _msg: &ActorMsg, is_fatal: bool) -> ProcessFailureStrategy {
        if is_fatal {
            ProcessFailureStrategy::Stop
        } else {
            ProcessFailureStrategy::Resume
        }
    }
}

/// Reason an actor was stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// Normal shutdown (parent stopped, system shutdown).
    Normal,
    /// Actor init failed after max retries.
    InitFailed,
    /// Actor processing encountered a fatal error.
    ProcessingError,
}

/// An actor message. Uses an enum to avoid type-erasure overhead.
///
/// Each variant corresponds to a ThingsBoard `MsgType`.
#[derive(Debug, Clone)]
pub enum ActorMsg {
    // ── Application lifecycle ──────────────────────────────
    AppInit,
    PartitionChange {
        service_type: String,
    },
    SessionTimeout,

    // ── Component lifecycle ────────────────────────────────
    ComponentLifecycle {
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
        entity_type: String,
        event: LifecycleEvent,
    },

    // ── Queue → Rule Engine ────────────────────────────────
    QueueToRuleEngine {
        tenant_id: uuid::Uuid,
        rule_chain_id: Option<uuid::Uuid>,
        msg: Box<RuleEngineMsg>,
    },

    // ── Rule chain routing ─────────────────────────────────
    RuleChainToRuleNode {
        rule_node_id: uuid::Uuid,
        from_relation_type: String,
        msg: Box<RuleEngineMsg>,
    },
    RuleNodeToRuleChainTellNext {
        rule_chain_id: uuid::Uuid,
        originator_node_id: uuid::Uuid,
        relation_types: Vec<String>,
        msg: Box<RuleEngineMsg>,
        failure_message: Option<String>,
    },
    RuleChainToRuleChain {
        target_chain_id: uuid::Uuid,
        source_chain_id: uuid::Uuid,
        from_relation_type: String,
        msg: Box<RuleEngineMsg>,
    },
    RuleChainInput {
        target_chain_id: uuid::Uuid,
        msg: Box<RuleEngineMsg>,
    },
    RuleChainOutput {
        target_chain_id: uuid::Uuid,
        target_node_id: uuid::Uuid,
        relation_type: String,
        msg: Box<RuleEngineMsg>,
    },
    RuleNodeUpdated {
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
    },
    RuleToSelf {
        msg: Box<RuleEngineMsg>,
    },

    // ── Transport → Device ─────────────────────────────────
    TransportToDevice {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        payload: Vec<u8>,
    },

    // ── Device notifications ───────────────────────────────
    DeviceAttributesUpdate {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        scope: String,
        deleted: bool,
    },
    DeviceCredentialsUpdate {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
    },
    DeviceNameOrTypeUpdate {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        device_name: String,
        device_type: String,
    },
    DeviceDelete {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
    },
    DeviceEdgeUpdate {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        edge_id: Option<uuid::Uuid>,
    },

    // ── Device RPC ─────────────────────────────────────────
    DeviceRpcRequest {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        request_id: uuid::Uuid,
        oneway: bool,
        body: String,
        expiration_time: i64,
        persisted: bool,
    },
    DeviceRpcResponse {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        request_id: i32,
        response: Option<String>,
        error: Option<String>,
    },
    DeviceRpcTimeout {
        rpc_id: i32,
        timeout_ms: u64,
    },
    RemoveRpc {
        tenant_id: uuid::Uuid,
        device_id: uuid::Uuid,
        request_id: uuid::Uuid,
    },

    // ── Calculated Fields ──────────────────────────────────
    CfCacheInit {
        tenant_id: uuid::Uuid,
    },
    CfStateRestore {
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
    },
    CfPartitionsChange {
        tenant_id: uuid::Uuid,
    },
    CfEntityLifecycle {
        tenant_id: uuid::Uuid,
        event: LifecycleEvent,
    },
    CfTelemetry {
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
        payload: Vec<u8>,
    },
    CfLinkedTelemetry {
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
        payload: Vec<u8>,
    },
    CfEntityAction {
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
        action: String,
    },

    // ── Edge ───────────────────────────────────────────────
    EdgeEventUpdate {
        tenant_id: uuid::Uuid,
        edge_id: uuid::Uuid,
    },
    EdgeHighPriority {
        tenant_id: uuid::Uuid,
        edge_id: uuid::Uuid,
        payload: Vec<u8>,
    },
    EdgeSyncRequest {
        tenant_id: uuid::Uuid,
        edge_id: uuid::Uuid,
        request_id: uuid::Uuid,
    },

    // ── Statistics ──────────────────────────────────────────
    StatsPersistTick,
    StatsPersist {
        messages_processed: u64,
        errors_occurred: u64,
        tenant_id: uuid::Uuid,
        entity_id: uuid::Uuid,
    },
}

/// Lifecycle event for components (rule chains, devices, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    Created,
    Updated,
    Activated,
    Suspended,
    Deleted,
}

/// Payload carried through the rule engine.
#[derive(Debug, Clone)]
pub struct RuleEngineMsg {
    /// Unique message ID.
    pub id: uuid::Uuid,
    /// Originator entity type.
    pub originator_type: String,
    /// Originator entity ID.
    pub originator_id: uuid::Uuid,
    /// Message type (e.g., "POST_TELEMETRY_REQUEST").
    pub msg_type: String,
    /// JSON data payload.
    pub data: String,
    /// JSON metadata.
    pub metadata: String,
    /// Rule chain ID currently processing this message.
    pub rule_chain_id: Option<uuid::Uuid>,
    /// Rule node ID currently processing this message.
    pub rule_node_id: Option<uuid::Uuid>,
}

impl ActorMsg {
    /// Extract tenant_id if the message is tenant-aware.
    pub fn tenant_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::QueueToRuleEngine { tenant_id, .. }
            | Self::ComponentLifecycle { tenant_id, .. }
            | Self::TransportToDevice { tenant_id, .. }
            | Self::DeviceAttributesUpdate { tenant_id, .. }
            | Self::DeviceCredentialsUpdate { tenant_id, .. }
            | Self::DeviceNameOrTypeUpdate { tenant_id, .. }
            | Self::DeviceDelete { tenant_id, .. }
            | Self::DeviceEdgeUpdate { tenant_id, .. }
            | Self::DeviceRpcRequest { tenant_id, .. }
            | Self::DeviceRpcResponse { tenant_id, .. }
            | Self::RemoveRpc { tenant_id, .. }
            | Self::CfCacheInit { tenant_id, .. }
            | Self::CfStateRestore { tenant_id, .. }
            | Self::CfPartitionsChange { tenant_id, .. }
            | Self::CfEntityLifecycle { tenant_id, .. }
            | Self::CfTelemetry { tenant_id, .. }
            | Self::CfLinkedTelemetry { tenant_id, .. }
            | Self::CfEntityAction { tenant_id, .. }
            | Self::EdgeEventUpdate { tenant_id, .. }
            | Self::EdgeHighPriority { tenant_id, .. }
            | Self::EdgeSyncRequest { tenant_id, .. }
            | Self::RuleNodeUpdated { tenant_id, .. }
            | Self::StatsPersist { tenant_id, .. } => Some(*tenant_id),
            _ => None,
        }
    }

    /// Extract device_id if the message is device-aware.
    pub fn device_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::TransportToDevice { device_id, .. }
            | Self::DeviceAttributesUpdate { device_id, .. }
            | Self::DeviceCredentialsUpdate { device_id, .. }
            | Self::DeviceNameOrTypeUpdate { device_id, .. }
            | Self::DeviceDelete { device_id, .. }
            | Self::DeviceEdgeUpdate { device_id, .. }
            | Self::DeviceRpcRequest { device_id, .. }
            | Self::DeviceRpcResponse { device_id, .. }
            | Self::RemoveRpc { device_id, .. } => Some(*device_id),
            _ => None,
        }
    }

    /// Whether this message should be high-priority in the mailbox.
    pub fn is_high_priority(&self) -> bool {
        matches!(
            self,
            Self::PartitionChange { .. }
                | Self::ComponentLifecycle { .. }
                | Self::DeviceAttributesUpdate { .. }
                | Self::DeviceCredentialsUpdate { .. }
                | Self::DeviceNameOrTypeUpdate { .. }
                | Self::DeviceDelete { .. }
                | Self::DeviceEdgeUpdate { .. }
                | Self::DeviceRpcRequest { .. }
                | Self::DeviceRpcResponse { .. }
                | Self::RemoveRpc { .. }
                | Self::RuleNodeUpdated { .. }
                | Self::CfCacheInit { .. }
                | Self::CfStateRestore { .. }
                | Self::CfPartitionsChange { .. }
                | Self::CfEntityLifecycle { .. }
                | Self::EdgeHighPriority { .. }
        )
    }
}

#[cfg(test)]
mod tests;
