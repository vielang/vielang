//! Per-rule-chain actor — orchestrates rule nodes.
//!
//! Mirrors ThingsBoard's `RuleChainActor` + `RuleChainActorMessageProcessor`:
//! - Loads rule chain metadata (nodes + connections) from the DAO.
//! - Creates `RuleNodeActor` for each node in the chain.
//! - Routes incoming messages to the first (entry) node.
//! - Handles `TellNext` — routes node output to next nodes via relation types.
//! - Supports cross-chain routing (RuleChainInput/Output messages).

use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, error, warn};
use uuid::Uuid;
use vl_actor::{
    ActorError, ActorMsg, EntityType, RuleEngineMsg, StopReason, TbActor, TbActorCtx, TbActorId,
};

use super::{ActorSystemCtx, RuleNodeActor};

/// Routing table entry: relation_type → target_node_ids.
#[derive(Debug, Clone)]
struct RouteEntry {
    relation_type: String,
    target_node_ids: Vec<Uuid>,
}

pub struct RuleChainActor {
    tenant_id: Uuid,
    chain_id: Uuid,
    sys_ctx: ActorSystemCtx,
    ctx: Option<TbActorCtx>,
    /// Entry point node ID.
    first_node_id: Option<Uuid>,
    /// Routing table: source_node_index → Vec<RouteEntry>.
    /// We map node_index → node_id after loading.
    node_routes: HashMap<Uuid, Vec<RouteEntry>>,
}

impl RuleChainActor {
    pub fn new(tenant_id: Uuid, chain_id: Uuid, sys_ctx: ActorSystemCtx) -> Self {
        Self {
            tenant_id,
            chain_id,
            sys_ctx,
            ctx: None,
            first_node_id: None,
            node_routes: HashMap::new(),
        }
    }

    /// Load rule chain metadata and create rule node actors.
    async fn load_chain(&mut self) {
        let ctx = match &self.ctx {
            Some(c) => c,
            None => return,
        };

        // Load full metadata (nodes + connections).
        let metadata = match self.sys_ctx.rule_chain_dao.find_metadata(self.chain_id).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                warn!("chain {}: metadata not found", self.chain_id);
                return;
            }
            Err(e) => {
                error!("chain {}: failed to load metadata: {e}", self.chain_id);
                return;
            }
        };

        debug!(
            "chain {}: loaded {} nodes, {} connections",
            self.chain_id,
            metadata.nodes.len(),
            metadata.connections.len()
        );

        // Build index → node_id mapping.
        let mut index_to_id: HashMap<i32, Uuid> = HashMap::new();
        for (idx, node) in metadata.nodes.iter().enumerate() {
            if let Some(node_id) = node.id {
                index_to_id.insert(idx as i32, node_id);
            }
        }

        // Set first node.
        self.first_node_id = metadata
            .first_node_index
            .and_then(|idx| index_to_id.get(&idx).copied())
            .or_else(|| index_to_id.get(&0).copied());

        // Create node actors.
        for node in &metadata.nodes {
            let Some(node_id) = node.id else { continue };
            let sys = self.sys_ctx.clone();
            let tid = self.tenant_id;
            let cid = self.chain_id;
            let node_type = node.type_.clone();
            let config = node.configuration.clone();

            let actor_id = TbActorId::entity(node_id, EntityType::RuleNode);
            if let Err(e) = ctx.get_or_create_child(actor_id, move || {
                Box::new(RuleNodeActor::new(tid, cid, node_id, node_type, config, sys))
            }) {
                error!(
                    "chain {}: failed to create RuleNodeActor {node_id}: {e}",
                    self.chain_id
                );
            }
        }

        // Build routing table: from_index → (relation_type, to_node_id).
        self.node_routes.clear();
        for conn in &metadata.connections {
            let Some(&from_id) = index_to_id.get(&conn.from_index) else {
                continue;
            };
            let Some(&to_id) = index_to_id.get(&conn.to_index) else {
                continue;
            };
            self.node_routes
                .entry(from_id)
                .or_default()
                .push(RouteEntry {
                    relation_type: conn.type_.clone(),
                    target_node_ids: vec![to_id],
                });
        }
    }

    /// Route a message to the first (entry) node.
    fn send_to_first_node(&self, msg: Box<RuleEngineMsg>) {
        let Some(first_id) = self.first_node_id else {
            warn!("chain {}: no first node, dropping message", self.chain_id);
            return;
        };
        let Some(ctx) = &self.ctx else { return };
        let node_actor_id = TbActorId::entity(first_id, EntityType::RuleNode);
        ctx.tell(
            &node_actor_id,
            ActorMsg::RuleChainToRuleNode {
                rule_node_id: first_id,
                from_relation_type: String::new(),
                msg,
            },
        );
    }

    /// Handle TellNext: route from one node to the next based on relation type.
    fn on_tell_next(
        &self,
        originator_node_id: Uuid,
        relation_types: &[String],
        msg: Box<RuleEngineMsg>,
    ) {
        let Some(routes) = self.node_routes.get(&originator_node_id) else {
            debug!(
                "chain {}: no routes from node {originator_node_id}",
                self.chain_id
            );
            return;
        };
        let Some(ctx) = &self.ctx else { return };

        for route in routes {
            if relation_types.is_empty() || relation_types.contains(&route.relation_type) {
                for &target_id in &route.target_node_ids {
                    let node_actor_id = TbActorId::entity(target_id, EntityType::RuleNode);
                    ctx.tell(
                        &node_actor_id,
                        ActorMsg::RuleChainToRuleNode {
                            rule_node_id: target_id,
                            from_relation_type: route.relation_type.clone(),
                            msg: msg.clone(),
                        },
                    );
                }
            }
        }
    }
}

