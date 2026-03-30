#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use vl_core::entities::{TbMsg, msg_type};
    use crate::{
        memory::{InMemoryBus, InMemoryProducer},
        topics, QueueMsg, TbConsumer, TbProducer,
    };

    fn make_bus() -> InMemoryBus {
        InMemoryBus::new(64)
    }

    fn make_tb_msg(msg_type: &str) -> TbMsg {
        TbMsg::new(msg_type, Uuid::new_v4(), "DEVICE", r#"{"temperature":25.5}"#)
    }

    // ── Round-trip ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn send_and_poll_round_trip() {
        let bus = make_bus();
        let producer = InMemoryProducer::new(bus.clone());
        let mut consumer = bus.subscribe(&[topics::VL_RULE_ENGINE], "test-group");

        let orig = make_tb_msg(msg_type::POST_TELEMETRY_REQUEST);
        producer
            .send_tb_msg(topics::VL_RULE_ENGINE, &orig)
            .await
            .unwrap();

        let msgs = consumer.poll().await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].topic, topics::VL_RULE_ENGINE);

        let decoded = msgs[0].to_tb_msg().unwrap();
        assert_eq!(decoded.msg_type, msg_type::POST_TELEMETRY_REQUEST);
        assert_eq!(decoded.originator_id, orig.originator_id);
    }

    // ── Multiple consumers (fanout) ───────────────────────────────────────────

    #[tokio::test]
    async fn multiple_consumers_receive_same_message() {
        let bus = make_bus();
        let producer = InMemoryProducer::new(bus.clone());
        let mut c1 = bus.subscribe(&[topics::VL_CORE], "group-a");
        let mut c2 = bus.subscribe(&[topics::VL_CORE], "group-b");

        let msg = make_tb_msg(msg_type::ENTITY_CREATED);
        producer.send_tb_msg(topics::VL_CORE, &msg).await.unwrap();

        let r1 = c1.poll().await.unwrap();
        let r2 = c2.poll().await.unwrap();

        assert_eq!(r1.len(), 1, "group-a should receive message");
        assert_eq!(r2.len(), 1, "group-b should receive message");
    }

    // ── Empty poll ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn poll_returns_empty_when_no_messages() {
        let bus = make_bus();
        let mut consumer = bus.subscribe(&[topics::VL_NOTIFICATIONS], "test");

        let msgs = consumer.poll().await.unwrap();
        assert!(msgs.is_empty());
    }

    // ── Topic isolation ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn messages_on_different_topics_are_isolated() {
        let bus = make_bus();
        let producer = InMemoryProducer::new(bus.clone());
        let mut consumer = bus.subscribe(&[topics::VL_RULE_ENGINE], "test");

        // Publish to a topic the consumer is NOT subscribed to
        let msg = make_tb_msg(msg_type::CONNECT_EVENT);
        producer
            .send_tb_msg(topics::VL_TRANSPORT_API_REQUESTS, &msg)
            .await
            .unwrap();

        let msgs = consumer.poll().await.unwrap();
        assert!(msgs.is_empty(), "consumer should not receive messages from other topics");
    }

    // ── No receivers — producer doesn't error ─────────────────────────────────

    #[tokio::test]
    async fn producer_ok_with_no_receivers() {
        let bus = make_bus();
        let producer = InMemoryProducer::new(bus.clone());

        // No consumer subscribed — should not error
        let msg = make_tb_msg(msg_type::POST_TELEMETRY_REQUEST);
        let result = producer.send_tb_msg(topics::VL_RULE_ENGINE, &msg).await;
        assert!(result.is_ok());
    }

    // ── commit is a no-op ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn commit_is_noop() {
        let bus = make_bus();
        let mut consumer = bus.subscribe(&[topics::VL_CORE], "test");
        assert!(consumer.commit().await.is_ok());
    }

    // ── Factory creates correct backend ───────────────────────────────────────

    #[tokio::test]
    async fn factory_in_memory_producer_works() {
        use vl_config::QueueConfig;
        let config = QueueConfig::default();
        let producer = crate::create_producer(&config).unwrap();

        // Wrap in Arc<dyn TbProducer> — verify it's usable
        let msg = make_tb_msg(msg_type::ALARM);
        let result = producer.send_tb_msg(topics::VL_CORE, &msg).await;
        assert!(result.is_ok());
    }

    // ── QueueMsg serialization ────────────────────────────────────────────────

    #[test]
    fn queue_msg_round_trip_serialization() {
        let tb_msg = make_tb_msg(msg_type::POST_TELEMETRY_REQUEST);
        let queue_msg = QueueMsg::from_tb_msg(topics::VL_RULE_ENGINE, &tb_msg).unwrap();

        assert_eq!(queue_msg.topic, topics::VL_RULE_ENGINE);
        assert_eq!(queue_msg.key, tb_msg.originator_id.to_string());

        let decoded = queue_msg.to_tb_msg().unwrap();
        assert_eq!(decoded.msg_type, tb_msg.msg_type);
        assert_eq!(decoded.data, tb_msg.data);
    }
}
