use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Stub for TbMsgGeneratorNode — generates periodic messages (not yet implemented).
/// Config: `{ "msgCount": 10, "msgType": "POST_TELEMETRY_REQUEST", "jsScript": "..." }`
///
/// Current behaviour: passes the input message through on Success.
pub struct MsgGeneratorNode;

impl MsgGeneratorNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for MsgGeneratorNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        Ok(vec![(RelationType::Success, msg)])
    }
}
