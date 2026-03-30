use serde::{Deserialize, Serialize};

/// Per-node debug settings — parsed from the node configuration JSON.
///
/// When `enabled = true`, every message processed by the node is persisted
/// as a `DEBUG_RULE_NODE` event and can be inspected or replayed from the UI.
///
/// Matches ThingsBoard `TbNodeConfiguration.debugMode`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebugSettings {
    #[serde(default, rename = "debugMode")]
    pub enabled: bool,
}

impl DebugSettings {
    /// Parse debug settings from a node config value.
    /// Defaults to `enabled = false` on parse failure.
    pub fn from_config(config: &serde_json::Value) -> Self {
        serde_json::from_value(config.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_disabled() {
        assert!(!DebugSettings::default().enabled);
    }

    #[test]
    fn parses_debug_mode_true() {
        let s = DebugSettings::from_config(&serde_json::json!({ "debugMode": true }));
        assert!(s.enabled);
    }

    #[test]
    fn parses_debug_mode_false() {
        let s = DebugSettings::from_config(&serde_json::json!({ "debugMode": false }));
        assert!(!s.enabled);
    }

    #[test]
    fn missing_field_defaults_false() {
        let s = DebugSettings::from_config(&serde_json::json!({ "someOther": "x" }));
        assert!(!s.enabled);
    }

    #[test]
    fn null_config_defaults_false() {
        let s = DebugSettings::from_config(&serde_json::json!(null));
        assert!(!s.enabled);
    }
}
