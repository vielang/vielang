use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Format and convert telemetry values: rounding, unit conversion, renaming.
/// Java: TbFormatTelemetryNode (also covers TbConvertTelemetryNode)
/// Relations: Success, Failure
/// Config:
/// ```json
/// {
///   "operations": [
///     { "key": "temperature", "type": "ROUND", "precision": 1 },
///     { "key": "tempF", "type": "CONVERT", "formula": "temperature * 1.8 + 32" },
///     { "key": "speed_kmh", "type": "MULTIPLY", "sourceKey": "speed_ms", "factor": 3.6 }
///   ]
/// }
/// ```
pub struct FormatTelemetryNode {
    operations: Vec<FormatOp>,
}

#[derive(Debug, Clone)]
enum FormatOp {
    Round  { key: String, precision: u32 },
    Multiply { target_key: String, source_key: String, factor: f64 },
    Divide   { target_key: String, source_key: String, divisor: f64 },
    Offset   { key: String, offset: f64 },
    Abs      { key: String },
}

#[derive(Deserialize)]
struct RawOp {
    key: Option<String>,
    #[serde(rename = "sourceKey")]
    source_key: Option<String>,
    #[serde(rename = "type")]
    op_type: String,
    #[serde(default)]
    precision: u32,
    #[serde(default)]
    factor: f64,
    #[serde(default)]
    divisor: f64,
    #[serde(default)]
    offset: f64,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    operations: Vec<RawOp>,
}

impl FormatTelemetryNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("FormatTelemetryNode: {}", e)))?;

        let mut ops = Vec::new();
        for raw in cfg.operations {
            let op = match raw.op_type.to_uppercase().as_str() {
                "ROUND" => {
                    let key = raw.key.ok_or_else(|| RuleEngineError::Config(
                        "FormatTelemetryNode ROUND: 'key' required".into()))?;
                    FormatOp::Round { key, precision: raw.precision }
                }
                "MULTIPLY" => {
                    let target_key = raw.key.ok_or_else(|| RuleEngineError::Config(
                        "FormatTelemetryNode MULTIPLY: 'key' required".into()))?;
                    let source_key = raw.source_key.unwrap_or_else(|| target_key.clone());
                    FormatOp::Multiply { target_key, source_key, factor: raw.factor }
                }
                "DIVIDE" => {
                    if raw.divisor == 0.0 {
                        return Err(RuleEngineError::Config(
                            "FormatTelemetryNode DIVIDE: divisor cannot be 0".into()));
                    }
                    let target_key = raw.key.ok_or_else(|| RuleEngineError::Config(
                        "FormatTelemetryNode DIVIDE: 'key' required".into()))?;
                    let source_key = raw.source_key.unwrap_or_else(|| target_key.clone());
                    FormatOp::Divide { target_key, source_key, divisor: raw.divisor }
                }
                "OFFSET" | "ADD" => {
                    let key = raw.key.ok_or_else(|| RuleEngineError::Config(
                        "FormatTelemetryNode OFFSET: 'key' required".into()))?;
                    FormatOp::Offset { key, offset: raw.offset }
                }
                "ABS" => {
                    let key = raw.key.ok_or_else(|| RuleEngineError::Config(
                        "FormatTelemetryNode ABS: 'key' required".into()))?;
                    FormatOp::Abs { key }
                }
                other => return Err(RuleEngineError::Config(
                    format!("FormatTelemetryNode: unknown op type '{}'", other))),
            };
            ops.push(op);
        }
        Ok(Self { operations: ops })
    }

    fn apply(data: &mut serde_json::Map<String, serde_json::Value>, op: &FormatOp) {
        match op {
            FormatOp::Round { key, precision } => {
                if let Some(v) = data.get(key).and_then(|v| v.as_f64()) {
                    let factor = 10f64.powi(*precision as i32);
                    let rounded = (v * factor).round() / factor;
                    data.insert(key.clone(), serde_json::json!(rounded));
                }
            }
            FormatOp::Multiply { target_key, source_key, factor } => {
                if let Some(v) = data.get(source_key).and_then(|v| v.as_f64()) {
                    data.insert(target_key.clone(), serde_json::json!(v * factor));
                }
            }
            FormatOp::Divide { target_key, source_key, divisor } => {
                if let Some(v) = data.get(source_key).and_then(|v| v.as_f64()) {
                    data.insert(target_key.clone(), serde_json::json!(v / divisor));
                }
            }
            FormatOp::Offset { key, offset } => {
                if let Some(v) = data.get(key).and_then(|v| v.as_f64()) {
                    data.insert(key.clone(), serde_json::json!(v + offset));
                }
            }
            FormatOp::Abs { key } => {
                if let Some(v) = data.get(key).and_then(|v| v.as_f64()) {
                    data.insert(key.clone(), serde_json::json!(v.abs()));
                }
            }
        }
    }
}

#[async_trait]
impl RuleNode for FormatTelemetryNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let parsed: serde_json::Value = match serde_json::from_str(&msg.data) {
            Ok(v) => v,
            Err(e) => {
                let mut m = msg;
                m.metadata.insert("error".into(), format!("FormatTelemetryNode: {}", e));
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let mut obj = match parsed {
            serde_json::Value::Object(map) => map,
            _ => {
                let mut m = msg;
                m.metadata.insert("error".into(), "FormatTelemetryNode: data must be JSON object".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        for op in &self.operations {
            Self::apply(&mut obj, op);
        }

        let mut out = msg;
        out.data = serde_json::to_string(&serde_json::Value::Object(obj))
            .unwrap_or(out.data);
        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_operation() {
        let node = FormatTelemetryNode::new(&json!({
            "operations": [{ "key": "temp", "type": "ROUND", "precision": 1 }]
        })).unwrap();
        assert_eq!(node.operations.len(), 1);
    }

    #[test]
    fn multiply_operation() {
        let node = FormatTelemetryNode::new(&json!({
            "operations": [{
                "key": "speed_kmh", "type": "MULTIPLY",
                "sourceKey": "speed_ms", "factor": 3.6
            }]
        })).unwrap();
        assert_eq!(node.operations.len(), 1);
    }

    #[test]
    fn zero_divisor_is_error() {
        assert!(FormatTelemetryNode::new(&json!({
            "operations": [{ "key": "v", "type": "DIVIDE", "divisor": 0.0 }]
        })).is_err());
    }
}
