use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Delete keys from message data (JSON) or metadata.
/// Config JSON:
/// ```json
/// {
///   "keys": ["sensitiveField"],
///   "fromMetadata": false
/// }
/// ```
pub struct DeleteKeysNode {
    keys:          Vec<String>,
    from_metadata: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    keys: Vec<String>,
    #[serde(rename = "fromMetadata", default)]
    from_metadata: bool,
}

impl DeleteKeysNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("DeleteKeysNode: {}", e)))?;
        Ok(Self { keys: cfg.keys, from_metadata: cfg.from_metadata })
    }
}

#[async_trait]
impl RuleNode for DeleteKeysNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        if self.from_metadata {
            for key in &self.keys {
                out.metadata.remove(key);
            }
        } else {
            if let Ok(mut data) = serde_json::from_str::<serde_json::Value>(&out.data) {
                if let Some(obj) = data.as_object_mut() {
                    for key in &self.keys {
                        obj.remove(key);
                    }
                }
                out.data = serde_json::to_string(&data).unwrap_or(out.data);
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
