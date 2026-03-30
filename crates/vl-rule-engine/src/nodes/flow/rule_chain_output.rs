use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Stub for RuleChainOutputNode — signals that a message should be forwarded
/// to the output of this rule chain (used when chains call sub-chains).
/// Config: `{}`
///
/// Current behaviour: passes the input message through on Success.
pub struct RuleChainOutputNode;

impl RuleChainOutputNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for RuleChainOutputNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        Ok(vec![(RelationType::Success, msg)])
    }
}
