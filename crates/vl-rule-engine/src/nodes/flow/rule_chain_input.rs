use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Entry point node for a rule chain — all messages enter through this node.
/// Simply passes the message through on Success.
/// Config: `{}`
pub struct RuleChainInputNode;

impl RuleChainInputNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for RuleChainInputNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        Ok(vec![(RelationType::Success, msg)])
    }
}
