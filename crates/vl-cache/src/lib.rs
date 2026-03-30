pub mod error;
pub mod factory;

#[cfg(test)]
mod tests;
pub mod keys;
pub mod local;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "redis")]
pub mod tiered;

pub use error::CacheError;
pub use factory::{create_cache, create_cache_async};
pub use local::MokaCache;

#[cfg(feature = "redis")]
pub use redis::RedisCache;

#[cfg(feature = "redis")]
pub use tiered::TieredCache;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

// ── Core trait ────────────────────────────────────────────────────────────────

/// Object-safe cache trait — uses raw bytes as values.
///
/// Use the free functions `get_cached` / `put_cached` for typed JSON access.
#[async_trait]
pub trait TbCache: Send + Sync {
    /// Get raw bytes by key. Returns None on cache miss.
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Put raw bytes. TTL overrides the backend default if provided.
    async fn put_bytes(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>;

    /// Remove a single key.
    async fn evict(&self, key: &str) -> Result<(), CacheError>;

    /// Remove all keys with the given prefix. Returns count of removed entries.
    async fn evict_by_prefix(&self, prefix: &str) -> Result<usize, CacheError>;
}

// ── Typed helpers (free functions — generics prevent object-safety) ───────────

/// Deserialize a cached value from JSON bytes.
pub async fn get_cached<V: DeserializeOwned>(
    cache: &dyn TbCache,
    key: &str,
) -> Result<Option<V>, CacheError> {
    match cache.get_bytes(key).await? {
        Some(bytes) => {
            let value = serde_json::from_slice::<V>(&bytes)?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Serialize a value as JSON bytes and store in cache.
pub async fn put_cached<V: Serialize>(
    cache: &dyn TbCache,
    key: &str,
    value: &V,
    ttl: Option<Duration>,
) -> Result<(), CacheError> {
    let bytes = serde_json::to_vec(value)?;
    cache.put_bytes(key, bytes, ttl).await
}
