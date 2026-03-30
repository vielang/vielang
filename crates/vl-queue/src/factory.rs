use std::sync::Arc;
use vl_config::{QueueConfig, QueueType};
use crate::{
    memory::{InMemoryBus, InMemoryProducer},
    QueueError, TbConsumer, TbProducer,
};

/// Shared in-memory bus singleton — created once and reused by all producers/consumers.
/// For Kafka, each producer/consumer connects independently to the cluster.
static MEMORY_BUS: std::sync::OnceLock<InMemoryBus> = std::sync::OnceLock::new();

fn get_bus() -> InMemoryBus {
    MEMORY_BUS.get_or_init(|| InMemoryBus::new(1024)).clone()
}

/// Create a producer from the queue config.
pub fn create_producer(config: &QueueConfig) -> Result<Arc<dyn TbProducer>, QueueError> {
    match config.queue_type {
        // ── In-Memory ─────────────────────────────────────────────────────────
        QueueType::InMemory => {
            Ok(Arc::new(InMemoryProducer::new(get_bus())))
        }

        // ── Kafka ─────────────────────────────────────────────────────────────
        QueueType::Kafka => {
            #[cfg(feature = "kafka")]
            {
                let kafka_cfg = config.kafka.as_ref()
                    .ok_or_else(|| QueueError::Send("Kafka config missing".into()))?;
                let producer = crate::kafka::KafkaProducer::new(kafka_cfg)?;
                Ok(Arc::new(producer))
            }
            #[cfg(not(feature = "kafka"))]
            {
                tracing::warn!("Kafka queue type requested but 'kafka' feature not enabled — falling back to in-memory");
                Ok(Arc::new(InMemoryProducer::new(get_bus())))
            }
        }

        // ── RabbitMQ ──────────────────────────────────────────────────────────
        QueueType::RabbitMq => {
            #[cfg(feature = "rabbitmq")]
            {
                let rmq_cfg = config.rabbitmq.as_ref()
                    .ok_or_else(|| QueueError::Send("RabbitMQ config missing".into()))?;
                let producer = tokio::runtime::Handle::current()
                    .block_on(crate::rabbitmq::RabbitMqProducer::new(rmq_cfg))?;
                Ok(Arc::new(producer))
            }
            #[cfg(not(feature = "rabbitmq"))]
            {
                tracing::warn!("RabbitMQ queue type requested but 'rabbitmq' feature not enabled — falling back to in-memory");
                Ok(Arc::new(InMemoryProducer::new(get_bus())))
            }
        }

        // ── Redis Streams ─────────────────────────────────────────────────────
        QueueType::RedisStreams => {
            #[cfg(feature = "redis-streams")]
            {
                let rs_cfg = config.redis_streams.as_ref()
                    .ok_or_else(|| QueueError::Send("Redis Streams config missing".into()))?;
                let producer = tokio::runtime::Handle::current()
                    .block_on(crate::redis_streams::RedisStreamsProducer::new(rs_cfg))?;
                Ok(Arc::new(producer))
            }
            #[cfg(not(feature = "redis-streams"))]
            {
                tracing::warn!("Redis Streams queue type requested but 'redis-streams' feature not enabled — falling back to in-memory");
                Ok(Arc::new(InMemoryProducer::new(get_bus())))
            }
        }

        // ── AWS SQS ───────────────────────────────────────────────────────────
        QueueType::Sqs => {
            #[cfg(feature = "sqs")]
            {
                let sqs_cfg = config.sqs.as_ref()
                    .ok_or_else(|| QueueError::Send("SQS config missing".into()))?;
                let producer = tokio::runtime::Handle::current()
                    .block_on(crate::sqs::SqsProducer::new(sqs_cfg))?;
                Ok(Arc::new(producer))
            }
            #[cfg(not(feature = "sqs"))]
            {
                tracing::warn!("SQS queue type requested but 'sqs' feature not enabled — falling back to in-memory");
                Ok(Arc::new(InMemoryProducer::new(get_bus())))
            }
        }

        // ── Persistent (PostgreSQL-backed) ────────────────────────────────────
        // The persistent backend requires a QueueMessageDao (PgPool).
        // Use vl_queue::create_persistent_producer(dao) directly in vl-api/main.rs.
        QueueType::Persistent => {
            tracing::error!(
                "queue_type = \"persistent\" cannot be created via create_producer(config) — \
                 call vl_queue::create_persistent_producer(dao) with a QueueMessageDao instead"
            );
            Err(QueueError::Send(
                "Persistent queue requires a database pool; use create_persistent_producer(dao)".into(),
            ))
        }
    }
}

