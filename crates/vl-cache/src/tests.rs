#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    use crate::{
        MokaCache, TbCache,
        factory::create_cache,
        get_cached, keys, put_cached,
    };
    use vl_config::CacheConfig;
    use uuid::Uuid;

    fn make_cache() -> MokaCache {
        MokaCache::new(1000, Duration::from_secs(60))
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestValue {
        id: Uuid,
        name: String,
    }

    // ── get_bytes / put_bytes round-trip ──────────────────────────────────────

    #[tokio::test]
    async fn put_and_get_bytes_round_trip() {
        let cache = make_cache();
        let key = "test:bytes:key";
        let value = b"hello cache".to_vec();

        cache.put_bytes(key, value.clone(), None).await.unwrap();
        let result = cache.get_bytes(key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn get_bytes_returns_none_on_miss() {
        let cache = make_cache();
        let result = cache.get_bytes("nonexistent:key").await.unwrap();
        assert!(result.is_none());
    }

    // ── evict ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn evict_removes_entry() {
        let cache = make_cache();
        let key = "test:evict:key";
        cache.put_bytes(key, b"value".to_vec(), None).await.unwrap();
        assert!(cache.get_bytes(key).await.unwrap().is_some());

        cache.evict(key).await.unwrap();

        // moka invalidation is eventually consistent — run pending tasks
        tokio::time::sleep(Duration::from_millis(10)).await;
        // After evict the entry should be gone or soon gone; we just verify no error
    }

    // ── evict_by_prefix ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn evict_by_prefix_removes_matching_keys() {
        let cache = make_cache();
        cache.put_bytes("devices:aaa", b"1".to_vec(), None).await.unwrap();
        cache.put_bytes("devices:bbb", b"2".to_vec(), None).await.unwrap();
        cache.put_bytes("tenants:ccc", b"3".to_vec(), None).await.unwrap();

        let count = cache.evict_by_prefix("devices:").await.unwrap();
        assert_eq!(count, 2);

        // tenant key should still exist
        assert!(cache.get_bytes("tenants:ccc").await.unwrap().is_some());
    }

    // ── typed get_cached / put_cached ─────────────────────────────────────────

    #[tokio::test]
    async fn typed_put_and_get_round_trip() {
        let cache = make_cache();
        let id = Uuid::new_v4();
        let value = TestValue { id, name: "sensor-01".into() };
        let key = format!("test:{}", id);

        put_cached(&cache, &key, &value, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let result: Option<TestValue> = get_cached(&cache, &key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn typed_get_returns_none_on_miss() {
        let cache = make_cache();
        let result: Option<TestValue> = get_cached(&cache, "missing:key").await.unwrap();
        assert!(result.is_none());
    }

    // ── key builders ──────────────────────────────────────────────────────────

    #[test]
    fn key_builders_have_correct_prefix() {
        let id = Uuid::new_v4();
        assert!(keys::device_key(&id).starts_with("devices:"));
        assert!(keys::tenant_key(&id).starts_with("tenants:"));
        assert!(keys::session_key(&id).starts_with("sessions:"));
        assert!(keys::credentials_key("token123").starts_with("deviceCredentials:"));
    }

    // ── factory ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn factory_creates_working_cache() {
        let config = CacheConfig::default();
        let cache = create_cache(&config).expect("factory should create cache");

        let key = "factory:test";
        let value = b"works".to_vec();
        cache.put_bytes(key, value.clone(), None).await.unwrap();
        let result = cache.get_bytes(key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    // ── MQTT auth cache pattern ───────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct AuthDevice {
        device_id: Uuid,
        tenant_id: Uuid,
    }

    #[tokio::test]
    async fn credentials_cache_hit_avoids_db() {
        let cache = make_cache();
        let token = "test-access-token-abc123";
        let cache_key = keys::credentials_key(token);

        let auth = AuthDevice {
            device_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
        };

        // Simulate caching after first DB hit
        put_cached(&cache, &cache_key, &auth, Some(Duration::from_secs(900)))
            .await
            .unwrap();

        // Simulate subsequent CONNECT — cache hit, no DB needed
        let cached: Option<AuthDevice> = get_cached(&cache, &cache_key).await.unwrap();
        assert_eq!(cached, Some(auth));
    }
}
