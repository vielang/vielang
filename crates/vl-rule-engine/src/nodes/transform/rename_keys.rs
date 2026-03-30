use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Rename keys in message data (JSON) or metadata.
/// Config JSON:
/// ```json
/// {
///   "renameMap": { "temp": "temperature", "hum": "humidity" },
///   "fromMetadata": false
/// }
/// ```
pub struct RenameKeysNode {
    rename_map:    HashMap<String, String>,
    from_metadata: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "renameMap", default)]
    rename_map: HashMap<String, String>,
    #[serde(rename = "fromMetadata", default)]
    from_metadata: bool,
}

impl RenameKeysNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("RenameKeysNode: {}", e)))?;
        Ok(Self { rename_map: cfg.rename_map, from_metadata: cfg.from_metadata })
    }
}

#[async_trait]
impl RuleNode for RenameKeysNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        if self.from_metadata {
            for (old_key, new_key) in &self.rename_map {
                if let Some(val) = out.metadata.remove(old_key) {
                    out.metadata.insert(new_key.clone(), val);
                }
            }
        } else {
            if let Ok(mut data) = serde_json::from_str::<serde_json::Value>(&out.data) {
                if let Some(obj) = data.as_object_mut() {
                    for (old_key, new_key) in &self.rename_map {
                        if let Some(val) = obj.remove(old_key) {
                            obj.insert(new_key.clone(), val);
                        }
                    }
                }
                out.data = serde_json::to_string(&data).unwrap_or(out.data);
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
