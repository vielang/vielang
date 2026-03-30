use std::sync::Arc;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

use vl_cluster::EdgeEventProcessor;
use vl_dao::postgres::rule_chain::RuleChainDao;

/// Handles RULE_CHAIN entity sync between cloud and edge.
pub struct RuleChainProcessor {
    rule_chain_dao: Arc<RuleChainDao>,
}

impl RuleChainProcessor {
    pub fn new(rule_chain_dao: Arc<RuleChainDao>) -> Self {
        Self { rule_chain_dao }
    }
}

#[async_trait]
impl EdgeEventProcessor for RuleChainProcessor {
    fn entity_type(&self) -> &'static str {
        "RULE_CHAIN"
    }

    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match action {
            "DELETED" => Ok(serde_json::json!({
                "entityType": "RULE_CHAIN",
                "entityId": entity_id.to_string(),
                "action": "DELETED",
            })),
            _ => {
                let rule_chain = self.rule_chain_dao
                    .find_by_id(entity_id)
                    .await
                    .map_err(|e| format!("DAO error fetching rule chain: {}", e))?
                    .ok_or_else(|| format!("Rule chain not found: {}", entity_id))?;

                let body = serde_json::to_value(&rule_chain)
                    .map_err(|e| format!("Serialization error: {}", e))?;

                Ok(serde_json::json!({
                    "entityType": "RULE_CHAIN",
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
        let rule_chain: vl_core::entities::RuleChain = serde_json::from_value(
            payload.get("body").cloned().unwrap_or(payload.clone())
        ).map_err(|e| format!("Failed to deserialize rule chain from edge: {}", e))?;

        self.rule_chain_dao
            .save(&rule_chain)
            .await
            .map_err(|e| {
                warn!(rule_chain_id = %rule_chain.id, "Failed to upsert rule chain from edge: {}", e);
                format!("Failed to save rule chain: {}", e)
            })?;

        Ok(())
    }
}
