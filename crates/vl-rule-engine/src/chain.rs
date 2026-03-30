use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::warn;

use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RuleNode, RuleNodeCtx},
    nodes::debug::save_debug_event_bg,
    registry::NodeRegistry,
};

// ── Config DTOs (stored as JSON in rule_chain.configuration) ─────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    pub id:         Uuid,
    #[serde(rename = "type")]
    pub node_type:  String,
    pub config:     serde_json::Value,
    /// When true, every processed message is persisted as a DEBUG_RULE_NODE event.
    /// Matches ThingsBoard `TbNodeConfiguration.debugMode`.
    #[serde(default, rename = "debugMode")]
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDef {
    pub from_id:       Uuid,
    pub relation_type: String,
    pub to_id:         Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleChainConfig {
    pub nodes:       Vec<NodeDef>,
    pub connections: Vec<ConnectionDef>,
    /// Index into `nodes` for the first node (default 0)
    #[serde(default)]
    pub first_node_index: usize,
}

// ── Executor ──────────────────────────────────────────────────────────────────

pub struct RuleChain {
    pub id:            Uuid,
    pub first_node_id: Uuid,
    /// node_id → node implementation
    nodes:             HashMap<Uuid, Box<dyn RuleNode>>,
    /// (from_node_id, relation_type_string) → list of next node ids
    connections:       HashMap<(Uuid, String), Vec<Uuid>>,
    /// Set of node_ids that have debugMode = true
    debug_nodes:       HashSet<Uuid>,
}

impl RuleChain {
    /// Build from a DB rule_chain record.
    /// `config_json` = `rule_chain.configuration` column value.
    pub fn from_config_str(
        id: Uuid,
        config_json: &str,
        registry: &NodeRegistry,
    ) -> Result<Self, RuleEngineError> {
        let config: RuleChainConfig = serde_json::from_str(config_json)?;
        Self::from_config(id, &config, registry)
    }

    pub fn from_config(
        id: Uuid,
        config: &RuleChainConfig,
        registry: &NodeRegistry,
    ) -> Result<Self, RuleEngineError> {
        if config.nodes.is_empty() {
            return Err(RuleEngineError::Config("Rule chain must have at least one node".into()));
        }

        let first_node_id = config.nodes
            .get(config.first_node_index)
            .map(|n| n.id)
            .ok_or_else(|| RuleEngineError::Config("first_node_index out of bounds".into()))?;

        let mut nodes: HashMap<Uuid, Box<dyn RuleNode>> = HashMap::new();
        let mut debug_nodes: HashSet<Uuid> = HashSet::new();
        for node_def in &config.nodes {
            let node = registry.create(&node_def.node_type, &node_def.config)?;
            nodes.insert(node_def.id, node);
            if node_def.debug_mode {
                debug_nodes.insert(node_def.id);
            }
        }

        let mut connections: HashMap<(Uuid, String), Vec<Uuid>> = HashMap::new();
        for conn in &config.connections {
            connections
                .entry((conn.from_id, conn.relation_type.clone()))
                .or_default()
                .push(conn.to_id);
        }

        Ok(Self { id, first_node_id, nodes, connections, debug_nodes })
    }

    /// Process a message through this chain.
    /// Uses an iterative work-queue to avoid async recursion.
    pub async fn process_msg(
        &self,
        ctx: &RuleNodeCtx,
        initial_msg: TbMsg,
    ) -> Result<(), RuleEngineError> {
        let mut work: VecDeque<(Uuid, TbMsg)> = VecDeque::new();
        work.push_back((self.first_node_id, initial_msg));

        while let Some((node_id, msg)) = work.pop_front() {
            let node = match self.nodes.get(&node_id) {
                Some(n) => n,
                None => {
                    warn!(chain_id = %self.id, node_id = %node_id, "Node not found in chain");
                    continue;
                }
            };

            let debug = self.debug_nodes.contains(&node_id);
            // Clone message before move — only pays the cost when debug is on
            let pre_msg = if debug { Some(msg.clone()) } else { None };

            let results = node.process(ctx, msg).await;

            // Persist debug event (fire-and-forget) before propagating errors
            if debug {
                let pre = pre_msg.unwrap();
                match &results {
                    Ok(outputs) => {
                        let rel = outputs.first()
                            .map(|(r, _)| r.to_string())
                            .unwrap_or_else(|| "Success".to_string());
                        save_debug_event_bg(ctx, node_id, &pre, &rel, None);
                    }
                    Err(e) => {
                        save_debug_event_bg(ctx, node_id, &pre, "Failure", Some(e.to_string()));
                    }
                }
            }

            let outputs = results?;
            for (relation_type, out_msg) in outputs {
                let key = (node_id, relation_type.to_string());
                if let Some(next_ids) = self.connections.get(&key) {
                    for &next_id in next_ids {
                        work.push_back((next_id, out_msg.clone()));
                    }
                }
            }
        }

        Ok(())
    }
}
