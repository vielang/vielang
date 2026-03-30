use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::alarm::AlarmDao;

/// Handles ALARM entity sync between cloud and edge.
pub struct AlarmProcessor {
    alarm_dao: Arc<AlarmDao>,
}

impl AlarmProcessor {
    pub fn new(alarm_dao: Arc<AlarmDao>) -> Self {
        Self { alarm_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for AlarmProcessor {
    fn entity_type(&self) -> &'static str {
        "ALARM"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "ALARM",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                let alarm = self.alarm_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching alarm: {}", e))?
                    .ok_or_else(|| format!("Alarm not found: {}", entity_id))?;

                let body = serde_json::to_value(&alarm)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "ALARM",
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
        let alarm: vl_core::entities::Alarm = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize alarm from edge: {}", e))?;

        self.alarm_dao
            .save(&alarm)
            .await
            .map_err(|e| {
                warn!(alarm_id = %alarm.id, "Failed to upsert alarm from edge: {}", e);
                format!("Failed to save alarm: {}", e)
            })?;

        Ok(())
    }
}
