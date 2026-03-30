use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Perform arithmetic on message fields and put result into output key.
/// Java: TbMathNode
/// Config:
/// ```json
/// {
///   "operation": "ADD",
///   "leftOperand": { "type": "MSG_BODY", "key": "temperature" },
///   "rightOperand": { "type": "CONSTANT", "value": 10.0 },
///   "result": { "type": "MSG_BODY", "key": "adjustedTemp" }
/// }
/// ```
pub struct MathNode {
    operation: String,
    left: Operand,
    right: Operand,
    result_key: String,
    result_type: String,
}

#[derive(Deserialize, Clone)]
struct Operand {
    #[serde(rename = "type")]
    operand_type: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    value: f64,
}

#[derive(Deserialize)]
struct Config {
    operation: String,
    #[serde(rename = "leftOperand")]
    left_operand: Operand,
    #[serde(rename = "rightOperand")]
    right_operand: Operand,
    #[serde(rename = "result")]
    result: ResultDef,
}

#[derive(Deserialize)]
struct ResultDef {
    #[serde(rename = "type", default = "default_msg_body")]
    result_type: String,
    #[serde(default)]
    key: String,
}

fn default_msg_body() -> String { "MSG_BODY".into() }

impl MathNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("MathNode: {}", e)))?;
        Ok(Self {
            operation: cfg.operation,
            left: cfg.left_operand,
            right: cfg.right_operand,
            result_key: cfg.result.key,
            result_type: cfg.result.result_type,
        })
    }
}

#[async_trait]
impl RuleNode for MathNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = serde_json::from_str(&msg.data).unwrap_or(serde_json::json!({}));

        let left = resolve_operand(&self.left, &data, &msg.metadata);
        let right = resolve_operand(&self.right, &data, &msg.metadata);

        let result = match self.operation.as_str() {
            "ADD"      => left + right,
            "SUBTRACT" => left - right,
            "MULTIPLY" => left * right,
            "DIVIDE"   => if right != 0.0 { left / right } else { f64::NAN },
            "MODULO"   => left % right,
            "POWER"    => left.powf(right),
            _          => left,
        };

        let mut out = msg;
        match self.result_type.as_str() {
            "METADATA" => {
                out.metadata.insert(self.result_key.clone(), result.to_string());
            }
            _ => {
                // MSG_BODY
                if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                    obj[&self.result_key] = serde_json::json!(result);
                    out.data = serde_json::to_string(&obj).unwrap_or(out.data);
                }
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

fn resolve_operand(op: &Operand, data: &serde_json::Value, metadata: &std::collections::HashMap<String, String>) -> f64 {
    match op.operand_type.as_str() {
        "CONSTANT" => op.value,
        "METADATA" => metadata.get(&op.key)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0),
        _ => {
            // MSG_BODY
            data.get(&op.key)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        }
    }
}
