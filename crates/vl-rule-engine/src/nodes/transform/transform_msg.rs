use async_trait::async_trait;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
    script::RhaiEngine,
};

/// Transform message data/metadata via Rhai script.
/// Script has access to: `msg` (JSON string), `msgType` (string), `metadata` (Map)
/// Script must return a string (new JSON data).
/// Config JSON:
/// ```json
/// {
///   "jsScript": "let m = parse_json(msg); m[\"temp_f\"] = m[\"temp_c\"] * 9.0/5.0 + 32.0; to_json(m)"
/// }
/// ```
/// Also accepts `"rhaiScript"` key (legacy alias).
pub struct TransformMsgNode {
    script: String,
    engine: RhaiEngine,
}

impl TransformMsgNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        // Accept jsScript (ThingsBoard-compatible) or rhaiScript (legacy alias)
        let script = config["jsScript"].as_str()
            .or_else(|| config["rhaiScript"].as_str())
            .ok_or_else(|| RuleEngineError::Config("TransformMsgNode: jsScript required".into()))?
            .to_string();
        Ok(Self { script, engine: RhaiEngine::new() })
    }
}

#[async_trait]
impl RuleNode for TransformMsgNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        match self.engine.run_transform(&self.script, &msg) {
            Ok(new_data) => {
                let mut out = msg;
                out.data = new_data;
                Ok(vec![(RelationType::Success, out)])
            }
            Err(e) => {
                let mut out = msg;
                out.metadata.insert("error".into(), e.to_string());
                Ok(vec![(RelationType::Failure, out)])
            }
        }
    }
}
