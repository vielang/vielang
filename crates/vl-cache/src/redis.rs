//! Redis cache backend (L2) — requires `--features redis`.
//!
//! Uses the `fred` crate for async Redis access.
//! Key format: `{cache_name}:{id}` (khớp ThingsBoard Java).

use std::time::Duration;

use async_trait::async_trait;
use fred::prelude::*;

use crate::{CacheError, TbCache};

pub struct RedisCache {
    client:      RedisClient,
    default_ttl: Duration,
}

impl RedisCache {
    pub async fn new(url: &str, default_ttl: Duration) -> Result<Self, CacheError> {
        let config = RedisConfig::from_url(url)
            .map_err(|e| CacheError::Connection(e.to_string()))?;

        let client = Builder::from_config(config)
            .build()
            .map_err(|e| CacheError::Connection(e.to_string()))?;

        client
            .init()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))?;

        Ok(Self { client, default_ttl })
    }
}

#[async_trait]
impl TbCache for RedisCache {
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let result: Option<Vec<u8>> = self
            .client
            .get(key)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;
        Ok(result)
    }

    async fn put_bytes(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let secs = ttl.unwrap_or(self.default_ttl).as_secs() as i64;
        self.client
            .set(key, value.as_slice(), Some(Expiration::EX(secs)), None, false)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))
    }

    async fn evict(&self, key: &str) -> Result<(), CacheError> {
        let _: i64 = self
            .client
            .del(key)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;
        Ok(())
    }

    async fn evict_by_prefix(&self, prefix: &str) -> Result<usize, CacheError> {
        use fred::interfaces::ScannerInterface;
        use futures::StreamExt as _;

        let pattern = format!("{}*", prefix);
        let mut scanner = self.client.scan(pattern, Some(100u32), None);
        let mut count = 0usize;

        while let Some(result) = scanner.next().await {
            let page = result.map_err(|e| CacheError::Redis(e.to_string()))?;
            if let Some(keys) = page.results() {
                if !keys.is_empty() {
                    count += keys.len();
                    let key_strings: Vec<String> = keys
                        .iter()
                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                        .collect();
                    if !key_strings.is_empty() {
                        let _: i64 = self
                            .client
                            .del(key_strings)
                            .await
                            .map_err(|e| CacheError::Redis(e.to_string()))?;
                    }
                }
            }
        }
        Ok(count)
    }
}
