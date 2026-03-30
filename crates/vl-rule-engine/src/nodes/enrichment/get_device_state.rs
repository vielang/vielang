use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Enrich message metadata with device connectivity / activity state.
/// Java: TbGetDeviceStateNode (activity-based state enrichment)
/// Relations: Success, Failure (device not found or not a DEVICE originator)
/// Config:
/// ```json
/// {
///   "inactivityTimeoutMs": 60000,
///   "fetchLastActivityTime": true,
///   "fetchConnectivityState": true
/// }
/// ```
/// Adds to metadata:
///   - `deviceState`        → "ACTIVE" | "INACTIVE" | "DISCONNECTED"
///   - `lastActivityTime`   → epoch milliseconds string (if fetchLastActivityTime)
///   - `inactivityTimeout`  → configured timeout string
pub struct GetDeviceStateNode {
    inactivity_timeout_ms:   i64,
    fetch_last_activity:     bool,
    fetch_connectivity:      bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "inactivityTimeoutMs", default = "default_timeout")]
    inactivity_timeout_ms: i64,
    #[serde(rename = "fetchLastActivityTime", default = "default_true")]
    fetch_last_activity: bool,
    #[serde(rename = "fetchConnectivityState", default = "default_true")]
    fetch_connectivity: bool,
}

fn default_timeout() -> i64 { 60_000 }
fn default_true() -> bool { true }

impl GetDeviceStateNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetDeviceStateNode: {}", e)))?;
        Ok(Self {
            inactivity_timeout_ms: cfg.inactivity_timeout_ms,
            fetch_last_activity: cfg.fetch_last_activity,
            fetch_connectivity: cfg.fetch_connectivity,
        })
    }
}

#[async_trait]
impl RuleNode for GetDeviceStateNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if msg.originator_type.to_uppercase() != "DEVICE" {
            let mut m = msg;
            m.metadata.insert("error".into(),
                "GetDeviceStateNode: originator must be DEVICE".into());
            return Ok(vec![(RelationType::Failure, m)]);
        }

        // Verify device exists
        if ctx.dao.device.find_by_id(msg.originator_id).await?.is_none() {
            return Ok(vec![(RelationType::Failure, msg)]);
        }

        let last_activity = ctx.dao.device
            .find_last_activity_time(msg.originator_id)
            .await?;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let state = match last_activity {
            None     => "DISCONNECTED",
            Some(ts) if (now_ms - ts) <= self.inactivity_timeout_ms => "ACTIVE",
            Some(_)  => "INACTIVE",
        };

        let mut out = msg;

        if self.fetch_connectivity {
            out.metadata.insert("deviceState".into(), state.to_string());
            out.metadata.insert("inactivityTimeout".into(),
                self.inactivity_timeout_ms.to_string());
        }

        if self.fetch_last_activity {
            let last_ts = last_activity.map(|t| t.to_string()).unwrap_or_else(|| "0".into());
            out.metadata.insert("lastActivityTime".into(), last_ts);
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults() {
        let node = GetDeviceStateNode::new(&json!({})).unwrap();
        assert_eq!(node.inactivity_timeout_ms, 60_000);
        assert!(node.fetch_last_activity);
        assert!(node.fetch_connectivity);
    }

    #[test]
    fn custom_timeout() {
        let node = GetDeviceStateNode::new(&json!({ "inactivityTimeoutMs": 300000 })).unwrap();
        assert_eq!(node.inactivity_timeout_ms, 300_000);
    }
}
