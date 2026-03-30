//! Two-level cache: L1 (moka in-process) + L2 (Redis).
//!
//! Read path:  L1 hit → return immediately.
//!             L1 miss → try L2 → on hit populate L1 → return.
//! Write path: write to L2, then invalidate L1.

use std::time::Duration;

use async_trait::async_trait;
use tracing::debug;

use crate::{CacheError, MokaCache, RedisCache, TbCache};

pub struct TieredCache {
    l1: MokaCache,
    l2: RedisCache,
}

impl TieredCache {
    pub fn new(l1: MokaCache, l2: RedisCache) -> Self {
        Self { l1, l2 }
    }
}

#[async_trait]
impl TbCache for TieredCache {
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        // L1 first
        if let Some(v) = self.l1.get_bytes(key).await? {
            return Ok(Some(v));
        }
        // L2 fallback
        match self.l2.get_bytes(key).await? {
            Some(v) => {
                // Populate L1 (fire-and-forget error)
                let _ = self.l1.put_bytes(key, v.clone(), None).await;
                debug!(key, "Cache L2 hit → populated L1");
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    async fn put_bytes(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        // Write to L2 (source of truth), invalidate L1
        self.l2.put_bytes(key, value, ttl).await?;
        let _ = self.l1.evict(key).await;
        Ok(())
    }

    async fn evict(&self, key: &str) -> Result<(), CacheError> {
        self.l2.evict(key).await?;
        let _ = self.l1.evict(key).await;
        Ok(())
    }

    async fn evict_by_prefix(&self, prefix: &str) -> Result<usize, CacheError> {
        let count = self.l2.evict_by_prefix(prefix).await?;
        let _ = self.l1.evict_by_prefix(prefix).await;
        Ok(count)
    }
}
