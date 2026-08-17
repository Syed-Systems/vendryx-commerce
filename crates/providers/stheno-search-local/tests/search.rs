use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_search::{SearchDocument, SearchFilter, SearchProvider, SearchQuery, SearchService};
use stheno_search_local::LocalSearchProvider;

#[tokio::test]
async fn test_search_indexing_filtering_and_scoring() {
    let provider = Arc::new(LocalSearchProvider::new());
    let service = SearchService::new(provider);

    // Index test products
    let p1 = SearchDocument {
        id: "prod_hoodie".to_string(),
        title: "Heavyweight Black Hoodie".to_string(),
        description: Some("Warm fleece organic cotton hoodie".to_string()),
        handle: Some("black-hoodie".to_string()),
        sales_channel_ids: vec!["sc_web".to_string()],
        category_ids: vec!["cat_apparel".to_string()],
        tags: vec!["cotton".to_string(), "hoodie".to_string()],
        price_cents: Some(8500),
        attributes: HashMap::new(),
    };

    let p2 = SearchDocument {
        id: "prod_cap".to_string(),
        title: "Embroidered Logo Cap".to_string(),
        description: Some("Adjustable cotton twill cap".to_string()),
        handle: Some("logo-cap".to_string()),
        sales_channel_ids: vec!["sc_web".to_string()],
        category_ids: vec!["cat_apparel".to_string()],
        tags: vec!["cotton".to_string(), "cap".to_string()],
        price_cents: Some(3000),
        attributes: HashMap::new(),
    };

    let p3 = SearchDocument {
        id: "prod_mug".to_string(),
        title: "Ceramic Coffee Mug".to_string(),
        description: Some("12oz matte black ceramic mug".to_string()),
        handle: Some("ceramic-mug".to_string()),
        sales_channel_ids: vec!["sc_retail".to_string()],
        category_ids: vec!["cat_accessories".to_string()],
        tags: vec!["ceramic".to_string()],
        price_cents: Some(1800),
        attributes: HashMap::new(),
    };

    service
        .index_documents("products", vec![p1, p2, p3])
        .await
        .unwrap();

    // 1. Text search: "hoodie"
    let res = service
        .search(
            "products",
            SearchQuery {
                q: "hoodie".to_string(),
                filter: None,
                limit: 10,
                offset: 0,
                sort_by: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(res.total_hits, 1);
    assert_eq!(res.hits[0].document.id, "prod_hoodie");

    // 2. Filtered search: category "cat_apparel" + price sort
    let apparel_res = service
        .search(
            "products",
            SearchQuery {
                q: "cotton".to_string(),
                filter: Some(SearchFilter {
                    category_ids: Some(vec!["cat_apparel".to_string()]),
                    tags: None,
                    min_price_cents: None,
                    max_price_cents: None,
                    sales_channel_id: Some("sc_web".to_string()),
                }),
                limit: 10,
                offset: 0,
                sort_by: Some("price_asc".to_string()),
            },
        )
        .await
        .unwrap();

    assert_eq!(apparel_res.total_hits, 2);
    // Price asc: Cap ($30.00) first, Hoodie ($85.00) second
    assert_eq!(apparel_res.hits[0].document.id, "prod_cap");
    assert_eq!(apparel_res.hits[1].document.id, "prod_hoodie");
}

#[tokio::test]
async fn test_event_driven_search_sync() {
    let provider = Arc::new(LocalSearchProvider::new());
    let service = SearchService::new(provider);

    // Event: product.created
    service
        .handle_product_event(
            "product.created",
            &serde_json::json!({
                "id": "prod_event_1",
                "title": "Solar Powered Backpack",
                "description": "High tech solar travel bag",
                "handle": "solar-backpack",
                "price_cents": 12000
            }),
        )
        .await
        .unwrap();

    let search_res = service
        .search(
            "products",
            SearchQuery {
                q: "solar".to_string(),
                filter: None,
                limit: 10,
                offset: 0,
                sort_by: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(search_res.total_hits, 1);
    assert_eq!(search_res.hits[0].document.title, "Solar Powered Backpack");

    // Event: product.deleted
    service
        .handle_product_event(
            "product.deleted",
            &serde_json::json!({ "id": "prod_event_1" }),
        )
        .await
        .unwrap();

    let search_after_del = service
        .search(
            "products",
            SearchQuery {
                q: "solar".to_string(),
                filter: None,
                limit: 10,
                offset: 0,
                sort_by: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(search_after_del.total_hits, 0);
}

#[tokio::test]
async fn test_search_convergence_after_bulk_update() {
    // Task P8-11: Search convergence <= 5 s after 1,000-product bulk update
    let provider = Arc::new(LocalSearchProvider::new());
    let service = SearchService::new(provider);

    let start = Instant::now();

    let mut bulk_docs = Vec::with_capacity(1000);
    for i in 0..1000 {
        bulk_docs.push(SearchDocument {
            id: format!("prod_bulk_{:04}", i),
            title: format!("Bulk Product SKU-{:04} Special Edition", i),
            description: Some(format!("High quality variant description for item {}", i)),
            handle: Some(format!("bulk-prod-{}", i)),
            sales_channel_ids: vec!["sc_global".to_string()],
            category_ids: vec!["cat_bulk".to_string()],
            tags: vec!["bulk".to_string(), format!("batch_{}", i % 10)],
            price_cents: Some(1000 + (i as i64 * 10)),
            attributes: HashMap::new(),
        });
    }

    service
        .index_documents("products", bulk_docs)
        .await
        .unwrap();

    // Verify search convergence across the 1,000 documents
    let query_res = service
        .search(
            "products",
            SearchQuery {
                q: "SKU-0500".to_string(),
                filter: None,
                limit: 10,
                offset: 0,
                sort_by: None,
            },
        )
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(query_res.total_hits, 1);
    assert_eq!(query_res.hits[0].document.id, "prod_bulk_0500");
    assert!(
        elapsed.as_secs() < 5,
        "1,000-product indexing and search took {:?}, expected < 5s",
        elapsed
    );
}

#[tokio::test]
async fn test_search_persistence_and_full_rebuild() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let provider = Arc::new(LocalSearchProvider::new());
    let service = SearchService::with_pool(provider.clone(), pool);

    let doc = SearchDocument {
        id: "prod_persisted_1".to_string(),
        title: "Durable Leather Boots".to_string(),
        description: Some("Handcrafted full-grain leather boots".to_string()),
        handle: Some("leather-boots".to_string()),
        sales_channel_ids: vec!["sc_web".to_string()],
        category_ids: vec!["cat_footwear".to_string()],
        tags: vec!["boots".to_string(), "leather".to_string()],
        price_cents: Some(25000),
        attributes: HashMap::new(),
    };

    service
        .index_documents("products", vec![doc])
        .await
        .unwrap();

    // Clear provider index directly to simulate empty/crashed search node
    provider.clear_index("products").await.unwrap();
    let empty_res = service
        .search(
            "products",
            SearchQuery {
                q: "leather".to_string(),
                filter: None,
                limit: 10,
                offset: 0,
                sort_by: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(empty_res.total_hits, 0);

    // Rebuild index from Postgres storage (P14-13)
    let rebuilt_count = service.rebuild_from_storage("products").await.unwrap();
    assert_eq!(rebuilt_count, 1);

    let search_res = service
        .search(
            "products",
            SearchQuery {
                q: "leather".to_string(),
                filter: None,
                limit: 10,
                offset: 0,
                sort_by: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(search_res.total_hits, 1);
    assert_eq!(search_res.hits[0].document.id, "prod_persisted_1");

    db.drop().await;
}
