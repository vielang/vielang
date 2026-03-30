use async_trait::async_trait;
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::Arc;
use dashmap::DashMap;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Compute running statistics over a sliding window of telemetry values.
/// Java: TbStatisticsNode
/// Supported stats: COUNT, SUM, AVG, MIN, MAX, STD_DEV, VARIANCE, MEDIAN
/// Output is added to the message body under the configured output keys.
/// Relations: Success, Failure (key missing)
/// Config:
/// ```json
/// {
///   "inputKey": "temperature",
///   "windowSize": 10,           // number of latest values to keep
///   "outputKey": "temp_stats",  // prefix for output keys
///   "computations": ["AVG", "MIN", "MAX", "STD_DEV"],
///   "tellFailureIfAbsent": false
/// }
/// ```
/// Output keys: `{outputKey}_avg`, `{outputKey}_min`, etc.
pub struct StatisticsNode {
    input_key:             String,
    window_size:           usize,
    output_prefix:         String,
    computations:          Vec<StatComp>,
    tell_failure_if_absent: bool,
    // per-device circular buffer: originator_id → deque of values
    window: Arc<DashMap<uuid::Uuid, VecDeque<f64>>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StatComp { Count, Sum, Avg, Min, Max, StdDev, Variance, Median }

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "inputKey")]
    input_key: String,
    #[serde(rename = "windowSize", default = "default_window")]
    window_size: usize,
    #[serde(rename = "outputKey", default = "default_output")]
    output_key: String,
    #[serde(default = "default_computations")]
    computations: Vec<String>,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_window() -> usize { 10 }
fn default_output() -> String { "stats".into() }
fn default_computations() -> Vec<String> { vec!["AVG".into(), "MIN".into(), "MAX".into()] }

impl StatisticsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("StatisticsNode: {}", e)))?;
        if cfg.window_size == 0 {
            return Err(RuleEngineError::Config("StatisticsNode: windowSize must be > 0".into()));
        }
        let mut comps = Vec::new();
        for c in &cfg.computations {
            let comp = match c.to_uppercase().as_str() {
                "COUNT"    => StatComp::Count,
                "SUM"      => StatComp::Sum,
                "AVG"      => StatComp::Avg,
                "MIN"      => StatComp::Min,
                "MAX"      => StatComp::Max,
                "STD_DEV" | "STDDEV" => StatComp::StdDev,
                "VARIANCE" => StatComp::Variance,
                "MEDIAN"   => StatComp::Median,
                other      => return Err(RuleEngineError::Config(
                    format!("StatisticsNode: unknown computation '{}'", other))),
            };
            comps.push(comp);
        }
        Ok(Self {
            input_key: cfg.input_key,
            window_size: cfg.window_size,
            output_prefix: cfg.output_key,
            computations: comps,
            tell_failure_if_absent: cfg.tell_failure_if_absent,
            window: Arc::new(DashMap::new()),
        })
    }

    fn compute(values: &[f64], comp: StatComp) -> f64 {
        if values.is_empty() { return 0.0; }
        match comp {
            StatComp::Count    => values.len() as f64,
            StatComp::Sum      => values.iter().sum(),
            StatComp::Avg      => values.iter().sum::<f64>() / values.len() as f64,
            StatComp::Min      => values.iter().cloned().fold(f64::INFINITY, f64::min),
            StatComp::Max      => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            StatComp::Variance => {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
            }
            StatComp::StdDev => {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                var.sqrt()
            }
            StatComp::Median => {
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 {
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted[mid]
                }
            }
        }
    }

    fn stat_suffix(c: StatComp) -> &'static str {
        match c {
            StatComp::Count    => "count",
            StatComp::Sum      => "sum",
            StatComp::Avg      => "avg",
            StatComp::Min      => "min",
            StatComp::Max      => "max",
            StatComp::StdDev   => "std_dev",
            StatComp::Variance => "variance",
            StatComp::Median   => "median",
        }
    }
}

#[async_trait]
impl RuleNode for StatisticsNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = match serde_json::from_str(&msg.data) {
            Ok(v) => v,
            Err(_) => {
                let mut m = msg;
                m.metadata.insert("error".into(), "StatisticsNode: data is not JSON".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let current = match data.get(&self.input_key).and_then(|v| v.as_f64()) {
            Some(v) => v,
            None => {
                if self.tell_failure_if_absent {
                    let mut m = msg;
                    m.metadata.insert("error".into(),
                        format!("StatisticsNode: key '{}' not found or not numeric", self.input_key));
                    return Ok(vec![(RelationType::Failure, m)]);
                }
                return Ok(vec![(RelationType::Success, msg)]);
            }
        };

        // Update sliding window
        let values: Vec<f64> = {
            let mut entry = self.window
                .entry(msg.originator_id)
                .or_insert_with(VecDeque::new);
            entry.push_back(current);
            if entry.len() > self.window_size {
                entry.pop_front();
            }
            entry.iter().copied().collect()
        };

        // Compute and write results
        let mut out = msg;
        if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
            for comp in &self.computations {
                let result = Self::compute(&values, *comp);
                let key = format!("{}_{}", self.output_prefix, Self::stat_suffix(*comp));
                obj[key] = serde_json::json!(result);
            }
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
        let node = StatisticsNode::new(&json!({
            "inputKey": "temperature",
            "windowSize": 5,
            "computations": ["AVG", "MIN", "MAX", "STD_DEV"]
        })).unwrap();
        assert_eq!(node.window_size, 5);
        assert_eq!(node.computations.len(), 4);
    }

    #[test]
    fn zero_window_is_error() {
        assert!(StatisticsNode::new(&json!({
            "inputKey": "x",
            "windowSize": 0
        })).is_err());
    }

    #[test]
    fn stats_computation() {
        let vals = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(StatisticsNode::compute(&vals, StatComp::Avg), 30.0);
        assert_eq!(StatisticsNode::compute(&vals, StatComp::Min), 10.0);
        assert_eq!(StatisticsNode::compute(&vals, StatComp::Max), 50.0);
        assert_eq!(StatisticsNode::compute(&vals, StatComp::Median), 30.0);
        assert_eq!(StatisticsNode::compute(&vals, StatComp::Count), 5.0);
        assert_eq!(StatisticsNode::compute(&vals, StatComp::Sum), 150.0);
    }

    #[test]
    fn std_dev_computation() {
        // std_dev of [2, 4, 4, 4, 5, 5, 7, 9] = 2.0
        let vals = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std = StatisticsNode::compute(&vals, StatComp::StdDev);
        assert!((std - 2.0).abs() < 0.001);
    }
}
