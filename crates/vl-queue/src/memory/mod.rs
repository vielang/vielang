pub mod consumer;

pub use consumer::MemoryConsumer;

use std::sync::Arc;
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing::debug;

use crate::{QueueError, QueueMsg, TbProducer};

const DEFAULT_CAPACITY: usize = 1024;

// ── Shared Bus ────────────────────────────────────────────────────────────────

/// Shared state for the in-memory queue bus.
/// Each topic maps to a broadcast channel — supports multiple consumer groups
/// receiving the same messages (similar to Kafka consumer groups).
#[derive(Clone)]
pub struct InMemoryBus {
    /// topic → broadcast sender
    topics:   Arc<DashMap<String, broadcast::Sender<QueueMsg>>>,
    capacity: usize,
}

impl InMemoryBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            topics:   Arc::new(DashMap::new()),
            capacity,
        }
    }

    /// Get or create a broadcast sender for the given topic.
    pub fn get_or_create(&self, topic: &str) -> broadcast::Sender<QueueMsg> {
        if let Some(tx) = self.topics.get(topic) {
            return tx.clone();
        }
        let (tx, _) = broadcast::channel(self.capacity);
        self.topics.entry(topic.to_string()).or_insert(tx).clone()
    }

    /// Subscribe a new receiver for the given topics.
    /// Returns a MemoryConsumer that reads from all subscribed topics.
    pub fn subscribe(&self, topics: &[&str], group_id: impl Into<String>) -> MemoryConsumer {
        let receivers: Vec<broadcast::Receiver<QueueMsg>> = topics
            .iter()
            .map(|t| self.get_or_create(t).subscribe())
            .collect();
        MemoryConsumer::new(receivers, group_id.into())
    }
}

impl Default for InMemoryBus {
    fn default() -> Self { Self::new(DEFAULT_CAPACITY) }
}

// ── Producer ──────────────────────────────────────────────────────────────────

/// In-memory producer — sends to the broadcast channel for each topic.
#[derive(Clone)]
pub struct InMemoryProducer {
    bus: InMemoryBus,
}

impl InMemoryProducer {
    pub fn new(bus: InMemoryBus) -> Self { Self { bus } }
}

#[async_trait]
impl TbProducer for InMemoryProducer {
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError> {
        let tx = self.bus.get_or_create(&msg.topic);
        let topic = msg.topic.clone();
        match tx.send(msg.clone()) {
            Ok(n) => {
                debug!(topic = %topic, receivers = n, "Queue: sent message");
                metrics::counter!("vielang_queue_publish_total", "topic" => topic, "backend" => "memory").increment(1);
                Ok(())
            }
            // No active receivers — not an error, messages are just dropped
            Err(_) => {
                debug!(topic = %topic, "Queue: no receivers on topic, message dropped");
                metrics::counter!("vielang_queue_publish_total", "topic" => topic, "backend" => "memory").increment(1);
                Ok(())
            }
        }
    }
}