#[async_trait]
impl TbActor for RuleChainActor {
    async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
        debug!("RuleChainActor {} initializing", self.chain_id);
        self.ctx = Some(ctx);
        self.load_chain().await;
        Ok(())
    }

    async fn destroy(&mut self, reason: StopReason) {
        debug!("RuleChainActor {} destroyed: {reason:?}", self.chain_id);
    }

    async fn process(&mut self, msg: ActorMsg) -> bool {
        match msg {
            ActorMsg::QueueToRuleEngine { msg: re_msg, .. } => {
                self.send_to_first_node(re_msg);
                true
            }

            ActorMsg::RuleNodeToRuleChainTellNext {
                originator_node_id,
                ref relation_types,
                msg: re_msg,
                ..
            } => {
                self.on_tell_next(originator_node_id, relation_types, re_msg);
                true
            }

            ActorMsg::RuleChainToRuleChain { msg: re_msg, .. } => {
                self.send_to_first_node(re_msg);
                true
            }

            ActorMsg::RuleChainInput { msg: re_msg, .. } => {
                self.send_to_first_node(re_msg);
                true
            }

            ActorMsg::RuleChainOutput {
                target_node_id,
                ref relation_type,
                msg: re_msg,
                ..
            } => {
                let Some(ctx) = &self.ctx else { return true };
                let node_actor_id = TbActorId::entity(target_node_id, EntityType::RuleNode);
                ctx.tell(
                    &node_actor_id,
                    ActorMsg::RuleChainToRuleNode {
                        rule_node_id: target_node_id,
                        from_relation_type: relation_type.clone(),
                        msg: re_msg,
                    },
                );
                true
            }

            ActorMsg::ComponentLifecycle { event, .. } => {
                match event {
                    vl_actor::LifecycleEvent::Updated | vl_actor::LifecycleEvent::Created => {
                        self.load_chain().await;
                    }
                    _ => {}
                }
                true
            }

            ActorMsg::PartitionChange { .. } | ActorMsg::StatsPersistTick => true,

            _ => false,
        }
    }
}
