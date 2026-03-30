use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Routes messages based on device activity state.
/// Java: TbDeviceStateSwitchNode (part of profile nodes, also used as standalone filter)
/// Relations (as RelationType::Other):
///   - "Active"       — device has sent data within inactivity_timeout
///   - "Inactive"     — device has not sent data within inactivity_timeout
///   - "Disconnected" — device has never connected or last_activity_time is null
/// Config:
/// ```json
/// { "inactivityTimeoutMs": 60000 }
/// ```
pub struct DeviceStateSwitchNode {
    inactivity_timeout_ms: i64,
}

impl DeviceStateSwitchNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let timeout = config["inactivityTimeoutMs"]
            .as_i64()
            .unwrap_or(60_000); // default 1 minute
        Ok(Self { inactivity_timeout_ms: timeout })
    }
}

#[async_trait]
impl RuleNode for DeviceStateSwitchNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let last_activity = ctx.dao.device
            .find_last_activity_time(msg.originator_id)
            .await?;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let relation = match last_activity {
            None => RelationType::Other("Disconnected".into()),
            Some(ts) if (now_ms - ts) <= self.inactivity_timeout_ms => {
                RelationType::Other("Active".into())
            }
            Some(_) => RelationType::Other("Inactive".into()),
        };

        Ok(vec![(relation, msg)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_timeout() {
        let node = DeviceStateSwitchNode::new(&json!({})).unwrap();
        assert_eq!(node.inactivity_timeout_ms, 60_000);
    }

    #[test]
    fn custom_timeout() {
        let node = DeviceStateSwitchNode::new(&json!({ "inactivityTimeoutMs": 300000 })).unwrap();
        assert_eq!(node.inactivity_timeout_ms, 300_000);
    }
}
