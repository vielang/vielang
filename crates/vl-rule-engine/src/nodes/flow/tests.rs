/// Unit tests for flow control nodes — no DB required.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use serde_json::json;
    use uuid::Uuid;

    use vl_core::entities::{TbMsg, msg_type};
    use vl_dao::postgres::{alarm::AlarmDao, asset::AssetDao, customer::CustomerDao, device::DeviceDao, device_profile::DeviceProfileDao, event::EventDao, geofence::GeofenceDao, kv::KvDao, relation::RelationDao, tenant::TenantDao};
    use crate::node::{DaoServices, RelationType, RuleNode, RuleNodeCtx};
    use crate::nodes::flow::*;

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

    fn make_msg(msg_type: &str) -> TbMsg {
        TbMsg::new(msg_type, Uuid::new_v4(), "DEVICE", "{}")
    }

    // ── RuleChainInputNode ────────────────────────────────────────────────────

    #[tokio::test]
    async fn rule_chain_input_passes_through() {
        let node = RuleChainInputNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::POST_TELEMETRY_REQUEST);
        let results = node.process(&make_ctx(), msg.clone()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.id, msg.id);
    }

    // ── MsgTypeSwitchNode ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn msg_type_switch_telemetry() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::POST_TELEMETRY_REQUEST);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, RelationType::Other("Post Telemetry".into()));
    }

    #[tokio::test]
    async fn msg_type_switch_attributes() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::POST_ATTRIBUTES_REQUEST);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Other("Post Attributes".into()));
    }

    #[tokio::test]
    async fn msg_type_switch_alarm() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::ALARM);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Other("Alarm".into()));
    }

    #[tokio::test]
    async fn msg_type_switch_connect_event() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::CONNECT_EVENT);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Other("Connect".into()));
    }

    #[tokio::test]
    async fn msg_type_switch_disconnect_event() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::DISCONNECT_EVENT);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Other("Disconnect".into()));
    }

    #[tokio::test]
    async fn msg_type_switch_unknown_type_routes_to_other() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg("SOME_CUSTOM_TYPE");
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Other("Other".into()));
    }

    #[tokio::test]
    async fn msg_type_switch_entity_created() {
        let node = MsgTypeSwitchNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::ENTITY_CREATED);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Other("Entity Created".into()));
    }

    // ── CheckpointNode ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn checkpoint_passes_through_on_success() {
        let node = CheckpointNode::new(&json!({ "queueName": "Main" })).unwrap();
        let msg = make_msg(msg_type::POST_TELEMETRY_REQUEST);
        let results = node.process(&make_ctx(), msg.clone()).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, RelationType::Success);
        assert_eq!(results[0].1.id, msg.id);
    }

    #[tokio::test]
    async fn checkpoint_default_queue_name() {
        // Config without queueName — should use default "Main"
        let node = CheckpointNode::new(&json!({})).unwrap();
        let msg = make_msg(msg_type::CONNECT_EVENT);
        let results = node.process(&make_ctx(), msg).await.unwrap();

        assert_eq!(results[0].0, RelationType::Success);
    }

    // ── Combined flow: Input → Switch → Checkpoint ────────────────────────────

    #[tokio::test]
    async fn input_node_msg_id_preserved() {
        let input = RuleChainInputNode::new(&json!({})).unwrap();
        let original_id = Uuid::new_v4();
        let msg = TbMsg {
            id: original_id,
            ts: 0,
            msg_type: msg_type::POST_TELEMETRY_REQUEST.into(),
            originator_id: Uuid::new_v4(),
            originator_type: "DEVICE".into(),
            customer_id: None,
            metadata: Default::default(),
            data: "{}".into(),
            rule_chain_id: None,
            rule_node_id: None,
            tenant_id: None,
        };

        let results = input.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(results[0].1.id, original_id);
    }
}
