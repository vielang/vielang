use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Trigger recalculation of all calculated fields for the message originator.
///
/// Mirrors `TbCalculatedFieldsSyncNode.java`. When telemetry or attributes change,
/// this node ensures any derived/calculated fields are updated to reflect the
/// new values. It marks the message with a sync metadata key so that downstream
/// handlers or the rule engine can enqueue recalculation jobs.
///
/// Config JSON:
/// ```json
/// { "failOnError": false }
/// ```
pub struct CalculatedFieldsSyncNode {
    fail_on_error: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "failOnError", default)]
    fail_on_error: bool,
}

impl CalculatedFieldsSyncNode {
    pub fn new(config: &Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .unwrap_or(Config { fail_on_error: false });
        Ok(Self { fail_on_error: cfg.fail_on_error })
    }
}

#[async_trait]
impl RuleNode for CalculatedFieldsSyncNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        debug!(
            originator_id   = %msg.originator_id,
            originator_type = %msg.originator_type,
            tenant_id       = %ctx.tenant_id,
            "Triggering calculated fields sync"
        );

        // Tag the message so the rule engine / downstream handlers know
        // to enqueue a recalculation job for this entity's calculated fields.
        let mut out = msg;
        out.metadata.insert(
            "calculatedFieldsSync".into(),
            "true".into(),
        );
        out.metadata.insert(
            "calculatedFieldsSyncEntityId".into(),
            out.originator_id.to_string(),
        );
        out.metadata.insert(
            "calculatedFieldsSyncEntityType".into(),
            out.originator_type.clone(),
        );

        // In a full implementation, calculated field recalculation errors would be
        // caught here and routed to Failure when fail_on_error is true.
        // For now, we always succeed after tagging the message.
        let relation = if self.fail_on_error {
            RelationType::Success // fail_on_error applies to actual recalc errors; tagging always succeeds
        } else {
            RelationType::Success
        };

        Ok(vec![(relation, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let node = CalculatedFieldsSyncNode::new(&json!({})).unwrap();
        assert!(!node.fail_on_error);
    }

    #[test]
    fn test_fail_on_error_config() {
        let node = CalculatedFieldsSyncNode::new(&json!({"failOnError": true})).unwrap();
        assert!(node.fail_on_error);
    }
}
