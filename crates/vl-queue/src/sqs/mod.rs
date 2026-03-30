//! AWS SQS queue backend — requires `--features sqs`.
//!
//! Mapping:
//! - Topic      → SQS queue name (with optional prefix)
//! - FIFO queue → message_group_id = msg.key for ordering
//! - Commit     → DeleteMessageBatch (batch delete receipt handles)

use std::collections::HashMap;

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_sqs::{
    Client as SqsClient,
    config::Region,
    types::DeleteMessageBatchRequestEntry,
};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use tracing::{debug, error, instrument};

use vl_config::SqsConfig;
use crate::{QueueError, QueueMsg, TbConsumer, TbProducer};

// ── Producer ──────────────────────────────────────────────────────────────────

pub struct SqsProducer {
    client:       SqsClient,
    queue_prefix: String,
    fifo:         bool,
    /// Cache: topic → queue URL
    queue_urls:   tokio::sync::Mutex<HashMap<String, String>>,
}

impl SqsProducer {
    pub async fn new(config: &SqsConfig) -> Result<Self, QueueError> {
        let client = build_client(config).await;
        Ok(Self {
            client,
            queue_prefix: config.queue_prefix.clone(),
            fifo:         config.fifo,
            queue_urls:   tokio::sync::Mutex::new(HashMap::new()),
        })
    }

    async fn queue_url(&self, topic: &str) -> Result<String, QueueError> {
        let mut cache = self.queue_urls.lock().await;
        if let Some(url) = cache.get(topic) {
            return Ok(url.clone());
        }
        let queue_name = self.queue_name(topic);
        let resp = self.client
            .get_queue_url()
            .queue_name(&queue_name)
            .send()
            .await
            .map_err(|e| QueueError::Sqs(e.to_string()))?;

        let url = resp.queue_url().unwrap_or_default().to_string();
        cache.insert(topic.to_string(), url.clone());
        Ok(url)
    }

    fn queue_name(&self, topic: &str) -> String {
        let base = format!("{}{}", self.queue_prefix, topic.replace('.', "-"));
        if self.fifo {
            format!("{}.fifo", base)
        } else {
            base
        }
    }
}

#[async_trait]
impl TbProducer for SqsProducer {
    #[instrument(skip(self, msg), fields(topic = %msg.topic))]
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError> {
        let url = self.queue_url(&msg.topic).await?;

        let mut req = self.client
            .send_message()
            .queue_url(&url)
            .message_body(B64.encode(&msg.value));

        if self.fifo {
            // FIFO queues require MessageGroupId (and optionally MessageDeduplicationId)
            req = req.message_group_id(&msg.key);
        }

        req.send()
            .await
            .map_err(|e| QueueError::Sqs(e.to_string()))?;

        debug!(topic = %msg.topic, "SQS: SendMessage OK");
        Ok(())
    }
}

// ── Consumer ─────────────────────────────────────────────────────────────────

pub struct SqsConsumer {
    client:       SqsClient,
    queue_url:    String,
    max_messages: i32,
    wait_seconds: i32,
    group_id:     String,
    /// Receipt handles pending delete on commit()
    pending:      tokio::sync::Mutex<Vec<String>>,
}

impl SqsConsumer {
    pub async fn new(
        config:    &SqsConfig,
        topic:     &str,
        group_id:  &str,
    ) -> Result<Self, QueueError> {
        let client = build_client(config).await;

        // Resolve queue URL
        let queue_name = {
            let base = format!("{}{}", config.queue_prefix, topic.replace('.', "-"));
            if config.fifo { format!("{}.fifo", base) } else { base }
        };
        let resp = client
            .get_queue_url()
            .queue_name(&queue_name)
            .send()
            .await
            .map_err(|e| QueueError::Sqs(e.to_string()))?;
        let queue_url = resp.queue_url().unwrap_or_default().to_string();

        Ok(Self {
            client,
            queue_url,
            max_messages: config.max_messages,
            wait_seconds: config.wait_seconds,
            group_id:     group_id.to_string(),
            pending:      tokio::sync::Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl TbConsumer for SqsConsumer {
    #[instrument(skip(self))]
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError> {
        let resp = self.client
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(self.max_messages)
            .wait_time_seconds(self.wait_seconds)
            .send()
            .await
            .map_err(|e| QueueError::Sqs(e.to_string()))?;

        let sqs_msgs = resp.messages();
        let mut msgs = Vec::with_capacity(sqs_msgs.len());
        let mut pendings = self.pending.lock().await;

        for m in sqs_msgs {
            let receipt = m.receipt_handle().unwrap_or_default().to_string();
            let body    = m.body().unwrap_or_default();
            let value   = B64.decode(body).unwrap_or_else(|_| body.as_bytes().to_vec());

            debug!(receipt = %receipt, "SQS: received message");

            msgs.push(QueueMsg {
                topic:   self.queue_url.clone(),
                key:     String::new(),
                value,
                headers: Default::default(),
                ack_id:  Some(receipt.clone()),
            });

            pendings.push(receipt);
        }

        Ok(msgs)
    }

    #[instrument(skip(self))]
    async fn commit(&mut self) -> Result<(), QueueError> {
        let mut pending = self.pending.lock().await;
        if pending.is_empty() {
            return Ok(());
        }

        // DeleteMessageBatch supports up to 10 entries per call
        for chunk in pending.chunks(10) {
            let entries: Vec<DeleteMessageBatchRequestEntry> = chunk
                .iter()
                .enumerate()
                .filter_map(|(i, handle)| {
                    DeleteMessageBatchRequestEntry::builder()
                        .id(i.to_string())
                        .receipt_handle(handle)
                        .build()
                        .ok()
                })
                .collect();

            let result = self.client
                .delete_message_batch()
                .queue_url(&self.queue_url)
                .set_entries(Some(entries))
                .send()
                .await
                .map_err(|e| QueueError::Sqs(e.to_string()))?;

            if !result.failed().is_empty() {
                for failure in result.failed() {
                    error!(
                        id = %failure.id(),
                        code = %failure.code(),
                        "SQS: DeleteMessageBatch partial failure"
                    );
                }
            }

            debug!(count = chunk.len(), "SQS: DeleteMessageBatch committed");
        }

        pending.clear();
        Ok(())
    }

    fn group_id(&self) -> &str {
        &self.group_id
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn build_client(config: &SqsConfig) -> SqsClient {
    let region = Region::new(config.region.clone());

    // Prefer environment credentials (AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY)
    // Fall back to explicit config values if provided
    if !config.access_key.is_empty() {
        let creds = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "vielang-config",
        );
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(creds)
            .load()
            .await;
        SqsClient::new(&sdk_config)
    } else {
        // Use default credential chain (env vars, instance profile, etc.)
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .load()
            .await;
        SqsClient::new(&sdk_config)
    }
}
