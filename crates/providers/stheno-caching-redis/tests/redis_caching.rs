use std::sync::Arc;
use stheno_caching::CacheService;
use stheno_caching_redis::{RedisCacheProvider, RedisConfig};

#[tokio::test]
async fn test_redis_cache_provider_and_outage_resilience() {
    let config = RedisConfig {
        url: "redis://127.0.0.1:6379".to_string(),
        key_prefix: Some("stheno".to_string()),
        default_ttl_seconds: Some(1800),
    };

    let redis_provider = Arc::new(RedisCacheProvider::new(config));
    let service = CacheService::new(redis_provider.clone());

    // 1. Normal operations
    service
        .set_json(
            "cart:cart_101",
            &serde_json::json!({ "id": "cart_101", "total": "$100" }),
            None,
        )
        .await
        .unwrap();

    let cached: Option<serde_json::Value> = service.get_json("cart:cart_101").await.unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap()["id"], "cart_101");

    // 2. SIMULATE REDIS OUTAGE (Task P8-08)
    redis_provider.simulate_outage(true);

    // Read during outage: MUST NOT FAIL! Reads from in-process fallback cache
    let fallback_read: Option<serde_json::Value> = service.get_json("cart:cart_101").await.unwrap();
    assert!(fallback_read.is_some());
    assert_eq!(fallback_read.unwrap()["id"], "cart_101");

    // Write during outage: MUST NOT FAIL! Writes to in-process fallback cache
    service
        .set_json(
            "cart:cart_102",
            &serde_json::json!({ "id": "cart_102", "total": "$250" }),
            None,
        )
        .await
        .unwrap();

    let read_during_outage: Option<serde_json::Value> =
        service.get_json("cart:cart_102").await.unwrap();
    assert!(read_during_outage.is_some());
    assert_eq!(read_during_outage.unwrap()["id"], "cart_102");

    // 3. Redis recovers
    redis_provider.simulate_outage(false);
    let recovered_read: Option<serde_json::Value> =
        service.get_json("cart:cart_101").await.unwrap();
    assert!(recovered_read.is_some());
}
