use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_index::{IndexService, QueryIndexFilterInput, UpsertIndexDataInput};

#[tokio::test]
async fn test_index_models_sync_and_rebuild() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = IndexService::new(pool);

    // 1. Incremental upserts (Task P8-12, P8-13)
    let p1 = service
        .upsert_index_data(UpsertIndexDataInput {
            entity_name: "product".to_string(),
            entity_id: "prod_001".to_string(),
            data: serde_json::json!({
                "title": "Minimalist Sneakers",
                "brand": "Stheno",
                "price": 140.0,
                "sales_channel_id": "sc_online",
                "status": "published"
            }),
            version: Some(1),
        })
        .await
        .unwrap();

    assert_eq!(p1.entity_id, "prod_001");
    assert_eq!(p1.version, 1);

    // Check sync state & lag metric
    let sync_state = service.get_sync_state("product").await.unwrap();
    assert_eq!(sync_state.last_synced_version, 1);
    assert_eq!(sync_state.lag_seconds, 0);

    // 2. Define field & relation
    let _field = service
        .define_field("product", "brand", "string", true, true, true)
        .await
        .unwrap();

    let _rel = service
        .add_relation(
            "product",
            "prod_001",
            "product_variant",
            "var_001",
            "has_many",
        )
        .await
        .unwrap();

    // 3. Full Rebuild
    let rebuild_docs = vec![
        (
            "prod_001".to_string(),
            serde_json::json!({ "title": "Sneakers", "brand": "Stheno", "price": 140.0, "status": "published" }),
        ),
        (
            "prod_002".to_string(),
            serde_json::json!({ "title": "Boots", "brand": "Stheno", "price": 220.0, "status": "published" }),
        ),
        (
            "prod_003".to_string(),
            serde_json::json!({ "title": "Sandals", "brand": "Other", "price": 60.0, "status": "draft" }),
        ),
    ];

    let rebuilt_state = service
        .rebuild_index("product", rebuild_docs)
        .await
        .unwrap();
    assert_eq!(rebuilt_state.status, "ready");

    db.drop().await;
}

#[tokio::test]
async fn test_cross_module_filtering_and_sorting() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = IndexService::new(pool);

    let products = vec![
        (
            "p1".to_string(),
            serde_json::json!({ "title": "Alpha T-Shirt", "category": "apparel", "price": 35.0, "sales_channel": "sc_main" }),
        ),
        (
            "p2".to_string(),
            serde_json::json!({ "title": "Beta Jacket", "category": "apparel", "price": 180.0, "sales_channel": "sc_main" }),
        ),
        (
            "p3".to_string(),
            serde_json::json!({ "title": "Gamma Mug", "category": "home", "price": 15.0, "sales_channel": "sc_retail" }),
        ),
    ];

    service.rebuild_index("product", products).await.unwrap();

    // Filter by category: "apparel" and sort by price descending
    let mut filters = HashMap::new();
    filters.insert("category".to_string(), serde_json::json!("apparel"));

    let results = service
        .query_index(QueryIndexFilterInput {
            entity_name: "product".to_string(),
            filters,
            sort_by: Some("price".to_string()),
            sort_order: Some("desc".to_string()),
            limit: Some(10),
            offset: Some(0),
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].entity_id, "p2"); // Beta Jacket ($180.0)
    assert_eq!(results[1].entity_id, "p1"); // Alpha T-Shirt ($35.0)

    db.drop().await;
}
