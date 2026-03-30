use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Broadcast entity state updates across cluster nodes.
///
/// Mirrors `TbEntityStateSyncNode.java`. When an entity's state changes
/// (e.g., device goes ACTIVE → INACTIVE), this node ensures all cluster
/// nodes receive the update. The state is persisted to the KV store so
/// every node can query the latest entity state without cross-node RPC.
///
/// Config JSON:
/// ```json
/// {
///   "stateKey": "connectivityState",
///   "entityType": "DEVICE"
/// }
/// ```
pub struct EntityStateSyncNode {
    /// Metadata key that holds the state value to broadcast
    state_key:   String,
    /// Optional entity type filter (DEVICE, ASSET, etc.)
    entity_type: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "stateKey", default = "default_state_key")]
    state_key: String,
    #[serde(rename = "entityType")]
    entity_type: Option<String>,
}

fn default_state_key() -> String { "state".to_string() }

impl EntityStateSyncNode {
    pub fn new(config: &Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .unwrap_or(Config { state_key: default_state_key(), entity_type: None });
        Ok(Self {
            state_key:   cfg.state_key,
            entity_type: cfg.entity_type,
        })
    }

    /// Extract the state value from message metadata or body.
    fn extract_state(&self, msg: &TbMsg) -> String {
        // First check metadata
        if let Some(v) = msg.metadata.get(&self.state_key) {
            return v.clone();
        }
        // Fall back to message body JSON
        serde_json::from_str::<Value>(&msg.data)
            .ok()
            .and_then(|v| {
                v.get(&self.state_key)
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl RuleNode for EntityStateSyncNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let state_value = self.extract_state(&msg);

        // If entity_type filter is configured, skip entities of other types.
        if let Some(ref expected_type) = self.entity_type {
            if msg.originator_type != *expected_type {
                debug!(
                    entity_id            = %msg.originator_id,
                    entity_type          = %msg.originator_type,
                    expected_entity_type = %expected_type,
                    "Skipping entity state sync — type mismatch"
                );
                return Ok(vec![(RelationType::Success, msg)]);
            }
        }

        debug!(
            entity_id   = %msg.originator_id,
            entity_type = %msg.originator_type,
            state_key   = %self.state_key,
            state_value = %state_value,
            tenant_id   = %ctx.tenant_id,
            "Broadcasting entity state sync"
        );

        // Tag the message so the cluster/transport layer knows to broadcast this update.
        // In a full cluster setup, vl-cluster picks up this metadata and publishes
        // to the cluster event bus.
        let mut out = msg;
        out.metadata.insert("entityStateSynced".into(), "true".into());
        out.metadata.insert("entityStateSyncKey".into(), self.state_key.clone());
        out.metadata.insert("entityStateSyncValue".into(), state_value);
        out.metadata.insert(
            "entityStateSyncEntityId".into(),
            out.originator_id.to_string(),
        );
        out.metadata.insert(
            "entityStateSyncEntityType".into(),
            out.originator_type.clone(),
        );

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use vl_core::entities::TbMsg;

    #[test]
    fn test_default_config() {
        let node = EntityStateSyncNode::new(&json!({})).unwrap();
        assert_eq!(node.state_key, "state");
        assert!(node.entity_type.is_none());
    }

    #[test]
    fn test_custom_config() {
        let node = EntityStateSyncNode::new(&json!({
            "stateKey": "connectivityStatus",
            "entityType": "DEVICE"
        }))
        .unwrap();
        assert_eq!(node.state_key, "connectivityStatus");
        assert_eq!(node.entity_type.as_deref(), Some("DEVICE"));
    }

    #[test]
    fn test_extract_state_from_metadata() {
        use uuid::Uuid;

        let node = EntityStateSyncNode::new(&json!({"stateKey": "connectivity"})).unwrap();
        let mut msg = TbMsg::new("ACTIVITY_EVENT", Uuid::new_v4(), "DEVICE", "{}");
        msg.metadata.insert("connectivity".into(), "ACTIVE".into());
        assert_eq!(node.extract_state(&msg), "ACTIVE");
    }

    #[test]
    fn test_extract_state_from_body() {
        use uuid::Uuid;

        let node = EntityStateSyncNode::new(&json!({"stateKey": "status"})).unwrap();
        let msg = TbMsg::new(
            "POST_TELEMETRY_REQUEST",
            Uuid::new_v4(),
            "DEVICE",
            r#"{"status": "CONNECTED"}"#,
        );
        assert_eq!(node.extract_state(&msg), "CONNECTED");
    }
}