/// Create a consumer subscribed to the given topics.
pub fn create_consumer(
    config:   &QueueConfig,
    topics:   &[&str],
    group_id: &str,
) -> Result<Box<dyn TbConsumer>, QueueError> {
    match config.queue_type {
        // ── In-Memory ─────────────────────────────────────────────────────────
        QueueType::InMemory => {
            Ok(Box::new(get_bus().subscribe(topics, group_id)))
        }

        // ── Kafka ─────────────────────────────────────────────────────────────
        QueueType::Kafka => {
            #[cfg(feature = "kafka")]
            {
                let kafka_cfg = config.kafka.as_ref()
                    .ok_or_else(|| QueueError::Receive("Kafka config missing".into()))?;
                let consumer = crate::kafka::KafkaConsumer::new(kafka_cfg, topics, group_id)?;
                Ok(Box::new(consumer))
            }
            #[cfg(not(feature = "kafka"))]
            {
                tracing::warn!("Kafka queue type requested but 'kafka' feature not enabled — falling back to in-memory");
                Ok(Box::new(get_bus().subscribe(topics, group_id)))
            }
        }

        // ── RabbitMQ ──────────────────────────────────────────────────────────
        QueueType::RabbitMq => {
            #[cfg(feature = "rabbitmq")]
            {
                let rmq_cfg = config.rabbitmq.as_ref()
                    .ok_or_else(|| QueueError::Receive("RabbitMQ config missing".into()))?;
                // Use first topic as the queue name; multi-topic consumers need one consumer per topic
                let queue_name = topics.first().copied().unwrap_or("default");
                let consumer = tokio::runtime::Handle::current()
                    .block_on(crate::rabbitmq::RabbitMqConsumer::new(rmq_cfg, queue_name, group_id))?;
                Ok(Box::new(consumer))
            }
            #[cfg(not(feature = "rabbitmq"))]
            {
                tracing::warn!("RabbitMQ queue type requested but 'rabbitmq' feature not enabled — falling back to in-memory");
                Ok(Box::new(get_bus().subscribe(topics, group_id)))
            }
        }

        // ── Redis Streams ─────────────────────────────────────────────────────
        QueueType::RedisStreams => {
            #[cfg(feature = "redis-streams")]
            {
                let rs_cfg = config.redis_streams.as_ref()
                    .ok_or_else(|| QueueError::Receive("Redis Streams config missing".into()))?;
                let consumer = tokio::runtime::Handle::current()
                    .block_on(crate::redis_streams::RedisStreamsConsumer::new(rs_cfg, topics, group_id))?;
                Ok(Box::new(consumer))
            }
            #[cfg(not(feature = "redis-streams"))]
            {
                tracing::warn!("Redis Streams queue type requested but 'redis-streams' feature not enabled — falling back to in-memory");
                Ok(Box::new(get_bus().subscribe(topics, group_id)))
            }
        }

        // ── AWS SQS ───────────────────────────────────────────────────────────
        QueueType::Sqs => {
            #[cfg(feature = "sqs")]
            {
                let sqs_cfg = config.sqs.as_ref()
                    .ok_or_else(|| QueueError::Receive("SQS config missing".into()))?;
                let topic    = topics.first().copied().unwrap_or("default");
                let consumer = tokio::runtime::Handle::current()
                    .block_on(crate::sqs::SqsConsumer::new(sqs_cfg, topic, group_id))?;
                Ok(Box::new(consumer))
            }
            #[cfg(not(feature = "sqs"))]
            {
                tracing::warn!("SQS queue type requested but 'sqs' feature not enabled — falling back to in-memory");
                Ok(Box::new(get_bus().subscribe(topics, group_id)))
            }
        }

        // ── Persistent (PostgreSQL-backed) ────────────────────────────────────
        QueueType::Persistent => {
            tracing::error!(
                "queue_type = \"persistent\" cannot be created via create_consumer(config) — \
                 call vl_queue::create_persistent_consumer(dao, ..) with a QueueMessageDao instead"
            );
            Err(QueueError::Receive(
                "Persistent queue requires a database pool; use create_persistent_consumer(dao, ..)".into(),
            ))
        }
    }
}
