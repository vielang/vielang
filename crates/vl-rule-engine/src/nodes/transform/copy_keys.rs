use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Copy keys between message data (JSON) and metadata.
/// Config JSON:
/// ```json
/// {
///   "keys": ["temperature", "humidity"],
///   "fromMetadata": false
/// }
/// ```
/// fromMetadata=false: data → metadata
/// fromMetadata=true:  metadata → data
pub struct CopyKeysNode {
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

impl CopyKeysNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CopyKeysNode: {}", e)))?;
        Ok(Self { keys: cfg.keys, from_metadata: cfg.from_metadata })
    }
}

#[async_trait]
impl RuleNode for CopyKeysNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        if self.from_metadata {
            // metadata → data
            let mut data: serde_json::Value = serde_json::from_str(&out.data)
                .unwrap_or(serde_json::Value::Object(Default::default()));
            for key in &self.keys {
                if let Some(val) = out.metadata.get(key) {
                    data[key] = serde_json::Value::String(val.clone());
                }
            }
            out.data = serde_json::to_string(&data).unwrap_or(out.data);
        } else {
            // data → metadata
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&out.data) {
                for key in &self.keys {
                    if let Some(val) = data.get(key) {
                        out.metadata.insert(key.clone(), val.to_string().trim_matches('"').to_string());
                    }
                }
            }
        }

        Ok(vec![(RelationType::Success, out)])
    }
}
