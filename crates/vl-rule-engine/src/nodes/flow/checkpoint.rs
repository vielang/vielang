use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Checkpoint node — marks a processing checkpoint in the rule chain.
/// Passes the message through on Success. In distributed mode, this would
/// acknowledge the message was processed up to this point.
/// Config JSON: `{ "queueName": "Main" }`
pub struct CheckpointNode {
    queue_name: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "queueName", default = "default_queue")]
    queue_name: String,
}

fn default_queue() -> String { "Main".to_string() }

impl CheckpointNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CheckpointNode: {}", e)))?;
        Ok(Self { queue_name: cfg.queue_name })
    }
}

#[async_trait]
impl RuleNode for CheckpointNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        tracing::debug!(
            node_id    = %ctx.node_id,
            queue_name = %self.queue_name,
            msg_id     = %msg.id,
            "Checkpoint reached"
        );
        Ok(vec![(RelationType::Success, msg)])
    }
}
