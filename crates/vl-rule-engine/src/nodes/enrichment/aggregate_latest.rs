use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Aggregate latest telemetry values from the originator across multiple keys.
/// Supported aggregations: SUM, AVG, MIN, MAX, COUNT.
/// Result is added to the message payload.
/// Java: TbAggregateLatestNode
/// Relations: Success, Failure (no keys found)
/// Config:
/// ```json
/// {
///   "inputKeys": ["temperature", "humidity", "pressure"],
///   "outputKey": "avg_env",
///   "aggregation": "AVG",
///   "round": 2,
///   "tellFailureIfAbsent": false
/// }
/// ```
pub struct AggregateLatestNode {
    input_keys:            Vec<String>,
    output_key:            String,
    aggregation:           AggType,
    precision:             Option<u32>,
    tell_failure_if_absent: bool,
}

#[derive(Debug, Clone, Copy)]
enum AggType { Sum, Avg, Min, Max, Count }

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "inputKeys")]
    input_keys: Vec<String>,
    #[serde(rename = "outputKey")]
    output_key: String,
    #[serde(rename = "aggregation", default = "default_agg")]
    aggregation: String,
    #[serde(rename = "round")]
    round: Option<u32>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_agg() -> String { "AVG".into() }

impl AggregateLatestNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AggregateLatestNode: {}", e)))?;
        if cfg.input_keys.is_empty() {
            return Err(RuleEngineError::Config(
                "AggregateLatestNode: 'inputKeys' must not be empty".into()));
        }
        let agg = match cfg.aggregation.to_uppercase().as_str() {
            "SUM"   => AggType::Sum,
            "AVG"   => AggType::Avg,
            "MIN"   => AggType::Min,
            "MAX"   => AggType::Max,
            "COUNT" => AggType::Count,
            other   => return Err(RuleEngineError::Config(
                format!("AggregateLatestNode: unknown aggregation '{}'", other))),
        };
        Ok(Self {
            input_keys: cfg.input_keys,
            output_key: cfg.output_key,
            aggregation: agg,
            precision: cfg.round,
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }

    fn apply_agg(values: &[f64], agg: AggType) -> f64 {
        match agg {
            AggType::Sum   => values.iter().sum(),
            AggType::Avg   => values.iter().sum::<f64>() / values.len() as f64,
            AggType::Min   => values.iter().cloned().fold(f64::INFINITY, f64::min),
            AggType::Max   => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            AggType::Count => values.len() as f64,
        }
    }

    fn round(val: f64, precision: u32) -> f64 {
        let factor = 10f64.powi(precision as i32);
        (val * factor).round() / factor
    }
}

#[async_trait]
impl RuleNode for AggregateLatestNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let key_ids = ctx.dao.kv.lookup_key_ids(&self.input_keys).await?;

        let all_ids: Vec<i32> = key_ids.values().copied().collect();
        let entries = ctx.dao.kv.find_latest(msg.originator_id, &all_ids).await?;

        let values: Vec<f64> = entries.iter()
            .filter_map(|e| {
                e.dbl_v
                    .or_else(|| e.long_v.map(|v| v as f64))
                    .or_else(|| e.bool_v.map(|v| if v { 1.0 } else { 0.0 }))
            })
            .collect();

        if values.is_empty() {
            if self.tell_failure_if_absent {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    "AggregateLatestNode: no numeric values found for input keys".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let mut result = Self::apply_agg(&values, self.aggregation);
        if let Some(p) = self.precision {
            result = Self::round(result, p);
        }

        let mut out = msg;
        if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
            obj[&self.output_key] = serde_json::json!(result);
            out.data = serde_json::to_string(&obj).unwrap_or(out.data);
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_config() {
        let node = AggregateLatestNode::new(&json!({
            "inputKeys": ["a", "b", "c"],
            "outputKey": "avg_abc",
            "aggregation": "AVG"
        })).unwrap();
        assert_eq!(node.input_keys.len(), 3);
    }

    #[test]
    fn empty_keys_is_error() {
        assert!(AggregateLatestNode::new(&json!({
            "inputKeys": [],
            "outputKey": "result"
        })).is_err());
    }

    #[test]
    fn aggregations() {
        let vals = vec![10.0, 20.0, 30.0];
        assert_eq!(AggregateLatestNode::apply_agg(&vals, AggType::Sum), 60.0);
        assert_eq!(AggregateLatestNode::apply_agg(&vals, AggType::Avg), 20.0);
        assert_eq!(AggregateLatestNode::apply_agg(&vals, AggType::Min), 10.0);
        assert_eq!(AggregateLatestNode::apply_agg(&vals, AggType::Max), 30.0);
        assert_eq!(AggregateLatestNode::apply_agg(&vals, AggType::Count), 3.0);
    }
}
