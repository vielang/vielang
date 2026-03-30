use std::collections::HashSet;
use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Filter messages by originator entity type.
/// True if originator type is in the allowed list, False otherwise.
/// Java: TbOriginatorTypeFilterNode
/// Config: `{ "originatorTypes": ["DEVICE", "ASSET"] }`
pub struct OriginatorTypeFilterNode {
    allowed_types: HashSet<String>,
}

impl OriginatorTypeFilterNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let types = config["originatorTypes"]
            .as_array()
            .ok_or_else(|| RuleEngineError::Config("originatorTypes array required".into()))?;
        let allowed_types = types
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        Ok(Self { allowed_types })
    }
}

#[async_trait]
impl RuleNode for OriginatorTypeFilterNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let matched = self.allowed_types.contains(&msg.originator_type);
        let rel = if matched { RelationType::True } else { RelationType::False };
        Ok(vec![(rel, msg)])
    }
}
