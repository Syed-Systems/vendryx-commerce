use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use stheno_api_store::store_catalog_routes;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{CreateCategoryInput, CreateProductInput, ProductService};
use tower::ServiceExt;

#[tokio::test]
async fn test_store_catalog_endpoints_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(ProductService::new(pool.clone()));

    // Seed test product
    let _prod = service
        .create_product(CreateProductInput {
            title: "Parity Classic Hoodie".to_string(),
            handle: Some("parity-classic-hoodie".to_string()),
            subtitle: Some("Organic Cotton".to_string()),
            description: Some("Warm and comfortable".to_string()),
            discountable: Some(true),
        })
        .await
        .unwrap();

    // Seed test category
    let _cat = service
        .create_category(CreateCategoryInput {
            name: "Sweatshirts".to_string(),
            handle: Some("sweatshirts".to_string()),
            parent_category_id: None,
            rank: Some(0),
        })
        .await
        .unwrap();

    let app = store_catalog_routes(catalog_context(&pool, service));

    // 1. GET /store/products
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["products"][0]["handle"], "parity-classic-hoodie");

    // 2. GET /store/product-categories
    let cat_response = app
        .oneshot(
            Request::builder()
                .uri("/store/product-categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cat_response.status(), StatusCode::OK);
    let cat_bytes = axum::body::to_bytes(cat_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let cat_body: Value = serde_json::from_slice(&cat_bytes).unwrap();
    assert_eq!(cat_body["count"], 1);
    assert_eq!(cat_body["product_categories"][0]["handle"], "sweatshirts");

    db.drop().await;
}

/// Products carry prices and stock now, so the catalog router needs those services too.
/// Built here rather than threaded through every test, none of which is about pricing.
fn catalog_context(
    pool: &stheno_db::DbPool,
    product_service: Arc<ProductService>,
) -> stheno_api_store::products::CatalogContext {
    stheno_api_store::products::CatalogContext {
        product_service,
        pricing_service: Arc::new(stheno_pricing::PricingService::new(pool.clone())),
        region_service: Arc::new(stheno_region::RegionService::new(pool.clone())),
        inventory_service: Arc::new(stheno_inventory::InventoryService::new(pool.clone())),
    }
}
