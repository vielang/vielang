use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Route message to relation named after device type.
/// Fetches device from DB, uses device.device_type as relation name.
/// Falls back to "Other" if device not found.
/// Java: TbDeviceTypeSwitchNode
/// Config: `{}`
pub struct DeviceTypeSwitchNode;

impl DeviceTypeSwitchNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for DeviceTypeSwitchNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if msg.originator_type != "DEVICE" {
            return Ok(vec![(RelationType::Other("Other".into()), msg)]);
        }

        let relation = match ctx.dao.device.find_by_id(msg.originator_id).await? {
            Some(device) => RelationType::Other(device.device_type),
            None         => RelationType::Other("Other".into()),
        };
        Ok(vec![(relation, msg)])
    }
}
