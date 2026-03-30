use async_trait::async_trait;
use tracing::info;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Log node — logs message details and passes through on Success.
/// Config: `{ "level": "INFO" }` (INFO | DEBUG | WARN | ERROR)
pub struct LogNode {
    level: String,
}

impl LogNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let level = config["level"].as_str().unwrap_or("INFO").to_string();
        Ok(Self { level })
    }
}

#[async_trait]
impl RuleNode for LogNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        match self.level.to_uppercase().as_str() {
            "DEBUG" => tracing::debug!(
                node_id   = %ctx.node_id,
                msg_type  = %msg.msg_type,
                origin    = %msg.originator_id,
                data      = %msg.data,
                "RuleNode[Log]"
            ),
            "WARN" => tracing::warn!(
                node_id   = %ctx.node_id,
                msg_type  = %msg.msg_type,
                origin    = %msg.originator_id,
                data      = %msg.data,
                "RuleNode[Log]"
            ),
            "ERROR" => tracing::error!(
                node_id   = %ctx.node_id,
                msg_type  = %msg.msg_type,
                origin    = %msg.originator_id,
                data      = %msg.data,
                "RuleNode[Log]"
            ),
            _ => info!(
                node_id   = %ctx.node_id,
                msg_type  = %msg.msg_type,
                origin    = %msg.originator_id,
                data      = %msg.data,
                "RuleNode[Log]"
            ),
        }

        Ok(vec![(RelationType::Success, msg)])
    }
}
