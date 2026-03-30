use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Fetch the device profile ID and name of the originating device and add to metadata.
/// Config JSON:
/// ```json
/// { "addProfileIdToMetadata": true }
/// ```
pub struct DeviceProfileNode {
    add_profile_id: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "addProfileIdToMetadata", default)]
    add_profile_id: bool,
}

impl DeviceProfileNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("DeviceProfileNode: {}", e)))?;
        Ok(Self { add_profile_id: cfg.add_profile_id })
    }
}

#[async_trait]
impl RuleNode for DeviceProfileNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        if out.originator_type.to_uppercase() == "DEVICE" {
            if let Some(device) = ctx.dao.device.find_by_id(out.originator_id).await? {
                out.metadata.insert("deviceName".into(), device.name);
                out.metadata.insert("deviceType".into(), device.device_type);
                if self.add_profile_id {
                    out.metadata.insert(
                        "deviceProfileId".into(),
                        device.device_profile_id.to_string(),
                    );
                }
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
