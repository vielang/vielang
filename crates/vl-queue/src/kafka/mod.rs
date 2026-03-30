pub mod consumer;

pub use consumer::KafkaConsumer;

use async_trait::async_trait;
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::Duration;
use tracing::{debug, error};

use vl_config::KafkaConfig;
use crate::{QueueError, QueueMsg, TbProducer};

// ── Kafka Producer ────────────────────────────────────────────────────────────

pub struct KafkaProducer {
    inner: FutureProducer,
}

impl KafkaProducer {
    pub fn new(config: &KafkaConfig) -> Result<Self, QueueError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("acks", &config.acks)
            .set("retries", config.retries.to_string())
            .set("message.timeout.ms", "5000")
            .set("compression.type", "lz4")
            .create()
            .map_err(QueueError::Kafka)?;

        Ok(Self { inner: producer })
    }
}

#[async_trait]
impl TbProducer for KafkaProducer {
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError> {
        let record = FutureRecord::to(&msg.topic)
            .key(&msg.key)
            .payload(&msg.value);

        match self.inner.send(record, Timeout::After(Duration::from_secs(5))).await {
            Ok((partition, offset)) => {
                debug!(
                    topic = %msg.topic,
                    partition = partition,
                    offset = offset,
                    "Kafka: message delivered"
                );
                Ok(())
            }
            Err((e, _)) => {
                error!(topic = %msg.topic, error = %e, "Kafka: delivery failure");
                Err(QueueError::Kafka(e))
            }
        }
    }
}
