use std::collections::HashSet;
use async_trait::async_trait;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};
use vl_core::entities::TbMsg;

/// Filter messages by their type.
/// Config: `{ "messageTypes": ["POST_TELEMETRY_REQUEST", ...] }`
pub struct MsgTypeFilter {
    allowed_types: HashSet<String>,
}

impl MsgTypeFilter {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let types = config["messageTypes"]
            .as_array()
            .ok_or_else(|| RuleEngineError::Config("messageTypes array required".into()))?;
        let allowed_types = types
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        Ok(Self { allowed_types })
    }
}

#[async_trait]
impl RuleNode for MsgTypeFilter {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.allowed_types.contains(&msg.msg_type) {
            Ok(vec![(RelationType::True, msg)])
        } else {
            Ok(vec![(RelationType::False, msg)])
        }
    }
}
