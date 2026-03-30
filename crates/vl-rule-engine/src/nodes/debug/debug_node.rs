use async_trait::async_trait;
use uuid::Uuid;

use vl_core::entities::{DebugRuleNodeEventBody, Event, EventType, TbMsg};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Debug node — passthrough that persists every message as a `DEBUG_RULE_NODE`
/// event before forwarding on `Success`.
///
/// Use this node as an explicit checkpoint in a rule chain to capture all
/// messages flowing through a specific point. The events can be viewed in the
/// ThingsBoard UI rule engine debug panel or replayed via the replay tool.
///
/// ThingsBoard equivalent: placing a `TbDebugNode` in a rule chain.
///
/// Config: `{}` (no configuration required)
pub struct DebugNode;

impl DebugNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

/// Persist a `DEBUG_RULE_NODE` event in the background (fire-and-forget).
pub(crate) fn save_debug_event_bg(
    ctx: &RuleNodeCtx,
    node_id: Uuid,
    msg: &TbMsg,
    relation_type: &str,
    error: Option<String>,
) {
    let body = DebugRuleNodeEventBody {
        rule_chain_id: msg.rule_chain_id.unwrap_or_default(),
        rule_node_id: node_id,
        msg_type: msg.msg_type.clone(),
        relation_type: relation_type.to_string(),
        data: msg.data.clone(),
        metadata: serde_json::to_string(&msg.metadata).unwrap_or_default(),
        error,
    };
    let event = Event {
        id: Uuid::new_v4(),
        created_time: now_ms(),
        tenant_id: ctx.tenant_id,
        entity_id: node_id,
        entity_type: "RULE_NODE".to_string(),
        event_type: EventType::DebugRuleNode,
        // event_uid = node_id + msg_id: one debug record per (node, message) pair
        event_uid: format!("{}-{}", node_id, msg.id),
        body: serde_json::to_value(&body).unwrap_or_default(),
    };
    let dao = ctx.dao.event.clone();
    tokio::spawn(async move {
        if let Err(e) = dao.save(&event).await {
            tracing::warn!(error = %e, node_id = %node_id, "debug: failed to save event");
        }
    });
}

#[async_trait]
impl RuleNode for DebugNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        save_debug_event_bg(ctx, ctx.node_id, &msg, "Success", None);
        Ok(vec![(RelationType::Success, msg)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_empty_config() {
        assert!(DebugNode::new(&serde_json::json!({})).is_ok());
    }

    #[test]
    fn new_accepts_null_config() {
        assert!(DebugNode::new(&serde_json::json!(null)).is_ok());
    }
}
