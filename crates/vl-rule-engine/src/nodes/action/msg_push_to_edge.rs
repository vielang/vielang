use async_trait::async_trait;
use uuid::Uuid;

use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// TbMsgPushToEdgeNode — push message tới Edge gateway.
///
/// Config JSON: `{ "edgeId": "<uuid>" }` (nếu thiếu → push tới tất cả edges của tenant)
///
/// Requires: `RuleNodeCtx.edge_sender` must be set (khi có EdgeSessionRegistry).
/// Nếu không có edge_sender (edge gRPC disabled) → luôn trả Success (no-op).
pub struct MsgPushToEdgeNode {
    edge_id: Option<Uuid>,
}

impl MsgPushToEdgeNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let edge_id = config
            .get("edgeId")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Uuid>().ok());
        Ok(Self { edge_id })
    }
}

#[async_trait]
impl RuleNode for MsgPushToEdgeNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let Some(sender) = &ctx.edge_sender else {
            // Edge gRPC not enabled — pass through silently
            return Ok(vec![(RelationType::Success, msg)]);
        };

        let payload = serde_json::json!({
            "msgType":  msg.msg_type,
            "data":     msg.data,
            "metadata": msg.metadata,
        });

        match self.edge_id {
            Some(edge_id) => sender.push_to_edge(edge_id, payload),
            None          => sender.push_to_tenant_edges(ctx.tenant_id, payload),
        }

        Ok(vec![(RelationType::Success, msg)])
    }
}
