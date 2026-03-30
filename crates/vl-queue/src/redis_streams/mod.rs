//! Redis Streams queue backend — requires `--features redis-streams`.
//!
//! Mapping:
//! - Topic         → Stream key (e.g. "vl.rule-engine")
//! - Consumer group → XGROUP name
//! - Commit        → XACK per entry ID
//!
//! Uses XADD (producer), XREADGROUP (consumer), XACK (commit),
//! and XAUTOCLAIM to reclaim stuck pending messages.

use std::collections::HashMap;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use fred::prelude::*;
use fred::types::streams::{XCap, XCapKind, XCapTrim, XReadResponse, XReadValue};
use tracing::{debug, instrument};

use vl_config::RedisStreamsConfig;
use crate::{QueueError, QueueMsg, TbConsumer, TbProducer};

// ── Producer ─────────────────────────────────────────────────────────────────

pub struct RedisStreamsProducer {
    client:  Client,
    max_len: usize,
}

impl RedisStreamsProducer {
    pub async fn new(config: &RedisStreamsConfig) -> Result<Self, QueueError> {
        let client = build_client(&config.url).await?;
        Ok(Self {
            client,
            max_len: config.max_len,
        })
    }
}

#[async_trait]
impl TbProducer for RedisStreamsProducer {
    #[instrument(skip(self, msg), fields(topic = %msg.topic))]
    async fn send(&self, msg: &QueueMsg) -> Result<(), QueueError> {
        // XADD <stream> MAXLEN ~ <max_len> * key <key> payload <b64-value>
        let cap = XCap::try_from((XCapKind::MaxLen, XCapTrim::AlmostExact, self.max_len as i64))
            .map_err(|e| QueueError::RedisStreams(e.to_string()))?;

        let _id: String = self.client
            .xadd(
                msg.topic.as_str(),
                false,   // NOMKSTREAM = false → auto-create stream
                cap,
                "*",     // auto-generate entry ID
                vec![
                    ("key",     msg.key.as_str()),
                    ("payload", B64.encode(&msg.value).as_str()),
                ],
            )
            .await
            .map_err(redis_err)?;

        debug!(topic = %msg.topic, entry_id = %_id, "Redis Streams: XADD");
        Ok(())
    }
}

// ── Consumer ─────────────────────────────────────────────────────────────────

pub struct RedisStreamsConsumer {
    client:       Client,
    stream_keys:  Vec<String>,
    group_name:   String,
    consumer_id:  String,
    batch_size:   usize,
    pending_ttl:  u64,   // ms before XAUTOCLAIM reclaims pending messages
    /// Entry IDs to XACK on next commit(), grouped by stream key
    pending_acks: tokio::sync::Mutex<HashMap<String, Vec<String>>>,
    group_id:     String,
}

impl RedisStreamsConsumer {
    pub async fn new(
        config:   &RedisStreamsConfig,
        topics:   &[&str],
        group_id: &str,
    ) -> Result<Self, QueueError> {
        let client = build_client(&config.url).await?;

        // Ensure consumer group exists for every stream (XGROUP CREATE MKSTREAM)
        for topic in topics {
            ensure_group(&client, topic, &config.group).await?;
        }

        // Unique consumer name = config name + group_id suffix
        let consumer_id = format!("{}-{}", config.consumer_name, group_id);

        Ok(Self {
            client,
            stream_keys:  topics.iter().map(|t| t.to_string()).collect(),
            group_name:   config.group.clone(),
            consumer_id,
            batch_size:   config.batch_size,
            pending_ttl:  config.pending_ttl_s * 1000,
            pending_acks: tokio::sync::Mutex::new(HashMap::new()),
            group_id:     group_id.to_string(),
        })
    }
}

