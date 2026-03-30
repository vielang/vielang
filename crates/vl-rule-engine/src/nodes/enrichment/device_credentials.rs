use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Fetch device credentials and put into message metadata.
/// Metadata keys: deviceCredentialsType, deviceToken (for ACCESS_TOKEN), etc.
/// Java: TbFetchDeviceCredentialsNode
/// Config: `{}`
pub struct FetchDeviceCredentialsNode;

impl FetchDeviceCredentialsNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self)
    }
}

#[async_trait]
impl RuleNode for FetchDeviceCredentialsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if msg.originator_type != "DEVICE" {
            return Ok(vec![(RelationType::Failure, msg)]);
        }

        let Some(creds) = ctx.dao.device.get_credentials(msg.originator_id).await? else {
            return Ok(vec![(RelationType::Failure, msg)]);
        };

        let mut out = msg;
        let cred_type = format!("{:?}", creds.credentials_type);
        out.metadata.insert("deviceCredentialsType".into(), cred_type);
        out.metadata.insert("deviceToken".into(), creds.credentials_id);
        if let Some(val) = creds.credentials_value {
            out.metadata.insert("deviceCredentialsValue".into(), val);
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
