use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::relation::RelationDao;

/// Handles RELATION entity sync between cloud and edge.
///
/// Relations are structural links between entities (e.g., device-to-asset).
/// Unlike other processors, relations don't have a single UUID primary key —
/// they are identified by (from_id, to_id, relation_type).
pub struct RelationProcessor {
    relation_dao: Arc<RelationDao>,
}

impl RelationProcessor {
    pub fn new(relation_dao: Arc<RelationDao>) -> Self {
        Self { relation_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for RelationProcessor {
    fn entity_type(&self) -> &'static str {
        "RELATION"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        // Relations don't have a single UUID — entity_id here refers to the
        // "from" entity. The full relation context should be in the edge event body.
        // For downlink, we return a stub indicating the action.
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "RELATION",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                // Fetch all relations where this entity is the "from" side.
                // We don't know the entity type here, so we use a generic "ENTITY" placeholder.
                // In practice, the caller should provide the from_type via the edge event body.
                let relations = self.relation_dao
                    .find_by_from(entity_id, "DEVICE")
                    .await
                    .map_err(|e| format!("DAO error fetching relations: {}", e))?;

                let body = serde_json::to_value(&relations)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "RELATION",
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
        let relation: vl_core::entities::EntityRelation = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize relation from edge: {}", e))?;

        self.relation_dao
            .save(&relation)
            .await
            .map_err(|e| {
                warn!("Failed to upsert relation from edge: {}", e);
                format!("Failed to save relation: {}", e)
            })?;

        Ok(())
    }
}
