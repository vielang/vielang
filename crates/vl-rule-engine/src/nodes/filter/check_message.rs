use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Filter by checking message fields/metadata against conditions.
/// All conditions must pass → True, else False.
/// Java: TbCheckMessageNode
/// Config:
/// ```json
/// {
///   "conditions": [
///     { "key": "temperature", "type": "MSG_FIELD", "operation": "GREATER", "value": "30" }
///   ],
///   "checkAllConditions": true
/// }
/// ```
pub struct CheckMessageNode {
    conditions: Vec<Condition>,
    check_all: bool,
}

#[derive(Deserialize)]
struct Condition {
    key: String,
    #[serde(rename = "type", default = "default_key_type")]
    key_type: String,
    operation: String,
    #[serde(default)]
    value: String,
}

fn default_key_type() -> String { "MSG_FIELD".into() }

impl CheckMessageNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let conditions: Vec<Condition> = config["conditions"]
            .as_array()
            .ok_or_else(|| RuleEngineError::Config("conditions array required".into()))?
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
        let check_all = config["checkAllConditions"].as_bool().unwrap_or(true);
        Ok(Self { conditions, check_all })
    }
}

#[async_trait]
impl RuleNode for CheckMessageNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = serde_json::from_str(&msg.data).unwrap_or(serde_json::json!({}));

        let mut results = Vec::new();
        for cond in &self.conditions {
            let actual = match cond.key_type.as_str() {
                "METADATA" => msg.metadata.get(&cond.key).cloned().unwrap_or_default(),
                _ => {
                    // MSG_FIELD — look in data JSON
                    data.get(&cond.key)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default()
                }
            };
            let passed = check_operation(&actual, &cond.operation, &cond.value);
            results.push(passed);
        }

        let matched = if self.check_all {
            results.iter().all(|&b| b)
        } else {
            results.iter().any(|&b| b)
        };

        let relation = if matched { RelationType::True } else { RelationType::False };
        Ok(vec![(relation, msg)])
    }
}

fn check_operation(actual: &str, operation: &str, expected: &str) -> bool {
    match operation {
        "EQUAL"           => actual == expected,
        "NOT_EQUAL"       => actual != expected,
        "CONTAINS"        => actual.contains(expected),
        "NOT_CONTAINS"    => !actual.contains(expected),
        "STARTS_WITH"     => actual.starts_with(expected),
        "ENDS_WITH"       => actual.ends_with(expected),
        "IS_EMPTY"        => actual.is_empty(),
        "IS_NOT_EMPTY"    => !actual.is_empty(),
        "GREATER"         => {
            let a = actual.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            let e = expected.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            a > e
        }
        "LESS"            => {
            let a = actual.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            let e = expected.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            a < e
        }
        "GREATER_OR_EQUAL" => {
            let a = actual.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            let e = expected.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            a >= e
        }
        "LESS_OR_EQUAL"   => {
            let a = actual.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            let e = expected.parse::<f64>().unwrap_or(f64::NEG_INFINITY);
            a <= e
        }
        _ => false,
    }
}
