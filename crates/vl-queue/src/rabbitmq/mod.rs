//! RabbitMQ queue backend — requires `--features rabbitmq`.
//!
//! Mapping:
//! - Topic     → routing key
//! - Consumer  → named queue bound to the exchange
//! - Commit    → basic_ack (per-message ACK)
//! - Dead-letter exchange (DLX) for failed/rejected messages

use std::sync::Arc;

use async_trait::async_trait;
use lapin::{
    options::{
        BasicAckOptions, BasicPublishOptions, BasicQosOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    publisher_confirm::Confirmation,
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};

use vl_config::RabbitMqConfig;
use crate::{QueueError, QueueMsg, TbConsumer, TbProducer};

// ── Producer ──────────────────────────────────────────────────────────────────

/// RabbitMQ producer — publishes messages as persistent AMQP messages.
/// Channel is wrapped in a Mutex because lapin channels are not Send+Sync on their own.
pub struct RabbitMqProducer {
    channel:  Arc<Mutex<Channel>>,
    exchange: String,
}

impl RabbitMqProducer {
    pub async fn new(config: &RabbitMqConfig) -> Result<Self, QueueError> {
        let conn    = connect(config).await?;
        let channel = conn.create_channel().await.map_err(rmq_err)?;

        declare_exchange(&channel, &config.exchange, config.dlx_enabled, &config.dlx_exchange).await?;

        Ok(Self {
            channel:  Arc::new(Mutex::new(channel)),
            exchange: config.exchange.clone(),
        })
    }
}

#[async_trait]
impl TbProducer for RabbitMqProducer {
    #[instrument(skip(self, msg), fields(topic = %msg.topic))]
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError> {
        let channel = self.channel.lock().await;

        let confirm = channel
            .basic_publish(
                &self.exchange,
                &msg.topic,   // routing key
                BasicPublishOptions { mandatory: false, ..Default::default() },
                &msg.value,
                BasicProperties::default()
                    .with_delivery_mode(2)   // persistent
                    .with_content_type("application/octet-stream".into()),
            )
            .await
            .map_err(rmq_err)?
            .await
            .map_err(rmq_err)?;

        match confirm {
            Confirmation::Ack(_) => {
                debug!(topic = %msg.topic, "RabbitMQ: message acked by broker");
                Ok(())
            }
            Confirmation::Nack(_) => {
                error!(topic = %msg.topic, "RabbitMQ: broker NACKed message");
                Err(QueueError::Send("Broker NACKed the message".into()))
            }
            Confirmation::NotRequested => Ok(()),
        }
    }
}

// ── Consumer ─────────────────────────────────────────────────────────────────

/// RabbitMQ consumer — polls a named queue using basic_get.
/// Ack is deferred until `commit()` is called.
pub struct RabbitMqConsumer {
    channel:    Arc<Mutex<Channel>>,
    queue_name: String,
    group_id:   String,
    /// Pending delivery tags from unacked messages
    pending:    Mutex<Vec<u64>>,
}

impl RabbitMqConsumer {
    pub async fn new(
        config:     &RabbitMqConfig,
        queue_name: impl Into<String>,
        group_id:   impl Into<String>,
    ) -> Result<Self, QueueError> {
        let queue_name = queue_name.into();
        let conn       = connect(config).await?;
        let channel    = conn.create_channel().await.map_err(rmq_err)?;

        // QoS: limit unacked messages per consumer
        channel
            .basic_qos(config.prefetch_count, BasicQosOptions::default())
            .await
            .map_err(rmq_err)?;

        // Declare queue + bind to exchange
        declare_exchange(&channel, &config.exchange, config.dlx_enabled, &config.dlx_exchange).await?;
        declare_queue(&channel, &queue_name, config.dlx_enabled, &config.dlx_exchange).await?;
        channel
            .queue_bind(
                &queue_name,
                &config.exchange,
                &queue_name,   // routing key = queue name
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(rmq_err)?;

        Ok(Self {
            channel:    Arc::new(Mutex::new(channel)),
            queue_name,
            group_id:   group_id.into(),
            pending:    Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl TbConsumer for RabbitMqConsumer {
    #[instrument(skip(self))]
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError> {
        let channel = self.channel.lock().await;
        let mut msgs = Vec::new();

        // Drain available messages (non-blocking)
        loop {
            match channel
                .basic_get(&self.queue_name, lapin::options::BasicGetOptions::default())
                .await
                .map_err(rmq_err)?
            {
                Some(delivery) => {
                    let delivery_tag = delivery.delivery.delivery_tag;
                    let topic = delivery
                        .delivery
                        .routing_key
                        .as_str()
                        .to_string();
                    let value = delivery.delivery.data.to_vec();

                    debug!(
                        topic = %topic,
                        delivery_tag = delivery_tag,
                        "RabbitMQ: received message"
                    );

                    msgs.push(QueueMsg {
                        topic,
                        key:     String::new(),
                        value,
                        headers: Default::default(),
                        ack_id:  Some(delivery_tag.to_string()),
                    });

                    // Track pending acks
                    self.pending.lock().await.push(delivery_tag);
                }
                None => break,
            }

            // Stop after 500 messages to avoid blocking too long
            if msgs.len() >= 500 {
                break;
            }
        }

        Ok(msgs)
    }

    #[instrument(skip(self))]
    async fn commit(&mut self) -> Result<(), QueueError> {
        let mut pending = self.pending.lock().await;
        if pending.is_empty() {
            return Ok(());
        }

        let channel = self.channel.lock().await;
        for tag in pending.drain(..) {
            channel
                .basic_ack(tag, BasicAckOptions::default())
                .await
                .map_err(rmq_err)?;
        }

        debug!("RabbitMQ: committed all pending acks");
        Ok(())
    }

    fn group_id(&self) -> &str {
        &self.group_id
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn connect(config: &RabbitMqConfig) -> Result<Connection, QueueError> {
    let conn = Connection::connect(
        &config.url,
        ConnectionProperties::default(),
    )
    .await
    .map_err(rmq_err)?;

    Ok(conn)
}

async fn declare_exchange(
    channel:      &Channel,
    exchange:     &str,
    dlx_enabled:  bool,
    dlx_exchange: &str,
) -> Result<(), QueueError> {
    if exchange.is_empty() {
        return Ok(()); // default exchange — no declaration needed
    }

    channel
        .exchange_declare(
            exchange,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable:     true,
                auto_delete: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(rmq_err)?;

    if dlx_enabled && !dlx_exchange.is_empty() {
        channel
            .exchange_declare(
                dlx_exchange,
                ExchangeKind::Fanout,
                ExchangeDeclareOptions {
                    durable:     true,
                    auto_delete: false,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(rmq_err)?;
    }

    Ok(())
}

async fn declare_queue(
    channel:      &Channel,
    queue_name:   &str,
    dlx_enabled:  bool,
    dlx_exchange: &str,
) -> Result<(), QueueError> {
    let mut args = FieldTable::default();
    if dlx_enabled && !dlx_exchange.is_empty() {
        args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(dlx_exchange.into()),
        );
    }

    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable:     true,
                exclusive:   false,
                auto_delete: false,
                ..Default::default()
            },
            args,
        )
        .await
        .map_err(rmq_err)?;

    Ok(())
}

fn rmq_err(e: lapin::Error) -> QueueError {
    QueueError::RabbitMq(e.to_string())
}
