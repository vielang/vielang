use std::time::Duration;

use async_trait::async_trait;
use moka::sync::Cache;

use crate::{CacheError, TbCache};

/// In-process in-memory cache backed by moka.
///
/// Thread-safe, TTL-based eviction, bounded capacity.
/// Used as L1 in TieredCache or standalone in dev mode.
pub struct MokaCache {
    inner: Cache<String, Vec<u8>>,
    #[allow(dead_code)]
    default_ttl: Duration,
}

impl MokaCache {
    pub fn new(max_size: u64, default_ttl: Duration) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_size)
            .time_to_live(default_ttl)
            .build();
        Self { inner, default_ttl }
    }
}

#[async_trait]
impl TbCache for MokaCache {
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        Ok(self.inner.get(key))
    }

    async fn put_bytes(
        &self,
        key: &str,
        value: Vec<u8>,
        _ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        // moka uses global TTL configured at build time; per-entry TTL ignored
        self.inner.insert(key.to_string(), value);
        Ok(())
    }

    async fn evict(&self, key: &str) -> Result<(), CacheError> {
        self.inner.invalidate(key);
        Ok(())
    }

    async fn evict_by_prefix(&self, prefix: &str) -> Result<usize, CacheError> {
        let keys_to_remove: Vec<String> = self.inner
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k.as_ref().clone())
            .collect();
        let count = keys_to_remove.len();
        for key in &keys_to_remove {
            self.inner.invalidate(key);
        }
        Ok(count)
    }
}
