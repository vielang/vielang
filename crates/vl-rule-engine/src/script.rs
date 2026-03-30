use rhai::{Engine, Scope};
use vl_core::entities::TbMsg;
use crate::error::RuleEngineError;

/// Sandboxed Rhai scripting engine for rule nodes.
/// The `sync` rhai feature makes Engine: Send + Sync.
pub struct RhaiEngine {
    engine: Engine,
}

impl RhaiEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(10_000);
        engine.set_max_call_levels(32);
        // Disable file/module access
        engine.set_max_string_size(65_536);
        Self { engine }
    }

    /// Run a boolean filter script.
    /// Variables available: `msg` (String), `msgType` (String), `metadata` (Map<String,String>)
    pub fn run_filter(&self, script: &str, msg: &TbMsg) -> Result<bool, RuleEngineError> {
        let mut scope = Scope::new();
        scope.push("msg",      msg.data.clone());
        scope.push("msgType",  msg.msg_type.clone());
        scope.push("metadata", build_metadata_map(msg));
        self.engine
            .eval_with_scope::<bool>(&mut scope, script)
            .map_err(|e| RuleEngineError::Script(e.to_string()))
    }

    /// Run a transform script that returns a new JSON string.
    /// Variables available: `msg` (String), `msgType` (String), `metadata` (Map<String,String>)
    pub fn run_transform(&self, script: &str, msg: &TbMsg) -> Result<String, RuleEngineError> {
        let mut scope = Scope::new();
        scope.push("msg",      msg.data.clone());
        scope.push("msgType",  msg.msg_type.clone());
        scope.push("metadata", build_metadata_map(msg));
        self.engine
            .eval_with_scope::<String>(&mut scope, script)
            .map_err(|e| RuleEngineError::Script(e.to_string()))
    }

    /// Run a log script that returns a String (log message).
    /// Variables available: `msg` (String), `msgType` (String), `metadata` (Map<String,String>)
    pub fn run_log(&self, script: &str, msg: &TbMsg) -> Result<String, RuleEngineError> {
        let mut scope = Scope::new();
        scope.push("msg",      msg.data.clone());
        scope.push("msgType",  msg.msg_type.clone());
        scope.push("metadata", build_metadata_map(msg));
        self.engine
            .eval_with_scope::<String>(&mut scope, script)
            .map_err(|e| RuleEngineError::Script(e.to_string()))
    }

    /// Run an alarm condition script that returns bool.
    /// Variables available: `msg` (String), `msgType` (String), `metadata` (Map<String,String>)
    pub fn run_condition(&self, script: &str, msg: &TbMsg) -> Result<bool, RuleEngineError> {
        let mut scope = Scope::new();
        scope.push("msg",      msg.data.clone());
        scope.push("msgType",  msg.msg_type.clone());
        scope.push("metadata", build_metadata_map(msg));
        self.engine
            .eval_with_scope::<bool>(&mut scope, script)
            .map_err(|e| RuleEngineError::Script(e.to_string()))
    }
}

impl Default for RhaiEngine {
    fn default() -> Self { Self::new() }
}

fn build_metadata_map(msg: &TbMsg) -> rhai::Map {
    msg.metadata
        .iter()
        .map(|(k, v)| (k.clone().into(), rhai::Dynamic::from(v.clone())))
        .collect()
}
