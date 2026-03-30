use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueInfo {
    pub name: String,
    pub topic: String,
    pub poll_interval: i32,
    pub partitions: i32,
    pub consumer_per_partition: bool,
    pub pack_processing_timeout: i64,
    pub submit_strategy: QueueSubmitStrategy,
    pub processing_strategy: QueueProcessingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueueSubmitStrategy {
    #[default]
    Burst,
    Batch,
    SequentialByOriginator,
    SequentialByTenant,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueueProcessingStrategy {
    #[default]
    SkipAllFailures,
    Retry,
    RetryFailed,
    RetryAllFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStats {
    pub id: Uuid,
    pub queue_name: String,
    pub messages_total: i64,
    pub messages_per_second: f64,
    pub consumers_total: i32,
    pub lag: i64,
    pub collected_at: i64,
}
