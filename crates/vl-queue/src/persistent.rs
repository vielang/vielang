//! PostgreSQL-backed persistent queue backend.
//!
//! Survives server restarts: messages are written to the `queue_message` table
//! and replayed by consumers after any crash or restart.
//!
//! Enabled by the `persistent` feature flag.

use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tracing::{instrument, warn};
use uuid::Uuid;
use vl_dao::{NewQueueMessage, QueueMessageDao};

use crate::{QueueError, QueueMsg, TbConsumer, TbProducer};

// ── PersistentProducer ────────────────────────────────────────────────────────

/// Writes messages to the `queue_message` PostgreSQL table.
pub struct PersistentProducer {
    dao: Arc<QueueMessageDao>,
}

impl PersistentProducer {
    pub fn new(dao: Arc<QueueMessageDao>) -> Self {
        Self { dao }
    }
}

#[async_trait]
impl TbProducer for PersistentProducer {
    #[instrument(skip(self, msg))]
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError> {
        let headers = if msg.headers.is_empty() {
            None
        } else {
            let map: serde_json::Map<String, serde_json::Value> = msg
                .headers
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            Some(serde_json::Value::Object(map))
        };

        let topic = msg.topic.clone();
        self.dao
            .save_batch(&[NewQueueMessage {
                topic:        &msg.topic,
                partition_id: 0,
                payload:      &msg.value,
                headers,
                created_time: chrono::Utc::now().timestamp_millis(),
            }])
            .await
            .map_err(|e| QueueError::Send(e.to_string()))?;

        metrics::counter!("vielang_queue_publish_total", "topic" => topic, "backend" => "persistent").increment(1);
        Ok(())
    }
}

// ── PersistentConsumer ────────────────────────────────────────────────────────

/// Polls and acks messages from the `queue_message` PostgreSQL table.
///
/// Tracks the highest seen offset per topic in a `DashMap` so each poll
/// fetches only new messages. On restart the consumer starts from offset 0,
/// which replays all unacked messages — giving at-least-once delivery.
pub struct PersistentConsumer {
    dao:          Arc<QueueMessageDao>,
    topics:       Vec<String>,
    consumer_id:  String,
    last_offsets: Arc<DashMap<String, i64>>,
    batch_size:   i32,
    /// Accumulated pending acks: Vec<(id, topic)>
    pending_acks: tokio::sync::Mutex<Vec<(Uuid, String)>>,
}

impl PersistentConsumer {
    pub fn new(
        dao:         Arc<QueueMessageDao>,
        topics:      &[&str],
        consumer_id: &str,
        batch_size:  i32,
    ) -> Self {
        Self {
            dao,
            topics:       topics.iter().map(|s| s.to_string()).collect(),
            consumer_id:  consumer_id.to_owned(),
            last_offsets: Arc::new(DashMap::new()),
            batch_size,
            pending_acks: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl TbConsumer for PersistentConsumer {
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError> {
        let mut result = Vec::new();

        for topic in &self.topics {
            let after_offset = self
                .last_offsets
                .get(topic)
                .map(|v| *v)
                .unwrap_or(0);

            let messages = self
                .dao
                .poll(topic, &self.consumer_id, after_offset, self.batch_size)
                .await
                .map_err(|e| QueueError::Receive(e.to_string()))?;

            if let Some(max_offset) = messages.iter().map(|m| m.offset_value).max() {
                self.last_offsets.insert(topic.clone(), max_offset);
            }

            let mut pending = self.pending_acks.lock().await;
            for msg in &messages {
                pending.push((msg.id, topic.clone()));
            }

            for msg in messages {
                let headers: std::collections::HashMap<String, String> = msg
                    .headers
                    .as_ref()
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
                            .collect()
                    })
                    .unwrap_or_default();

                result.push(QueueMsg {
                    topic:   msg.topic,
                    key:     String::new(),
                    value:   msg.payload,
                    headers,
                    ack_id:  Some(msg.id.to_string()),
                });
            }
        }

        if !result.is_empty() {
            metrics::counter!("vielang_queue_consume_total", "backend" => "persistent", "group" => self.consumer_id.clone())
                .increment(result.len() as u64);
        }
        Ok(result)
    }

    async fn commit(&mut self) -> Result<(), QueueError> {
        let mut pending = self.pending_acks.lock().await;
        if pending.is_empty() {
            return Ok(());
        }

        let ids: Vec<Uuid> = pending.iter().map(|(id, _)| *id).collect();
        let acked = self
            .dao
            .ack_batch(&ids, &self.consumer_id)
            .await
            .map_err(|e| QueueError::Send(e.to_string()))?;

        if acked < ids.len() as u64 {
            warn!(
                "PersistentConsumer: acked {}/{} messages (some may already be acked)",
                acked,
                ids.len()
            );
        }

        pending.clear();
        Ok(())
    }

    fn group_id(&self) -> &str {
        &self.consumer_id
    }
}

// ── Factory helpers ───────────────────────────────────────────────────────────

/// Create a persistent producer backed by the given `QueueMessageDao`.
pub fn create_persistent_producer(dao: Arc<QueueMessageDao>) -> Arc<dyn TbProducer> {
    Arc::new(PersistentProducer::new(dao))
}

/// Create a persistent consumer subscribed to the given topics.
pub fn create_persistent_consumer(
    dao:         Arc<QueueMessageDao>,
    topics:      &[&str],
    group_id:    &str,
    batch_size:  i32,
) -> Box<dyn TbConsumer> {
    Box::new(PersistentConsumer::new(dao, topics, group_id, batch_size))
}
