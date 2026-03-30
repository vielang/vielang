use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Stub for TbSynchronizationBeginNode — serialises message processing per originator.
/// Config: `{}`
///
/// Current behaviour: passes the input message through on Success.
pub struct SynchronizationNode;

impl SynchronizationNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for SynchronizationNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        Ok(vec![(RelationType::Success, msg)])
    }
}
