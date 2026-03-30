/// SendNotificationNode — Phase 15 (updated P3: channels support)
///
/// Enrich message metadata với notification intent rồi route về Success.
/// Downstream consumers (vl-api notification service) sẽ đọc metadata
/// và thực hiện delivery thực tế.
///
/// Config JSON:
/// ```json
/// {
///   "templateId": "uuid-of-notification-template",
///   "targetIds":  ["uuid1", "uuid2"],
///   "bodyKey":    "alertMessage",
///   "channels":   ["SLACK", "EMAIL"]   // default: ["PLATFORM"]
/// }
/// ```
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};
use vl_core::entities::TbMsg;

#[derive(Debug)]
pub struct SendNotificationNode {
    template_id: Uuid,
    target_ids:  Vec<Uuid>,
    body_key:    Option<String>,
    /// Notification delivery channels — default: ["PLATFORM"]
    channels:    Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    template_id: Uuid,
    #[serde(default)]
    target_ids:  Vec<Uuid>,
    body_key:    Option<String>,
    #[serde(default = "default_channels")]
    channels:    Vec<String>,
}

fn default_channels() -> Vec<String> {
    vec!["PLATFORM".to_string()]
}

impl SendNotificationNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("SendNotificationNode: {}", e)))?;

        Ok(Self {
            template_id: cfg.template_id,
            target_ids:  cfg.target_ids,
            body_key:    cfg.body_key,
            channels:    cfg.channels,
        })
    }
}

#[async_trait::async_trait]
impl RuleNode for SendNotificationNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        mut msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, crate::error::RuleEngineError> {
        // Enrich metadata với notification intent
        msg.metadata.insert(
            "notification_template_id".into(),
            self.template_id.to_string(),
        );
        msg.metadata.insert(
            "notification_target_ids".into(),
            serde_json::to_string(&self.target_ids).unwrap_or_default(),
        );

        if let Some(key) = &self.body_key {
            msg.metadata.insert("notification_body_key".into(), key.clone());
        }

        msg.metadata.insert(
            "notification_channels".into(),
            serde_json::to_string(&self.channels).unwrap_or_default(),
        );

        tracing::info!(
            template_id = %self.template_id,
            target_count = self.target_ids.len(),
            msg_id       = %msg.id,
            "SendNotification: enriched message metadata"
        );

        Ok(vec![(RelationType::Success, msg)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use serde_json::json;
    use uuid::Uuid;

    use vl_core::entities::{TbMsg, msg_type};
    use vl_dao::postgres::{
        alarm::AlarmDao, asset::AssetDao, customer::CustomerDao, device::DeviceDao, device_profile::DeviceProfileDao,
        event::EventDao, geofence::GeofenceDao, kv::KvDao, relation::RelationDao, tenant::TenantDao,
    };
    use crate::node::{DaoServices, RuleNodeCtx};

    fn make_ctx() -> RuleNodeCtx {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").expect("lazy pool");
        RuleNodeCtx {
            node_id:     Uuid::nil(),
            tenant_id:   Uuid::nil(),
            edge_sender: None,
            dao: Arc::new(DaoServices {
                kv:             Arc::new(KvDao::new(pool.clone())),
                alarm:          Arc::new(AlarmDao::new(pool.clone())),
                device:         Arc::new(DeviceDao::new(pool.clone())),
                device_profile: Arc::new(DeviceProfileDao::new(pool.clone())),
                asset:          Arc::new(AssetDao::new(pool.clone())),
                relation:       Arc::new(RelationDao::new(pool.clone())),
                customer:       Arc::new(CustomerDao::new(pool.clone())),
                tenant:         Arc::new(TenantDao::new(pool.clone())),
                event:          Arc::new(EventDao::new(pool.clone())),
                geofence:       Arc::new(GeofenceDao::new(pool)),
            }),
        }
    }

    fn make_msg() -> TbMsg {
        TbMsg::new(msg_type::ALARM, Uuid::new_v4(), "DEVICE", "{}")
    }

    #[tokio::test]
    async fn send_notification_enriches_metadata() {
        let template_id = Uuid::new_v4();
        let target_id   = Uuid::new_v4();

        let node = SendNotificationNode::new(&json!({
            "templateId": template_id,
            "targetIds":  [target_id],
            "bodyKey":    "alertBody"
        })).unwrap();

        let msg     = make_msg();
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, RelationType::Success);
        let out = &results[0].1;
        assert_eq!(
            out.metadata.get("notification_template_id").map(|s| s.as_str()),
            Some(template_id.to_string().as_str())
        );
        assert!(out.metadata.contains_key("notification_target_ids"));
        assert_eq!(
            out.metadata.get("notification_body_key").map(|s| s.as_str()),
            Some("alertBody")
        );
    }

    #[tokio::test]
    async fn send_notification_without_targets() {
        let template_id = Uuid::new_v4();

        let node = SendNotificationNode::new(&json!({
            "templateId": template_id,
        })).unwrap();

        let results = node.process(&make_ctx(), make_msg()).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert!(results[0].1.metadata.contains_key("notification_template_id"));
    }

    #[tokio::test]
    async fn send_notification_missing_template_id_returns_error() {
        let result = SendNotificationNode::new(&json!({}));
        assert!(result.is_err());
    }
}
