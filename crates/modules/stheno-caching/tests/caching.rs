use std::sync::Arc;
use stheno_caching::{CacheService, InMemoryCacheProvider};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug, Clone)]
struct ProductCacheEntry {
    id: String,
    title: String,
    price_cents: i64,
}

#[tokio::test]
async fn test_in_process_cache_and_json_helpers() {
    let provider = Arc::new(InMemoryCacheProvider::new());
    let cache = CacheService::new(provider);

    let key = "product:prod_001";
    let product = ProductCacheEntry {
        id: "prod_001".to_string(),
        title: "Stheno Hoodie".to_string(),
        price_cents: 8900,
    };

    // 1. Set JSON
    cache.set_json(key, &product, Some(3600)).await.unwrap();

    // 2. Get JSON
    let cached: Option<ProductCacheEntry> = cache.get_json(key).await.unwrap();
    assert_eq!(cached, Some(product.clone()));

    // 3. Get or Set JSON
    let fetched = cache
        .get_or_set_json("product:prod_002", Some(3600), || async {
            Ok(ProductCacheEntry {
                id: "prod_002".to_string(),
                title: "Stheno Cap".to_string(),
                price_cents: 2900,
            })
        })
        .await
        .unwrap();

    assert_eq!(fetched.id, "prod_002");

    // 4. Invalidate prefix
    cache.invalidate_prefix("product:").await.unwrap();
    let empty: Option<ProductCacheEntry> = cache.get_json(key).await.unwrap();
    assert!(empty.is_none());
}
