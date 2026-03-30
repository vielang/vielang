use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Calculate delta (difference) between current and previous value for specified keys.
/// Previous values are kept in-memory (resets on restart) or read from latest telemetry.
/// Java: TbCalculateDeltaNode
/// Relations: Success, Failure (key missing or not numeric)
/// Config:
/// ```json
/// {
///   "inputValueKey": "pulseCounter",
///   "outputValueKey": "delta",
///   "useCache": true,
///   "tellFailureIfDeltaIsNegative": false,
///   "round": 3,
///   "addPeriodBetweenMsgs": false,
///   "periodValueKey": "periodInMs"
/// }
/// ```
pub struct CalculateDeltaNode {
    input_key:                      String,
    output_key:                     String,
    use_cache:                      bool,
    fail_on_negative:               bool,
    precision:                      Option<u32>,
    add_period:                     bool,
    period_key:                     String,
    // per-device cache: device_id → (last_value, last_ts)
    cache: Arc<RwLock<HashMap<uuid::Uuid, (f64, i64)>>>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "inputValueKey")]
    input_value_key: String,
    #[serde(rename = "outputValueKey", default = "default_output")]
    output_value_key: String,
    #[serde(rename = "useCache", default = "default_true")]
    use_cache: bool,
    #[serde(rename = "tellFailureIfDeltaIsNegative", default)]
    tell_failure_if_delta_is_negative: bool,
    #[serde(rename = "round")]
    round: Option<u32>,
    #[serde(rename = "addPeriodBetweenMsgs", default)]
    add_period: bool,
    #[serde(rename = "periodValueKey", default = "default_period_key")]
    period_value_key: String,
}

fn default_output() -> String { "delta".into() }
fn default_true() -> bool { true }
fn default_period_key() -> String { "periodInMs".into() }

impl CalculateDeltaNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CalculateDeltaNode: {}", e)))?;
        Ok(Self {
            input_key: cfg.input_value_key,
            output_key: cfg.output_value_key,
            use_cache: cfg.use_cache,
            fail_on_negative: cfg.tell_failure_if_delta_is_negative,
            precision: cfg.round,
            add_period: cfg.add_period,
            period_key: cfg.period_value_key,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn round(val: f64, precision: u32) -> f64 {
        let factor = 10f64.powi(precision as i32);
        (val * factor).round() / factor
    }
}

#[async_trait]
impl RuleNode for CalculateDeltaNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = match serde_json::from_str(&msg.data) {
            Ok(v) => v,
            Err(_) => {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    "CalculateDeltaNode: msg data is not JSON".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let current = match data.get(&self.input_key).and_then(|v| v.as_f64()) {
            Some(v) => v,
            None => {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    format!("CalculateDeltaNode: key '{}' missing or not numeric", self.input_key));
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let current_ts = msg.ts;
        let prev_opt = if self.use_cache {
            self.cache.read().await.get(&msg.originator_id).copied()
        } else {
            // Fetch from latest telemetry table
            let key_ids = ctx.dao.kv.lookup_key_ids(&[self.input_key.clone()]).await?;
            if let Some(&key_id) = key_ids.get(&self.input_key) {
                let entries = ctx.dao.kv.find_latest(msg.originator_id, &[key_id]).await?;
                entries.into_iter().next()
                    .and_then(|e| e.dbl_v.or_else(|| e.long_v.map(|v| v as f64)))
                    .map(|v| (v, 0i64))
            } else {
                None
            }
        };

        // Update cache with current value
        if self.use_cache {
            self.cache.write().await.insert(msg.originator_id, (current, current_ts));
        }

        let mut out = msg;

        if let Some((prev_val, prev_ts)) = prev_opt {
            let mut delta = current - prev_val;
            if let Some(p) = self.precision {
                delta = Self::round(delta, p);
            }

            if delta < 0.0 && self.fail_on_negative {
                out.metadata.insert("error".into(),
                    format!("CalculateDeltaNode: negative delta {}", delta));
                return Ok(vec![(RelationType::Failure, out)]);
            }

            if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                obj[&self.output_key] = serde_json::json!(delta);
                if self.add_period && prev_ts > 0 {
                    let period = current_ts - prev_ts;
                    obj[&self.period_key] = serde_json::json!(period);
                }
                out.data = serde_json::to_string(&obj).unwrap_or(out.data);
            }
        }
        // If no previous value: pass through without delta (first message)

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_config() {
        let node = CalculateDeltaNode::new(&json!({
            "inputValueKey": "pulseCounter",
            "outputValueKey": "delta",
            "useCache": true,
            "round": 2
        })).unwrap();
        assert_eq!(node.input_key, "pulseCounter");
        assert_eq!(node.output_key, "delta");
        assert_eq!(node.precision, Some(2));
    }

    #[test]
    fn rounding_works() {
        assert_eq!(CalculateDeltaNode::round(1.23456, 2), 1.23);
        assert_eq!(CalculateDeltaNode::round(1.23456, 3), 1.235);
    }
}
