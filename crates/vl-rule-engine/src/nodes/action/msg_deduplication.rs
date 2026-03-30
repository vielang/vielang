use async_trait::async_trait;
use std::sync::Arc;
use serde::Deserialize;
use dashmap::DashMap;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Deduplicate messages within a time window.
/// If an identical message (same originator + data hash) arrives within `intervalMs`, it is dropped.
/// Java: TbMsgDeduplicationNode
/// Config:
/// ```json
/// { "intervalMs": 5000, "strategy": "FIRST" }
/// ```
pub struct MsgDeduplicationNode {
    interval_ms: i64,
    /// "FIRST" keeps first, "LAST" keeps last
    strategy: String,
    seen: Arc<DashMap<String, i64>>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "intervalMs", default = "default_interval")]
    interval_ms: i64,
    #[serde(default = "default_strategy")]
    strategy: String,
}

fn default_interval() -> i64 { 5000 }
fn default_strategy() -> String { "FIRST".into() }

impl MsgDeduplicationNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("MsgDeduplicationNode: {}", e)))?;
        Ok(Self {
            interval_ms: cfg.interval_ms,
            strategy: cfg.strategy,
            seen: Arc::new(DashMap::new()),
        })
    }

    fn dedup_key(msg: &TbMsg) -> String {
        // key = originator_id + hash of data
        let hash = simple_hash(&msg.data);
        format!("{}:{}", msg.originator_id, hash)
    }
}

#[async_trait]
impl RuleNode for MsgDeduplicationNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let now = chrono::Utc::now().timestamp_millis();
        let key = Self::dedup_key(&msg);

        // Evict old entries
        self.seen.retain(|_, &mut ts| now - ts < self.interval_ms);

        if self.strategy == "LAST" {
            // Always update, always pass
            self.seen.insert(key, now);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        // FIRST strategy: drop if seen within window
        if let Some(entry) = self.seen.get(&key) {
            if now - *entry < self.interval_ms {
                // Duplicate — drop
                return Ok(vec![]);
            }
        }
        self.seen.insert(key, now);
        Ok(vec![(RelationType::Success, msg)])
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}
