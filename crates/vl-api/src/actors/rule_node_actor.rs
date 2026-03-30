//! Per-rule-node actor — executes a single rule engine node.
//!
//! Mirrors ThingsBoard's `RuleNodeActor` + `RuleNodeActorMessageProcessor`:
//! - Receives messages from the parent `RuleChainActor`.
//! - Invokes the configured rule node logic (filter, enrichment, action, etc.).
//! - Sends results back to the chain via `RuleNodeToRuleChainTellNext`.
//! - Supports self-messages for async callbacks.
//! - Tracks processing statistics.

use async_trait::async_trait;
use tracing::{debug, error};
use uuid::Uuid;
use vl_actor::{
    ActorError, ActorMsg, EntityType, RuleEngineMsg, StopReason, TbActor, TbActorCtx, TbActorId,
};

use super::ActorSystemCtx;

pub struct RuleNodeActor {
    tenant_id: Uuid,
    chain_id: Uuid,
    node_id: Uuid,
    node_type: String,
    configuration: Option<serde_json::Value>,
    sys_ctx: ActorSystemCtx,
    ctx: Option<TbActorCtx>,
    messages_processed: u64,
    errors_occurred: u64,
}

impl RuleNodeActor {
    pub fn new(
        tenant_id: Uuid,
        chain_id: Uuid,
        node_id: Uuid,
        node_type: String,
        configuration: Option<serde_json::Value>,
        sys_ctx: ActorSystemCtx,
    ) -> Self {
        Self {
            tenant_id,
            chain_id,
            node_id,
            node_type,
            configuration,
            sys_ctx,
            ctx: None,
            messages_processed: 0,
            errors_occurred: 0,
        }
    }

    /// Process a message through this rule node's logic.
    ///
    /// This delegates to the rule engine registry to find and execute
    /// the appropriate node implementation.
    async fn execute_node(&mut self, from_relation: &str, msg: Box<RuleEngineMsg>) {
        self.messages_processed += 1;

        // In a full implementation, this would:
        // 1. Look up the node implementation by node_type in the registry
        // 2. Call node.onMsg(ctx, msg)
        // 3. The node calls ctx.tellNext() which sends back to chain actor
        //
        // For now, forward to chain as "Success" by default.
        debug!(
            "node {} ({}): processing message from '{from_relation}'",
            self.node_id, self.node_type
        );

        // Send TellNext back to parent chain.
        let Some(ctx) = &self.ctx else { return };
        let chain_actor_id = TbActorId::entity(self.chain_id, EntityType::RuleChain);
        ctx.tell(
            &chain_actor_id,
            ActorMsg::RuleNodeToRuleChainTellNext {
                rule_chain_id: self.chain_id,
                originator_node_id: self.node_id,
                relation_types: vec!["Success".to_string()],
                msg,
                failure_message: None,
            },
        );
    }
}

#[async_trait]
impl TbActor for RuleNodeActor {
    async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
        debug!(
            "RuleNodeActor {} ({}) initializing",
            self.node_id, self.node_type
        );
        self.ctx = Some(ctx);
        Ok(())
    }

    async fn destroy(&mut self, reason: StopReason) {
        debug!(
            "RuleNodeActor {} ({}) destroyed: {reason:?} (processed={}, errors={})",
            self.node_id, self.node_type, self.messages_processed, self.errors_occurred
        );
    }

    async fn process(&mut self, msg: ActorMsg) -> bool {
        match msg {
            ActorMsg::RuleChainToRuleNode {
                from_relation_type,
                msg: re_msg,
                ..
            } => {
                self.execute_node(&from_relation_type, re_msg).await;
                true
            }

            ActorMsg::RuleToSelf { msg: re_msg } => {
                // Async callback — process as if from self.
                self.execute_node("Self", re_msg).await;
                true
            }

            ActorMsg::RuleNodeUpdated { .. } => {
                // Hot-reload: re-read configuration from DB.
                debug!(
                    "node {} ({}): configuration updated",
                    self.node_id, self.node_type
                );
                // In full impl: reload config from rule_node_dao and reinit node logic.
                true
            }

            ActorMsg::ComponentLifecycle { event, .. } => {
                debug!(
                    "node {} ({}): lifecycle event {event:?}",
                    self.node_id, self.node_type
                );
                true
            }

            ActorMsg::PartitionChange { .. } => true,

            ActorMsg::StatsPersistTick => {
                // Persist stats to the stats service.
                if self.messages_processed > 0 || self.errors_occurred > 0 {
                    debug!(
                        "node {} ({}): stats tick (processed={}, errors={})",
                        self.node_id, self.node_type,
                        self.messages_processed, self.errors_occurred
                    );
                    // Reset counters after reporting.
                    self.messages_processed = 0;
                    self.errors_occurred = 0;
                }
                true
            }

            _ => false,
        }
    }
}