#[async_trait]
impl TbConsumer for RedisStreamsConsumer {
    #[instrument(skip(self))]
    async fn poll(&mut self) -> Result<Vec<QueueMsg>, QueueError> {
        let keys: Vec<&str> = self.stream_keys.iter().map(|s| s.as_str()).collect();
        // ">" = new messages not yet delivered to this consumer group
        let ids: Vec<&str>  = keys.iter().map(|_| ">").collect();

        // XReadResponse<String, String, String, String> =
        //   HashMap<stream_key, Vec<(entry_id, HashMap<field, value>)>>
        let results: XReadResponse<String, String, String, String> = self.client
            .xreadgroup_map(
                self.group_name.as_str(),
                self.consumer_id.as_str(),
                Some(self.batch_size as u64),
                None,   // no BLOCK — return immediately
                false,  // NOACK = false
                keys.clone(),
                ids,
            )
            .await
            .map_err(redis_err)?;

        let mut msgs     = Vec::new();
        let mut pendings = self.pending_acks.lock().await;

        for (stream_key, entries) in &results {
            for (entry_id, fields) in entries {
                let key = fields.get("key").cloned().unwrap_or_default();
                let payload = fields.get("payload")
                    .and_then(|s| B64.decode(s).ok())
                    .unwrap_or_default();

                debug!(stream = %stream_key, id = %entry_id, "Redis Streams: XREADGROUP entry");

                msgs.push(QueueMsg {
                    topic:   stream_key.clone(),
                    key,
                    value:   payload,
                    headers: Default::default(),
                    ack_id:  Some(entry_id.clone()),
                });

                pendings.entry(stream_key.clone())
                    .or_default()
                    .push(entry_id.clone());
            }
        }

        // Try to reclaim any stuck pending messages (XAUTOCLAIM) on every poll
        for stream_key in &self.stream_keys {
            // xautoclaim_values returns (next_cursor: String, entries: Vec<XReadValue<Ri, Rk, Rv>>)
            // where XReadValue<Ri, Rk, Rv> = (id: Ri, fields: HashMap<Rk, Rv>)
            let (_, claimed): (String, Vec<XReadValue<String, String, String>>) = self.client
                .xautoclaim_values(
                    stream_key.as_str(),
                    self.group_name.as_str(),
                    self.consumer_id.as_str(),
                    self.pending_ttl,
                    "0-0",
                    Some(self.batch_size as u64),
                    false,
                )
                .await
                .map_err(redis_err)?;

            for (entry_id, fields) in claimed {
                let key = fields.get("key").cloned().unwrap_or_default();
                let payload = fields.get("payload")
                    .and_then(|s| B64.decode(s).ok())
                    .unwrap_or_default();

                if !payload.is_empty() {
                    debug!(stream = %stream_key, id = %entry_id, "Redis Streams: XAUTOCLAIM reclaim");
                    msgs.push(QueueMsg {
                        topic:   stream_key.clone(),
                        key,
                        value:   payload,
                        headers: Default::default(),
                        ack_id:  Some(entry_id.clone()),
                    });
                    pendings.entry(stream_key.clone())
                        .or_default()
                        .push(entry_id);
                }
            }
        }

        Ok(msgs)
    }

    #[instrument(skip(self))]
    async fn commit(&mut self) -> Result<(), QueueError> {
        let mut pendings = self.pending_acks.lock().await;
        if pendings.is_empty() {
            return Ok(());
        }

        for (stream_key, ids) in pendings.drain() {
            if ids.is_empty() { continue; }
            let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
            let _: i64 = self.client
                .xack(stream_key.as_str(), self.group_name.as_str(), id_refs)
                .await
                .map_err(redis_err)?;
            debug!(stream = %stream_key, count = ids.len(), "Redis Streams: XACK committed");
        }

        Ok(())
    }

    fn group_id(&self) -> &str {
        &self.group_id
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn build_client(url: &str) -> Result<Client, QueueError> {
    let config = Config::from_url(url).map_err(|e| QueueError::RedisStreams(e.to_string()))?;
    let client = Builder::from_config(config)
        .build()
        .map_err(|e| QueueError::RedisStreams(e.to_string()))?;
    client.init().await.map_err(redis_err)?;
    Ok(client)
}

async fn ensure_group(
    client:     &Client,
    stream_key: &str,
    group_name: &str,
) -> Result<(), QueueError> {
    // XGROUP CREATE <key> <group> $ MKSTREAM
    // $ = only new messages; MKSTREAM = create stream if not exists
    let result: Result<Value, _> = client
        .xgroup_create(stream_key, group_name, "$", true)
        .await;

    match result {
        Ok(_)  => Ok(()),
        Err(e) if e.to_string().contains("BUSYGROUP") => {
            // Group already exists — not an error
            Ok(())
        }
        Err(e) => Err(QueueError::RedisStreams(e.to_string())),
    }
}

fn redis_err(e: fred::error::Error) -> QueueError {
    QueueError::RedisStreams(e.to_string())
}
