use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Evaluate a mathematical expression using message data/metadata variables.
/// Java: TbRuleEngineCalculatedFieldNode
/// Uses Rhai scripting engine (sandboxed, already a dependency).
/// Relations: Success, Failure (evaluation error)
/// Config:
/// ```json
/// {
///   "expression": "temperature * 1.8 + 32",
///   "outputKey": "temperatureF",
///   "outputTarget": "MSG_BODY",     // MSG_BODY | METADATA
///   "variablesMapping": {           // optional: rename vars before evaluation
///     "temperature": "temp"         // expression uses "temp", data has "temperature"
///   },
///   "round": 2
/// }
/// ```
/// Variables available in expression:
///   - All keys from msg.data JSON object (numeric values)
///   - All keys from msg.metadata (numeric values)
///   - Built-in: abs(x), sqrt(x), pow(x,y), floor(x), ceil(x), round(x)
pub struct CalculatedFieldNode {
    expression:       String,
    output_key:       String,
    to_metadata:      bool,
    var_mapping:      std::collections::HashMap<String, String>,
    precision:        Option<u32>,
}

#[derive(Deserialize)]
struct Config {
    expression: String,
    #[serde(rename = "outputKey")]
    output_key: String,
    #[serde(rename = "outputTarget", default = "default_body")]
    output_target: String,
    #[serde(rename = "variablesMapping", default)]
    variables_mapping: std::collections::HashMap<String, String>,
    #[serde(rename = "round")]
    round: Option<u32>,
}

fn default_body() -> String { "MSG_BODY".into() }

impl CalculatedFieldNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CalculatedFieldNode: {}", e)))?;
        Ok(Self {
            expression: cfg.expression,
            output_key: cfg.output_key,
            to_metadata: cfg.output_target.to_uppercase() == "METADATA",
            var_mapping: cfg.variables_mapping,
            precision: cfg.round,
        })
    }

    fn round(val: f64, precision: u32) -> f64 {
        let factor = 10f64.powi(precision as i32);
        (val * factor).round() / factor
    }
}

#[async_trait]
impl RuleNode for CalculatedFieldNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = serde_json::from_str(&msg.data)
            .unwrap_or(serde_json::json!({}));

        // Build Rhai engine and scope
        let engine = rhai::Engine::new();
        let mut scope = rhai::Scope::new();

        // Push msg body numeric values into scope
        if let Some(obj) = data.as_object() {
            for (k, v) in obj {
                let var_name = self.var_mapping
                    .iter()
                    .find(|(_, mapped)| *mapped == k)
                    .map(|(orig, _)| orig.as_str())
                    .unwrap_or(k.as_str());
                if let Some(n) = v.as_f64() {
                    scope.push(var_name.to_string(), n);
                } else if let Some(b) = v.as_bool() {
                    scope.push(var_name.to_string(), b);
                }
            }
        }

        // Push metadata numeric values (lower priority than data)
        for (k, v) in &msg.metadata {
            if !scope.contains(k.as_str()) {
                if let Ok(n) = v.parse::<f64>() {
                    scope.push(k.clone(), n);
                }
            }
        }

        let result: f64 = match engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &self.expression) {
            Ok(val) => {
                if let Some(n) = val.as_float().ok().or_else(|| val.as_int().ok().map(|i| i as f64)) {
                    n
                } else {
                    let mut m = msg;
                    m.metadata.insert("error".into(),
                        "CalculatedFieldNode: expression did not return a number".into());
                    return Ok(vec![(RelationType::Failure, m)]);
                }
            }
            Err(e) => {
                let mut m = msg;
                m.metadata.insert("error".into(), format!("CalculatedFieldNode: {}", e));
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let output = if let Some(p) = self.precision {
            Self::round(result, p)
        } else {
            result
        };

        let mut out = msg;
        if self.to_metadata {
            out.metadata.insert(self.output_key.clone(), output.to_string());
        } else if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
            obj[&self.output_key] = serde_json::json!(output);
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
        let node = CalculatedFieldNode::new(&json!({
            "expression": "temperature * 1.8 + 32",
            "outputKey": "temperatureF",
            "round": 1
        })).unwrap();
        assert_eq!(node.output_key, "temperatureF");
        assert_eq!(node.precision, Some(1));
        assert!(!node.to_metadata);
    }

    #[test]
    fn missing_expression_is_error() {
        assert!(CalculatedFieldNode::new(&json!({
            "outputKey": "result"
        })).is_err());
    }

    #[test]
    fn rounding() {
        assert_eq!(CalculatedFieldNode::round(22.5 * 1.8 + 32.0, 1), 72.5);
    }
}
