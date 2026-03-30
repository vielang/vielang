use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::dashboard::DashboardDao;

/// Handles DASHBOARD entity sync between cloud and edge.
pub struct DashboardProcessor {
    dashboard_dao: Arc<DashboardDao>,
}

impl DashboardProcessor {
    pub fn new(dashboard_dao: Arc<DashboardDao>) -> Self {
        Self { dashboard_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for DashboardProcessor {
    fn entity_type(&self) -> &'static str {
        "DASHBOARD"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "DASHBOARD",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                let dashboard = self.dashboard_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching dashboard: {}", e))?
                    .ok_or_else(|| format!("Dashboard not found: {}", entity_id))?;

                let body = serde_json::to_value(&dashboard)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "DASHBOARD",
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
        let dashboard: vl_core::entities::Dashboard = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize dashboard from edge: {}", e))?;

        self.dashboard_dao
            .save(&dashboard)
            .await
            .map_err(|e| {
                warn!(dashboard_id = %dashboard.id, "Failed to upsert dashboard from edge: {}", e);
                format!("Failed to save dashboard: {}", e)
            })?;

        Ok(())
    }
}
