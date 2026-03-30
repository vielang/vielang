use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Unassign originator entity (DEVICE or ASSET) from its current customer.
/// Java: TbUnassignFromCustomerNode
/// Config: `{}`
pub struct UnassignFromCustomerNode;

impl UnassignFromCustomerNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for UnassignFromCustomerNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        match msg.originator_type.as_str() {
            "DEVICE" => {
                if let Some(mut device) = ctx.dao.device.find_by_id(msg.originator_id).await? {
                    device.customer_id = None;
                    ctx.dao.device.save(&device).await?;
                    Ok(vec![(RelationType::Success, msg)])
                } else {
                    Ok(vec![(RelationType::Failure, msg)])
                }
            }
            "ASSET" => {
                if let Some(mut asset) = ctx.dao.asset.find_by_id(msg.originator_id).await? {
                    asset.customer_id = None;
                    ctx.dao.asset.save(&asset).await?;
                    Ok(vec![(RelationType::Success, msg)])
                } else {
                    Ok(vec![(RelationType::Failure, msg)])
                }
            }
            _ => Ok(vec![(RelationType::Failure, msg)]),
        }
    }
}
