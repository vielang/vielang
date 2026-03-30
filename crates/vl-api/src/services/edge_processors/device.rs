use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::device::DeviceDao;

/// Handles DEVICE entity sync between cloud and edge.
pub struct DeviceProcessor {
    device_dao: Arc<DeviceDao>,
}

impl DeviceProcessor {
    pub fn new(device_dao: Arc<DeviceDao>) -> Self {
        Self { device_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for DeviceProcessor {
    fn entity_type(&self) -> &'static str {
        "DEVICE"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "DEVICE",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                let device = self.device_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching device: {}", e))?
                    .ok_or_else(|| format!("Device not found: {}", entity_id))?;

                let body = serde_json::to_value(&device)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "DEVICE",
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
        let device: vl_core::entities::Device = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize device from edge: {}", e))?;

        self.device_dao
            .save(&device)
            .await
            .map_err(|e| {
                warn!(device_id = %device.id, "Failed to upsert device from edge: {}", e);
                format!("Failed to save device: {}", e)
            })?;

        Ok(())
    }
}
