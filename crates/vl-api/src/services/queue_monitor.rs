use std::sync::Arc;
use std::time::Duration;
use vl_dao::{QueueMessageDao, QueueStatsDao};
use tracing::{error, info};

/// Known queue topics to monitor.
const KNOWN_TOPICS: &[&str] = &[
    vl_queue::topics::VL_CORE,
    vl_queue::topics::VL_RULE_ENGINE,
    vl_queue::topics::VL_TRANSPORT_API_REQUESTS,
    vl_queue::topics::VL_TRANSPORT_API_RESPONSES,
    vl_queue::topics::VL_NOTIFICATIONS,
];

pub struct QueueMonitorService {
    dao: Arc<QueueStatsDao>,
    /// Present when queue_type = persistent — used to report real pending counts.
    queue_msg_dao: Option<Arc<QueueMessageDao>>,
}

impl QueueMonitorService {
    pub fn new(dao: Arc<QueueStatsDao>) -> Self {
        Self { dao, queue_msg_dao: None }
    }

    /// Enable persistent-queue stats (pending message count from DB).
    pub fn with_queue_message_dao(mut self, queue_msg_dao: Arc<QueueMessageDao>) -> Self {
        self.queue_msg_dao = Some(queue_msg_dao);
        self
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            interval.tick().await; // skip first immediate tick
            loop {
                interval.tick().await;
                if let Err(e) = self.collect_stats().await {
                    error!("Queue monitor stats collection failed: {}", e);
                }
            }
        })
    }

    pub async fn collect_stats(&self) -> anyhow::Result<()> {
        for topic in KNOWN_TOPICS {
            // For the persistent backend, read real pending counts from DB.
            // For other backends (in-memory, Kafka, etc.) report 0 lag.
            let lag: i64 = if let Some(ref qm_dao) = self.queue_msg_dao {
                qm_dao.count_pending(topic).await.unwrap_or(0)
            } else {
                0
            };

            let messages_total: i64 = 0;
            let messages_per_second: f64 = 0.0;
            let consumers_total: i32 = 1;

            match self.dao.record_stats(topic, messages_total, messages_per_second, consumers_total, lag).await {
                Ok(_) => {
                    info!("Queue stats recorded for topic '{}': pending={}", topic, lag);
                }
                Err(e) => {
                    error!("Failed to record stats for topic {}: {}", topic, e);
                }
            }
        }
        Ok(())
    }
}
