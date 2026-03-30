use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Count incoming messages and emit a count message every `intervalMs` ms.
/// Java: TbMsgCountNode
/// Config:
/// ```json
/// { "intervalMs": 1000, "countKey": "messageCount" }
/// ```
pub struct MsgCountNode {
    interval_ms: i64,
    count_key: String,
    counter: Arc<AtomicI64>,
    last_emit: Arc<AtomicI64>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "intervalMs", default = "default_interval")]
    interval_ms: i64,
    #[serde(rename = "countKey", default = "default_count_key")]
    count_key: String,
}

fn default_interval() -> i64 { 1000 }
fn default_count_key() -> String { "messageCount".into() }

impl MsgCountNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("MsgCountNode: {}", e)))?;
        let now = chrono::Utc::now().timestamp_millis();
        Ok(Self {
            interval_ms: cfg.interval_ms,
            count_key: cfg.count_key,
            counter: Arc::new(AtomicI64::new(0)),
            last_emit: Arc::new(AtomicI64::new(now)),
        })
    }
}

#[async_trait]
impl RuleNode for MsgCountNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        let now = chrono::Utc::now().timestamp_millis();
        let last = self.last_emit.load(Ordering::Relaxed);

        if now - last >= self.interval_ms {
            // Reset counter and last_emit
            self.counter.store(0, Ordering::Relaxed);
            self.last_emit.store(now, Ordering::Relaxed);

            // Emit count message
            let mut out = msg;
            out.metadata.insert(self.count_key.clone(), count.to_string());
            out.data = serde_json::json!({ &self.count_key: count }).to_string();
            Ok(vec![(RelationType::Success, out)])
        } else {
            // Within window — pass original message with count in metadata
            let mut out = msg;
            out.metadata.insert(self.count_key.clone(), count.to_string());
            Ok(vec![(RelationType::Success, out)])
        }
    }
}
