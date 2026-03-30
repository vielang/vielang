// Unit tests for filter nodes
// These tests don't touch the DB — filter nodes only use ctx.node_id/tenant_id

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;

    use vl_core::entities::{TbMsg, msg_type};
    use vl_dao::postgres::{alarm::AlarmDao, asset::AssetDao, customer::CustomerDao, device::DeviceDao, device_profile::DeviceProfileDao, event::EventDao, geofence::GeofenceDao, kv::KvDao, relation::RelationDao, tenant::TenantDao};
    use crate::{
        node::{DaoServices, RelationType, RuleNodeCtx},
        nodes::filter::{
            CheckAlarmStatusNode, CheckMessageNode, CheckRelationNode,
            MsgTypeFilter, OriginatorTypeFilterNode, OriginatorTypeSwitchNode,
            ScriptFilter, ThresholdFilterNode,
        },
        node::RuleNode,
    };

    /// Build a ctx with a lazy pool — safe for nodes that never call ctx.dao
    fn make_ctx() -> RuleNodeCtx {
        // connect_lazy does not open a connection, safe for unit tests
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test")
            .expect("connect_lazy failed");
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

    fn make_msg(typ: &str) -> TbMsg {
        TbMsg::new(typ, Uuid::new_v4(), "DEVICE", "{}")
    }

    #[tokio::test]
    async fn msg_type_filter_matches() {
        let config = json!({ "messageTypes": ["POST_TELEMETRY_REQUEST"] });
        let node = MsgTypeFilter::new(&config).unwrap();
        let result = node.process(&make_ctx(), make_msg(msg_type::POST_TELEMETRY_REQUEST)).await.unwrap();
        assert_eq!(result[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn msg_type_filter_no_match() {
        let config = json!({ "messageTypes": ["CONNECT_EVENT"] });
        let node = MsgTypeFilter::new(&config).unwrap();
        let result = node.process(&make_ctx(), make_msg(msg_type::POST_TELEMETRY_REQUEST)).await.unwrap();
        assert_eq!(result[0].0, RelationType::False);
    }

    #[tokio::test]
    async fn script_filter_true_literal() {
        let config = json!({ "jsScript": "true" });
        let node = ScriptFilter::new(&config).unwrap();
        let result = node.process(&make_ctx(), make_msg(msg_type::POST_TELEMETRY_REQUEST)).await.unwrap();
        assert_eq!(result[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn script_filter_false_literal() {
        let config = json!({ "jsScript": "false" });
        let node = ScriptFilter::new(&config).unwrap();
        let result = node.process(&make_ctx(), make_msg(msg_type::CONNECT_EVENT)).await.unwrap();
        assert_eq!(result[0].0, RelationType::False);
    }

    #[tokio::test]
    async fn script_filter_msg_type_check() {
        let config = json!({ "jsScript": "msgType == \"POST_TELEMETRY_REQUEST\"" });
        let node = ScriptFilter::new(&config).unwrap();

        let r1 = node.process(&make_ctx(), make_msg(msg_type::POST_TELEMETRY_REQUEST)).await.unwrap();
        assert_eq!(r1[0].0, RelationType::True, "telemetry should match");

        let r2 = node.process(&make_ctx(), make_msg(msg_type::CONNECT_EVENT)).await.unwrap();
        assert_eq!(r2[0].0, RelationType::False, "connect event should not match");
    }

    // ── ThresholdFilterNode ───────────────────────────────────────────────────

    fn make_data_msg(data: serde_json::Value) -> TbMsg {
        TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, Uuid::new_v4(), "DEVICE", data.to_string())
    }

    #[tokio::test]
    async fn threshold_greater_than_passes() {
        let node = ThresholdFilterNode::new(&json!({
            "key": "temperature", "op": "GREATER_THAN", "threshold": 30.0
        })).unwrap();
        let msg = make_data_msg(json!({ "temperature": 35.5 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn threshold_greater_than_fails() {
        let node = ThresholdFilterNode::new(&json!({
            "key": "temperature", "op": "GREATER_THAN", "threshold": 30.0
        })).unwrap();
        let msg = make_data_msg(json!({ "temperature": 25.0 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::False);
    }

    #[tokio::test]
    async fn threshold_less_than_passes() {
        let node = ThresholdFilterNode::new(&json!({
            "key": "humidity", "op": "LESS_THAN", "threshold": 70.0
        })).unwrap();
        let msg = make_data_msg(json!({ "humidity": 60 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn threshold_missing_key_routes_failure() {
        let node = ThresholdFilterNode::new(&json!({
            "key": "nonExistent", "op": "GREATER_THAN", "threshold": 0.0
        })).unwrap();
        let msg = make_data_msg(json!({ "temperature": 25.0 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
        assert!(results[0].1.metadata.contains_key("error"));
    }

    #[tokio::test]
    async fn threshold_equal_passes() {
        let node = ThresholdFilterNode::new(&json!({
            "key": "level", "op": "EQUAL", "threshold": 100.0
        })).unwrap();
        let msg = make_data_msg(json!({ "level": 100.0 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    // ── CheckMessageNode ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn check_message_greater_condition_passes() {
        let node = CheckMessageNode::new(&json!({
            "conditions": [{ "key": "temperature", "type": "MSG_FIELD", "operation": "GREATER", "value": "30" }],
            "checkAllConditions": true
        })).unwrap();
        let msg = make_data_msg(json!({ "temperature": 35 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn check_message_equal_condition_fails() {
        let node = CheckMessageNode::new(&json!({
            "conditions": [{ "key": "status", "type": "MSG_FIELD", "operation": "EQUAL", "value": "active" }],
            "checkAllConditions": true
        })).unwrap();
        let msg = make_data_msg(json!({ "status": "inactive" }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::False);
    }

    #[tokio::test]
    async fn check_message_metadata_condition() {
        let node = CheckMessageNode::new(&json!({
            "conditions": [{ "key": "deviceType", "type": "METADATA", "operation": "EQUAL", "value": "sensor" }],
            "checkAllConditions": true
        })).unwrap();
        let mut msg = make_data_msg(json!({}));
        msg.metadata.insert("deviceType".into(), "sensor".into());
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn check_message_any_condition_mode() {
        // OR mode: one passes, one fails → True overall
        let node = CheckMessageNode::new(&json!({
            "conditions": [
                { "key": "a", "type": "MSG_FIELD", "operation": "EQUAL", "value": "x" },
                { "key": "b", "type": "MSG_FIELD", "operation": "EQUAL", "value": "y" }
            ],
            "checkAllConditions": false
        })).unwrap();
        let msg = make_data_msg(json!({ "a": "x", "b": "nope" }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn check_message_contains_operation() {
        let node = CheckMessageNode::new(&json!({
            "conditions": [{ "key": "msg", "type": "MSG_FIELD", "operation": "CONTAINS", "value": "error" }],
            "checkAllConditions": true
        })).unwrap();
        let msg = make_data_msg(json!({ "msg": "critical error occurred" }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    // ── OriginatorTypeFilterNode ──────────────────────────────────────────────

    #[tokio::test]
    async fn originator_type_filter_matches_device() {
        let node = OriginatorTypeFilterNode::new(&json!({
            "originatorTypes": ["DEVICE", "ASSET"]
        })).unwrap();
        let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, Uuid::new_v4(), "DEVICE", "{}");
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::True);
    }

    #[tokio::test]
    async fn originator_type_filter_rejects_user() {
        let node = OriginatorTypeFilterNode::new(&json!({
            "originatorTypes": ["DEVICE"]
        })).unwrap();
        let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, Uuid::new_v4(), "USER", "{}");
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::False);
    }

    // ── OriginatorTypeSwitchNode ──────────────────────────────────────────────

    #[tokio::test]
    async fn originator_type_switch_routes_by_type() {
        let node = OriginatorTypeSwitchNode::new(&json!({})).unwrap();

        let device_msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, Uuid::new_v4(), "DEVICE", "{}");
        let r1 = node.process(&make_ctx(), device_msg).await.unwrap();
        assert_eq!(r1[0].0, RelationType::Other("DEVICE".into()));

        let asset_msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, Uuid::new_v4(), "ASSET", "{}");
        let r2 = node.process(&make_ctx(), asset_msg).await.unwrap();
        assert_eq!(r2[0].0, RelationType::Other("ASSET".into()));
    }

    // ── CheckAlarmStatusNode config-only (no DB) ──────────────────────────────

    #[test]
    fn check_alarm_status_with_filters_creates_ok() {
        assert!(CheckAlarmStatusNode::new(&json!({
            "alarmStatusList": ["ACTIVE_UNACK", "ACTIVE_ACK"],
            "alarmTypes": ["High Temperature"]
        })).is_ok());
    }

    #[test]
    fn check_alarm_status_empty_config_allowed() {
        assert!(CheckAlarmStatusNode::new(&json!({})).is_ok());
    }

    // ── CheckRelationNode config-only ─────────────────────────────────────────

    #[test]
    fn check_relation_config_parsing() {
        assert!(CheckRelationNode::new(&json!({
            "direction": "FROM",
            "relationType": "Contains",
            "entityType": "ASSET"
        })).is_ok());
    }
}
