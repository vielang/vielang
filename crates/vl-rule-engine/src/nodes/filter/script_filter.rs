use async_trait::async_trait;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
    script::RhaiEngine,
};
use vl_core::entities::TbMsg;

/// Filter via Rhai script returning bool.
/// Config: `{ "jsScript": "msgType == \"POST_TELEMETRY_REQUEST\"" }`
/// Also accepts `"rhaiScript"` key (legacy alias).
pub struct ScriptFilter {
    script: String,
    engine: RhaiEngine,
}

impl ScriptFilter {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        // Accept jsScript (ThingsBoard-compatible) or rhaiScript (legacy alias)
        let script = config["jsScript"].as_str()
            .or_else(|| config["rhaiScript"].as_str())
            .ok_or_else(|| RuleEngineError::Config("jsScript required".into()))?
            .to_string();
        Ok(Self { script, engine: RhaiEngine::new() })
    }
}

#[async_trait]
impl RuleNode for ScriptFilter {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        match self.engine.run_filter(&self.script, &msg) {
            Ok(true)  => Ok(vec![(RelationType::True, msg)]),
            Ok(false) => Ok(vec![(RelationType::False, msg)]),
            Err(e) => {
                let mut m = msg;
                m.metadata.insert("error".into(), e.to_string());
                Ok(vec![(RelationType::Failure, m)])
            }
        }
    }
}
