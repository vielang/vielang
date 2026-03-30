/// Unit tests for transformation nodes — no DB required.
/// Each test verifies correct message mutation.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use serde_json::json;
    use uuid::Uuid;

    use vl_core::entities::{TbMsg, msg_type};
    use vl_dao::postgres::{alarm::AlarmDao, asset::AssetDao, customer::CustomerDao, device::DeviceDao, device_profile::DeviceProfileDao, event::EventDao, geofence::GeofenceDao, kv::KvDao, relation::RelationDao, tenant::TenantDao};
    use crate::node::{DaoServices, RelationType, RuleNode, RuleNodeCtx};
    #[allow(unused_imports)]
    use crate::nodes::transform::*;

    /// Create a no-pool ctx — safe for pure-logic nodes that never hit DB
    fn make_ctx() -> RuleNodeCtx {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test")
            .expect("lazy pool");
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

    fn make_msg(data: serde_json::Value) -> TbMsg {
        TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, Uuid::new_v4(), "DEVICE", data.to_string())
    }

    // ── CopyKeysNode ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn copy_keys_data_to_metadata() {
        let node = CopyKeysNode::new(&json!({
            "keys": ["temperature", "humidity"],
            "fromMetadata": false
        })).unwrap();

        let msg = make_msg(json!({ "temperature": 25.5, "humidity": 80 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, RelationType::Success);
        let out = &results[0].1;
        assert_eq!(out.metadata.get("temperature").map(|s| s.as_str()), Some("25.5"));
        assert_eq!(out.metadata.get("humidity").map(|s| s.as_str()), Some("80"));
    }

    #[tokio::test]
    async fn copy_keys_metadata_to_data() {
        let node = CopyKeysNode::new(&json!({
            "keys": ["deviceName"],
            "fromMetadata": true
        })).unwrap();

        let mut msg = make_msg(json!({}));
        msg.metadata.insert("deviceName".into(), "sensor-01".into());

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let data: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(data["deviceName"].as_str(), Some("sensor-01"));
    }

    #[tokio::test]
    async fn copy_keys_missing_key_ignored() {
        let node = CopyKeysNode::new(&json!({
            "keys": ["nonExistent"],
            "fromMetadata": false
        })).unwrap();

        let msg = make_msg(json!({ "temperature": 20 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        // Should still succeed, just not add the key
        assert_eq!(results[0].0, RelationType::Success);
        assert!(results[0].1.metadata.get("nonExistent").is_none());
    }

    // ── DeleteKeysNode ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_keys_from_data() {
        let node = DeleteKeysNode::new(&json!({
            "keys": ["password", "secret"],
            "fromMetadata": false
        })).unwrap();

        let msg = make_msg(json!({ "temperature": 22, "password": "s3cr3t", "secret": "abc" }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let data: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert!(data.get("password").is_none());
        assert!(data.get("secret").is_none());
        assert_eq!(data["temperature"], json!(22));
    }

    #[tokio::test]
    async fn delete_keys_from_metadata() {
        let node = DeleteKeysNode::new(&json!({
            "keys": ["internalKey"],
            "fromMetadata": true
        })).unwrap();

        let mut msg = make_msg(json!({}));
        msg.metadata.insert("internalKey".into(), "hidden".into());
        msg.metadata.insert("deviceName".into(), "keep-me".into());

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert!(results[0].1.metadata.get("internalKey").is_none());
        assert_eq!(results[0].1.metadata.get("deviceName").map(|s| s.as_str()), Some("keep-me"));
    }

    // ── RenameKeysNode ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn rename_keys_in_data() {
        let node = RenameKeysNode::new(&json!({
            "renameMap": { "temp": "temperature", "hum": "humidity" },
            "fromMetadata": false
        })).unwrap();

        let msg = make_msg(json!({ "temp": 30, "hum": 65, "other": "keep" }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let data: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert!(data.get("temp").is_none());
        assert!(data.get("hum").is_none());
        assert_eq!(data["temperature"], json!(30));
        assert_eq!(data["humidity"], json!(65));
        assert_eq!(data["other"], json!("keep"));
    }

    #[tokio::test]
    async fn rename_keys_in_metadata() {
        let node = RenameKeysNode::new(&json!({
            "renameMap": { "dev": "deviceName" },
            "fromMetadata": true
        })).unwrap();

        let mut msg = make_msg(json!({}));
        msg.metadata.insert("dev".into(), "my-device".into());

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let out = &results[0].1;
        assert!(out.metadata.get("dev").is_none());
        assert_eq!(out.metadata.get("deviceName").map(|s| s.as_str()), Some("my-device"));
    }

    // ── ToEmailNode ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn to_email_substitutes_metadata_placeholders() {
        let node = ToEmailNode::new(&json!({
            "fromTemplate": "noreply@example.com",
            "toTemplate": "${customerEmail}",
            "subjectTemplate": "Alert: ${alarmType}",
            "bodyTemplate": "Device ${deviceName} triggered at ${ts}",
            "isHtml": false
        })).unwrap();

        let mut msg = make_msg(json!({}));
        msg.metadata.insert("customerEmail".into(), "cust@example.com".into());
        msg.metadata.insert("alarmType".into(), "HighTemp".into());
        msg.metadata.insert("deviceName".into(), "sensor-01".into());

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let out = &results[0].1;
        assert_eq!(out.metadata["email_to"], "cust@example.com");
        assert_eq!(out.metadata["email_subject"], "Alert: HighTemp");
        assert!(out.metadata["email_body"].contains("sensor-01"));
        assert_eq!(out.metadata["email_from"], "noreply@example.com");
        assert_eq!(out.metadata["email_is_html"], "false");
    }

    #[tokio::test]
    async fn to_email_unresolved_placeholder_left_as_is() {
        let node = ToEmailNode::new(&json!({
            "fromTemplate": "no-reply@x.com",
            "toTemplate": "${missingKey}",
            "subjectTemplate": "Test",
            "bodyTemplate": "body",
            "isHtml": false
        })).unwrap();

        let msg = make_msg(json!({}));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        // Unresolved placeholder stays in output
        assert!(results[0].1.metadata["email_to"].contains("missingKey"));
    }

    // ── JsonPathNode ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn json_path_default_passthrough() {
        // "$" is the default — message should pass through unchanged
        let node = JsonPathNode::new(&json!({ "jsonPath": "$" })).unwrap();
        let msg = make_msg(json!({ "temperature": 25.5, "humidity": 80 }));
        let original_data = msg.data.clone();

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.data, original_data);
    }

    #[tokio::test]
    async fn json_path_extract_single_field() {
        let node = JsonPathNode::new(&json!({ "jsonPath": "$.temperature" })).unwrap();
        let msg = make_msg(json!({ "temperature": 42.0, "humidity": 60 }));

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        // data should be just the value, not the whole object
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val, json!(42.0));
    }

    #[tokio::test]
    async fn json_path_nested_field() {
        let node = JsonPathNode::new(&json!({ "jsonPath": "$.device.id" })).unwrap();
        let msg = make_msg(json!({ "device": { "id": "abc-123", "type": "sensor" } }));

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val, json!("abc-123"));
    }

    #[tokio::test]
    async fn json_path_wildcard_returns_array() {
        let node = JsonPathNode::new(&json!({ "jsonPath": "$.sensors[*].value" })).unwrap();
        let msg = make_msg(json!({
            "sensors": [
                { "name": "temp", "value": 22 },
                { "name": "hum",  "value": 65 }
            ]
        }));

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert!(val.is_array());
        assert_eq!(val, json!([22, 65]));
    }

    #[tokio::test]
    async fn json_path_no_match_routes_to_failure() {
        let node = JsonPathNode::new(&json!({ "jsonPath": "$.nonExistent" })).unwrap();
        let msg = make_msg(json!({ "temperature": 25 }));

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
        assert!(results[0].1.metadata.contains_key("error"));
    }

    #[tokio::test]
    async fn json_path_no_config_defaults_to_passthrough() {
        // Empty config should default to "$" (pass-through)
        let node = JsonPathNode::new(&json!({})).unwrap();
        let msg = make_msg(json!({ "value": 99 }));
        let original_data = msg.data.clone();

        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.data, original_data);
    }

    // ── TransformMsgNode ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn transform_msg_script_converts_data() {
        let node = TransformMsgNode::new(&json!({
            "jsScript": r#"
                let new_data = "transformed";
                new_data
            "#
        })).unwrap();

        let msg = make_msg(json!({ "value": 42 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.data, "transformed");
    }

    #[tokio::test]
    async fn transform_msg_script_error_routes_to_failure() {
        let node = TransformMsgNode::new(&json!({
            "jsScript": "this is not valid rhai code @@@"
        })).unwrap();

        let msg = make_msg(json!({}));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
        assert!(results[0].1.metadata.contains_key("error"));
    }

    // ── ParseMsgNode ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn parse_msg_json_mode_validates_and_passes() {
        let node = ParseMsgNode::new(&json!({ "parseMode": "JSON" })).unwrap();
        let msg = make_msg(json!({ "temperature": 25 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["temperature"], json!(25));
    }

    #[tokio::test]
    async fn parse_msg_json_mode_invalid_routes_failure() {
        let node = ParseMsgNode::new(&json!({ "parseMode": "JSON" })).unwrap();
        let mut msg = make_msg(json!({}));
        msg.data = "not json at all".into();
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
        assert!(results[0].1.metadata.contains_key("error"));
    }

    #[tokio::test]
    async fn parse_msg_text_mode_wraps_in_object() {
        let node = ParseMsgNode::new(&json!({ "parseMode": "TEXT" })).unwrap();
        let mut msg = make_msg(json!({}));
        msg.data = "hello world".into();
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["text"].as_str(), Some("hello world"));
    }

    #[tokio::test]
    async fn parse_msg_csv_mode_produces_object() {
        let node = ParseMsgNode::new(&json!({
            "parseMode": "CSV",
            "delimiter": ",",
            "headerLine": true
        })).unwrap();
        let mut msg = make_msg(json!({}));
        msg.data = "temperature,humidity\n22.5,60".into();
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["temperature"], json!(22.5));
        assert_eq!(val["humidity"], json!(60.0));
    }

    // ── FormatTelemetryNode ───────────────────────────────────────────────────

    #[tokio::test]
    async fn format_telemetry_round_operation() {
        let node = FormatTelemetryNode::new(&json!({
            "operations": [{ "key": "temperature", "type": "ROUND", "precision": 1 }]
        })).unwrap();
        let msg = make_msg(json!({ "temperature": 22.567 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["temperature"], json!(22.6));
    }

    #[tokio::test]
    async fn format_telemetry_multiply_converts_units() {
        let node = FormatTelemetryNode::new(&json!({
            "operations": [{
                "key": "speed_kmh", "type": "MULTIPLY",
                "sourceKey": "speed_ms", "factor": 3.6
            }]
        })).unwrap();
        let msg = make_msg(json!({ "speed_ms": 10.0 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert!((val["speed_kmh"].as_f64().unwrap() - 36.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn format_telemetry_abs_operation() {
        let node = FormatTelemetryNode::new(&json!({
            "operations": [{ "key": "value", "type": "ABS" }]
        })).unwrap();
        let msg = make_msg(json!({ "value": -42.5 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["value"], json!(42.5));
    }

    #[tokio::test]
    async fn format_telemetry_non_object_routes_failure() {
        let node = FormatTelemetryNode::new(&json!({ "operations": [] })).unwrap();
        let mut msg = make_msg(json!({}));
        msg.data = "[1, 2, 3]".into(); // array, not object
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
    }

    // ── StringToJsonNode ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn string_to_json_converts_nested_string() {
        let node = StringToJsonNode::new(&json!({
            "fieldName": "payload",
            "fromMetadata": false
        })).unwrap();
        let msg = make_msg(json!({ "payload": "{\"temp\":22}" }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["payload"]["temp"], json!(22));
    }

    #[tokio::test]
    async fn string_to_json_missing_field_routes_failure() {
        let node = StringToJsonNode::new(&json!({
            "fieldName": "nonExistent",
            "fromMetadata": false,
            "failOnError": true
        })).unwrap();
        let msg = make_msg(json!({ "temperature": 25 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
        assert!(results[0].1.metadata.contains_key("error"));
    }

    #[tokio::test]
    async fn string_to_json_from_metadata() {
        let node = StringToJsonNode::new(&json!({
            "fieldName": "rawJson",
            "fromMetadata": true
        })).unwrap();
        let mut msg = make_msg(json!({}));
        msg.metadata.insert("rawJson".into(), r#"{"key":"value"}"#.into());
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        let val: serde_json::Value = serde_json::from_str(&results[0].1.data).unwrap();
        assert_eq!(val["rawJson"]["key"].as_str(), Some("value"));
    }

    // ── AssignAttributeNode ───────────────────────────────────────────────────

    #[tokio::test]
    async fn assign_attribute_from_data_to_metadata() {
        let node = AssignAttributeNode::new(&json!({
            "mapping": [{
                "sourceKey": "temperature",
                "targetKey": "ss_temperature",
                "fromData": true
            }]
        })).unwrap();
        let msg = make_msg(json!({ "temperature": 28.5 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.metadata.get("ss_temperature").map(|s| s.as_str()), Some("28.5"));
    }

    #[tokio::test]
    async fn assign_attribute_from_metadata_to_metadata() {
        let node = AssignAttributeNode::new(&json!({
            "mapping": [{
                "sourceKey": "deviceId",
                "targetKey": "device_id_copy",
                "fromData": false
            }]
        })).unwrap();
        let mut msg = make_msg(json!({}));
        msg.metadata.insert("deviceId".into(), "abc-123".into());
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.metadata.get("device_id_copy").map(|s| s.as_str()), Some("abc-123"));
    }

    #[tokio::test]
    async fn assign_attribute_missing_key_with_failure_flag() {
        let node = AssignAttributeNode::new(&json!({
            "mapping": [{ "sourceKey": "missing", "targetKey": "out", "fromData": true }],
            "tellFailureIfAbsent": true
        })).unwrap();
        let msg = make_msg(json!({ "temperature": 25 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Failure);
    }

    #[tokio::test]
    async fn assign_attribute_missing_key_without_failure_flag() {
        let node = AssignAttributeNode::new(&json!({
            "mapping": [{ "sourceKey": "missing", "targetKey": "out", "fromData": true }],
            "tellFailureIfAbsent": false
        })).unwrap();
        let msg = make_msg(json!({ "temperature": 25 }));
        let results = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].0, RelationType::Success);
    }
}
