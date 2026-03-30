use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Delay message processing by N seconds.
/// Java: TbMsgDelayNode
/// Config: `{ "periodInSeconds": 5, "maxPendingMsgs": 1000 }`
pub struct MsgDelayNode {
    period_ms: u64,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "periodInSeconds", default)]
    period_in_seconds: u64,
}

impl MsgDelayNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("MsgDelayNode: {}", e)))?;
        Ok(Self { period_ms: cfg.period_in_seconds * 1000 })
    }
}

#[async_trait]
impl RuleNode for MsgDelayNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.period_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.period_ms)).await;
        }
        Ok(vec![(RelationType::Success, msg)])
    }
}
