use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Route message to relation named after asset type.
/// Java: TbAssetTypeSwitchNode
/// Config: `{}`
pub struct AssetTypeSwitchNode;

impl AssetTypeSwitchNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for AssetTypeSwitchNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if msg.originator_type != "ASSET" {
            return Ok(vec![(RelationType::Other("Other".into()), msg)]);
        }

        let relation = match ctx.dao.asset.find_by_id(msg.originator_id).await? {
            Some(asset) => RelationType::Other(asset.asset_type),
            None        => RelationType::Other("Other".into()),
        };
        Ok(vec![(relation, msg)])
    }
}
