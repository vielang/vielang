pub mod error;
pub mod factory;
pub mod memory;
pub mod message;
pub mod topics;

#[cfg(feature = "kafka")]
pub mod kafka;

#[cfg(feature = "rabbitmq")]
pub mod rabbitmq;

#[cfg(feature = "redis-streams")]
pub mod redis_streams;

#[cfg(feature = "sqs")]
pub mod sqs;

#[cfg(feature = "persistent")]
pub mod persistent;

#[cfg(test)]
mod tests;

pub use error::QueueError;
pub use factory::{create_consumer, create_producer};
pub use message::QueueMsg;

#[cfg(feature = "persistent")]
pub use persistent::{create_persistent_consumer, create_persistent_producer};

use async_trait::async_trait;
use vl_core::entities::TbMsg;

// ── Producer trait ────────────────────────────────────────────────────────────

/// Publishes messages to a queue topic — object-safe, no associated types.
#[async_trait]
pub trait TbProducer: Send + Sync {
    /// Send a raw QueueMsg.
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError>;

    /// Convenience: serialize TbMsg and send to the given topic.
    async fn send_tb_msg(&self, topic: &str, tb_msg: &TbMsg) -> Result<(), QueueError> {
        let msg = QueueMsg::from_tb_msg(topic, tb_msg)?;
        self.send(&msg).await
    }
}

// ── Consumer trait ────────────────────────────────────────────────────────────

/// Consumes messages from one or more topics — object-safe.
#[async_trait]
pub trait TbConsumer: Send {
    /// Poll for available messages. Returns immediately with whatever is buffered.
    /// Returns an empty Vec if no messages are available.
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError>;

    /// Acknowledge processed messages (no-op for in-memory, commit offset for Kafka).
    async fn commit(&mut self) -> Result<(), QueueError>;

    /// Logical name for this consumer / consumer group.
    fn group_id(&self) -> &str;
}
