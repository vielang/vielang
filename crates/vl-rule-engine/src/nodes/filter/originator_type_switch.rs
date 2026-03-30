use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Route messages based on originator entity type.
/// Routes to a relation named after the originator type (e.g. "DEVICE", "ASSET").
/// Java: TbOriginatorTypeSwitchNode
/// Config: `{}`
pub struct OriginatorTypeSwitchNode;

impl OriginatorTypeSwitchNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for OriginatorTypeSwitchNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let rel = RelationType::Other(msg.originator_type.clone());
        Ok(vec![(rel, msg)])
    }
}
