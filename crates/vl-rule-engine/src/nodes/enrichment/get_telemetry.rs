use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Fetch latest telemetry values of originator and enrich msg metadata or data.
/// Java: TbGetTelemetryNode
/// Config:
/// ```json
/// {
///   "latestTsKeyNames": ["temperature", "humidity"],
///   "fetchTo": "METADATA"
/// }
/// ```
pub struct GetTelemetryNode {
    key_names: Vec<String>,
    fetch_to_metadata: bool,
    tell_failure_if_absent: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "latestTsKeyNames", default)]
    latest_ts_key_names: Vec<String>,
    #[serde(rename = "fetchTo", default = "default_fetch_to")]
    fetch_to: String,
    #[serde(rename = "tellFailureIfAbsent", default)]
    tell_failure_if_absent: bool,
}

fn default_fetch_to() -> String { "METADATA".into() }

impl GetTelemetryNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetTelemetryNode: {}", e)))?;
        Ok(Self {
            key_names: cfg.latest_ts_key_names,
            fetch_to_metadata: cfg.fetch_to != "DATA",
            tell_failure_if_absent: cfg.tell_failure_if_absent,
        })
    }
}

#[async_trait]
impl RuleNode for GetTelemetryNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let key_ids = ctx.dao.kv.lookup_key_ids(&self.key_names).await?;

        let mut out = msg;
        let mut missing = false;

        for name in &self.key_names {
            if let Some(&key_id) = key_ids.get(name) {
                let entries = ctx.dao.kv.find_latest(out.originator_id, &[key_id]).await?;
                if let Some(e) = entries.into_iter().next() {
                    let val = ts_val_to_string(&e);
                    if self.fetch_to_metadata {
                        out.metadata.insert(name.clone(), val);
                    } else {
                        if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
                            obj[name] = serde_json::Value::String(val);
                            out.data = serde_json::to_string(&obj).unwrap_or(out.data);
                        }
                    }
                } else {
                    missing = true;
                }
            } else {
                missing = true;
            }
        }

        if missing && self.tell_failure_if_absent {
            Ok(vec![(RelationType::Failure, out)])
        } else {
            Ok(vec![(RelationType::Success, out)])
        }
    }
}

fn ts_val_to_string(e: &vl_core::entities::TsKvEntry) -> String {
    if let Some(v) = e.bool_v  { return v.to_string(); }
    if let Some(v) = e.long_v  { return v.to_string(); }
    if let Some(v) = e.dbl_v   { return v.to_string(); }
    if let Some(ref v) = e.str_v  { return v.clone(); }
    if let Some(ref v) = e.json_v { return v.to_string(); }
    String::new()
}
