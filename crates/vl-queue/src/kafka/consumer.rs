use async_trait::async_trait;
use rdkafka::{
    config::ClientConfig,
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::Message,
};
use std::time::Duration;
use tracing::{debug, warn};

use vl_config::KafkaConfig;
use crate::{QueueError, QueueMsg, TbConsumer};

pub struct KafkaConsumer {
    inner:    StreamConsumer,
    group_id: String,
}

impl KafkaConsumer {
    pub fn new(config: &KafkaConfig, topics: &[&str], group_id: &str) -> Result<Self, QueueError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "6000")
            .set("max.poll.interval.ms", "300000")
            .create()
            .map_err(QueueError::Kafka)?;

        consumer.subscribe(topics).map_err(QueueError::Kafka)?;

        Ok(Self {
            inner:    consumer,
            group_id: group_id.to_string(),
        })
    }
}

#[async_trait]
impl TbConsumer for KafkaConsumer {
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError> {
        let mut msgs = Vec::new();

        // Drain up to 100 messages per poll call, non-blocking
        for _ in 0..100 {
            match tokio::time::timeout(
                Duration::from_millis(10),
                self.inner.recv(),
            ).await {
                Ok(Ok(message)) => {
                    let topic = message.topic().to_string();
                    let key = message
                        .key()
                        .map(|k| String::from_utf8_lossy(k).to_string())
                        .unwrap_or_default();
                    let value = message
                        .payload()
                        .map(|p| p.to_vec())
                        .unwrap_or_default();

                    debug!(topic = %topic, key = %key, "Kafka: received message");
                    msgs.push(QueueMsg::raw(topic, key, value));
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "Kafka: receive error");
                    return Err(QueueError::Kafka(e));
                }
                Err(_) => break, // timeout — no more messages right now
            }
        }

        Ok(msgs)
    }

    async fn commit(&mut self) -> Result<(), QueueError> {
        self.inner
            .commit_consumer_state(CommitMode::Async)
            .map_err(QueueError::Kafka)?;
        Ok(())
    }

    fn group_id(&self) -> &str {
        &self.group_id
    }
}
