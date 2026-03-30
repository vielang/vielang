use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::widget_type::WidgetTypeDao;
use vl_dao::postgres::widgets_bundle::WidgetsBundleDao;

/// Handles WIDGET entity sync between cloud and edge.
///
/// Widgets encompass both `WidgetType` (individual widget definitions) and
/// `WidgetsBundle` (collections of widget types). The downlink serializes
/// widget types by ID; uplink upserts them.
pub struct WidgetProcessor {
    widget_type_dao:    Arc<WidgetTypeDao>,
    widgets_bundle_dao: Arc<WidgetsBundleDao>,
}

impl WidgetProcessor {
    pub fn new(
        widget_type_dao: Arc<WidgetTypeDao>,
        widgets_bundle_dao: Arc<WidgetsBundleDao>,
    ) -> Self {
        Self { widget_type_dao, widgets_bundle_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for WidgetProcessor {
    fn entity_type(&self) -> &'static str {
        "WIDGET"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "WIDGET",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                // Try widget type first, then widgets bundle.
                if let Some(wt) = self.widget_type_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching widget type: {}", e))?
                {
                    let body = serde_json::to_value(&wt)
                        .map_err(|e| format!("Serialization error: {}", e))?;
                    return Ok(serde_json::json!({
                        "entityType": "WIDGET",
                        "subType": "WIDGET_TYPE",
                        "entityId": entity_id.to_string(),
                        "action": action,
                        "body": body,
                    }));
                }

                if let Some(wb) = self.widgets_bundle_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching widgets bundle: {}", e))?
                {
                    let body = serde_json::to_value(&wb)
                        .map_err(|e| format!("Serialization error: {}", e))?;
                    return Ok(serde_json::json!({
                        "entityType": "WIDGET",
                        "subType": "WIDGETS_BUNDLE",
                        "entityId": entity_id.to_string(),
                        "action": action,
                        "body": body,
                    }));
                }

                Err(format!("Widget not found (type or bundle): {}", entity_id))
            }
        }
    }

    async fn process_uplink(
        &self,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        let sub_type = payload.get("subType")
            .and_then(|v| v.as_str())
            .unwrap_or("WIDGET_TYPE");

        let body = payload.get("body").cloned().unwrap_or(payload.clone());

        match sub_type {
            "WIDGETS_BUNDLE" => {
                let bundle: vl_core::entities::WidgetsBundle = serde_json::from_value(body)
                    .map_err(|e| format!("Failed to deserialize widgets bundle from edge: {}", e))?;
                self.widgets_bundle_dao
                    .save(&bundle)
                    .await
                    .map_err(|e| {
                        warn!(bundle_id = %bundle.id, "Failed to upsert widgets bundle from edge: {}", e);
                        format!("Failed to save widgets bundle: {}", e)
                    })?;
            }
            _ => {
                let wt: vl_core::entities::WidgetType = serde_json::from_value(body)
                    .map_err(|e| format!("Failed to deserialize widget type from edge: {}", e))?;
                self.widget_type_dao
                    .save(&wt)
                    .await
                    .map_err(|e| {
                        warn!(widget_type_id = %wt.id, "Failed to upsert widget type from edge: {}", e);
                        format!("Failed to save widget type: {}", e)
                    })?;
            }
        }

        Ok(())
    }
}
