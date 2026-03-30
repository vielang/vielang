use async_trait::async_trait;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};
use vl_core::entities::TbMsg;

/// Filter node that compares a numeric value from msg body to a threshold.
/// Config: { "key": "temperature", "op": "GREATER_THAN", "threshold": 35.0 }
/// Supported ops: GREATER_THAN, GREATER_OR_EQUAL, LESS_THAN, LESS_OR_EQUAL, EQUAL, NOT_EQUAL
/// Relations: True (passes), False (fails), Failure (key missing or not numeric)
pub struct ThresholdFilterNode {
    key: String,
    op: ThresholdOp,
    threshold: f64,
}

#[derive(Debug, Clone, Copy)]
enum ThresholdOp {
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Equal,
    NotEqual,
}

impl ThresholdOp {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "GREATER_THAN"     => Some(Self::GreaterThan),
            "GREATER_OR_EQUAL" => Some(Self::GreaterOrEqual),
            "LESS_THAN"        => Some(Self::LessThan),
            "LESS_OR_EQUAL"    => Some(Self::LessOrEqual),
            "EQUAL"            => Some(Self::Equal),
            "NOT_EQUAL"        => Some(Self::NotEqual),
            _ => None,
        }
    }

    fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan     => value > threshold,
            Self::GreaterOrEqual  => value >= threshold,
            Self::LessThan        => value < threshold,
            Self::LessOrEqual     => value <= threshold,
            Self::Equal           => (value - threshold).abs() < f64::EPSILON,
            Self::NotEqual        => (value - threshold).abs() >= f64::EPSILON,
        }
    }
}

impl ThresholdFilterNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let key = config["key"]
            .as_str()
            .ok_or_else(|| RuleEngineError::Config("ThresholdFilter: 'key' required".into()))?
            .to_string();
        let op_str = config["op"]
            .as_str()
            .ok_or_else(|| RuleEngineError::Config("ThresholdFilter: 'op' required".into()))?;
        let op = ThresholdOp::from_str(op_str)
            .ok_or_else(|| RuleEngineError::Config(format!("ThresholdFilter: unknown op '{}'", op_str)))?;
        let threshold = config["threshold"]
            .as_f64()
            .ok_or_else(|| RuleEngineError::Config("ThresholdFilter: 'threshold' must be numeric".into()))?;
        Ok(Self { key, op, threshold })
    }
}

#[async_trait]
impl RuleNode for ThresholdFilterNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let data: serde_json::Value = match serde_json::from_str(&msg.data) {
            Ok(v) => v,
            Err(_) => {
                let mut m = msg;
                m.metadata.insert("error".into(), "ThresholdFilter: msg data is not valid JSON".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let value = match data.get(&self.key).and_then(|v| v.as_f64()) {
            Some(v) => v,
            None => {
                let mut m = msg;
                m.metadata.insert("error".into(), format!("ThresholdFilter: key '{}' not found or not numeric", self.key));
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        if self.op.evaluate(value, self.threshold) {
            Ok(vec![(RelationType::True, msg)])
        } else {
            Ok(vec![(RelationType::False, msg)])
        }
    }
}
