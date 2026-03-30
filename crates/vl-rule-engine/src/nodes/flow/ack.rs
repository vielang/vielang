use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Acknowledge node — stops message processing (returns empty route list).
/// In ThingsBoard, this ACKs the message from the queue and stops its propagation.
/// Java: TbAckNode
/// Config: `{}`
pub struct AckNode;

impl AckNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for AckNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // ACK: pass through with Success, no further routing
        Ok(vec![(RelationType::Success, msg)])
    }
}
