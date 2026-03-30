use std::sync::Arc;
use std::time::Duration;

use vl_config::{CacheConfig, CacheType};

use crate::{CacheError, MokaCache, TbCache};

/// Tạo cache backend từ config.
///
/// - `CacheType::InMemory` → MokaCache (default, không cần Redis)
/// - `CacheType::Redis`    → TieredCache (L1 moka + L2 Redis, requires feature "redis")
///                           Falls back to MokaCache nếu feature "redis" không được enable.
pub fn create_cache(config: &CacheConfig) -> Result<Arc<dyn TbCache>, CacheError> {
    match config.cache_type {
        CacheType::InMemory => {
            let cache = MokaCache::new(
                config.local.max_size,
                Duration::from_secs(config.local.ttl_seconds),
            );
            Ok(Arc::new(cache))
        }
        CacheType::Redis => {
            #[cfg(feature = "redis")]
            {
                // TieredCache requires async init — use block_in_place or return Err
                // Caller should use create_cache_async for Redis backend
                Err(CacheError::Connection(
                    "Use create_cache_async() for Redis backend".into(),
                ))
            }
            #[cfg(not(feature = "redis"))]
            {
                tracing::warn!(
                    "Cache type 'redis' requested but 'redis' feature not enabled — \
                     falling back to in-memory cache"
                );
                let cache = MokaCache::new(
                    config.local.max_size,
                    Duration::from_secs(config.local.ttl_seconds),
                );
                Ok(Arc::new(cache))
            }
        }
    }
}

/// Async factory — dùng cho Redis backend (cần async để connect).
pub async fn create_cache_async(config: &CacheConfig) -> Result<Arc<dyn TbCache>, CacheError> {
    match config.cache_type {
        CacheType::InMemory => create_cache(config),
        CacheType::Redis => {
            #[cfg(feature = "redis")]
            {
                use crate::{RedisCache, TieredCache};

                let l1 = MokaCache::new(
                    config.local.max_size,
                    Duration::from_secs(config.local.ttl_seconds),
                );
                let l2 = RedisCache::new(
                    &config.redis.url,
                    Duration::from_secs(config.redis.ttl_seconds),
                )
                .await?;
                Ok(Arc::new(TieredCache::new(l1, l2)))
            }
            #[cfg(not(feature = "redis"))]
            {
                tracing::warn!(
                    "Cache type 'redis' requested but 'redis' feature not enabled — \
                     falling back to in-memory cache"
                );
                create_cache(config)
            }
        }
    }
}
