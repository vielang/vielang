use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_core::entities::AttributeScope;
use vl_dao::postgres::kv::KvDao;

/// Handles ATTRIBUTE KV sync between cloud and edge.
///
/// Attributes are key-value pairs attached to entities (scope: CLIENT_SCOPE,
/// SERVER_SCOPE, SHARED_SCOPE). Edge gateways sync shared attributes bidirectionally.
pub struct AttributeProcessor {
    kv_dao: Arc<KvDao>,
}

impl AttributeProcessor {
    pub fn new(kv_dao: Arc<KvDao>) -> Self {
        Self { kv_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for AttributeProcessor {
    fn entity_type(&self) -> &'static str {
        "ATTRIBUTE"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "ATTRIBUTE",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                // Fetch shared-scope attributes for this entity to push to edge.
                let attributes = self.kv_dao
                    .find_attributes(entity_id, AttributeScope::SharedScope, None)
                    .await
                    .map_err(|e| format!("DAO error fetching attributes: {}", e))?;

                let body = serde_json::to_value(&attributes)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "ATTRIBUTE",
                    "entityId": entity_id.to_string(),
                    "action": action,
                    "body": body,
                }))
            }
        }
    }

    async fn process_uplink(
        &self,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        let attr: vl_core::entities::AttributeKvEntry = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize attribute from edge: {}", e))?;

        self.kv_dao
            .save_attribute(&attr)
            .await
            .map_err(|e| {
                warn!(entity_id = %attr.entity_id, key = %attr.attribute_key, "Failed to upsert attribute from edge: {}", e);
                format!("Failed to save attribute: {}", e)
            })?;

        Ok(())
    }
}
