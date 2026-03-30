use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::asset::AssetDao;

/// Handles ASSET entity sync between cloud and edge.
pub struct AssetProcessor {
    asset_dao: Arc<AssetDao>,
}

impl AssetProcessor {
    pub fn new(asset_dao: Arc<AssetDao>) -> Self {
        Self { asset_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for AssetProcessor {
    fn entity_type(&self) -> &'static str {
        "ASSET"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "ASSET",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                let asset = self.asset_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching asset: {}", e))?
                    .ok_or_else(|| format!("Asset not found: {}", entity_id))?;

                let body = serde_json::to_value(&asset)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "ASSET",
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
        let asset: vl_core::entities::Asset = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize asset from edge: {}", e))?;

        self.asset_dao
            .save(&asset)
            .await
            .map_err(|e| {
                warn!(asset_id = %asset.id, "Failed to upsert asset from edge: {}", e);
                format!("Failed to save asset: {}", e)
            })?;

        Ok(())
    }
}
