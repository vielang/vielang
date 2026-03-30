use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Send error: {0}")]
    Send(String),

    #[error("Receive error: {0}")]
    Receive(String),

    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    #[error("Consumer lag: {0} messages dropped")]
    Lagged(u64),

    #[error("Queue closed")]
    Closed,

    #[cfg(feature = "kafka")]
    #[error("Kafka error: {0}")]
    Kafka(#[from] rdkafka::error::KafkaError),

    #[error("RabbitMQ error: {0}")]
    RabbitMq(String),

    #[error("Redis Streams error: {0}")]
    RedisStreams(String),

    #[error("SQS error: {0}")]
    Sqs(String),
}
