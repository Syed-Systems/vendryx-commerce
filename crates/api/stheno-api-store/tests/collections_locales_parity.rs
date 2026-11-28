//! Parity coverage for the store endpoints the forked Medusa storefront requires:
//! collections, product-variants, customers/me, and locales.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use stheno_api_store::{store_catalog_routes, store_customer_routes, store_locale_routes};
use stheno_auth::AuthService;
use stheno_customer::CustomerService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_product::{
    CreateCollectionInput, CreateProductInput, CreateVariantInput, ProductService,
};
use stheno_translation::{CreateLocaleInput, TranslationService};
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_store_collection_endpoints_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(ProductService::new(pool.clone()));

    let featured = service
        .create_collection(CreateCollectionInput {
            title: "Featured".to_string(),
            handle: Some("featured".to_string()),
            metadata: None,
        })
        .await
        .unwrap();
    service
        .create_collection(CreateCollectionInput {
            title: "Latest Drops".to_string(),
            handle: Some("latest-drops".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    let app = store_catalog_routes(catalog_context(&pool, service));

    // GET /store/collections uses the Medusa { collections, count, offset, limit } envelope.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/collections")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["count"], 2);
    assert!(json["collections"].is_array());
    assert!(json["offset"].is_number());
    assert!(json["limit"].is_number());

    // The handle filter is what getCollectionByHandle in the storefront relies on.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/collections?handle=featured")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;
    assert_eq!(json["count"], 1);
    assert_eq!(json["collections"][0]["handle"], "featured");

    // GET /store/collections/{id} - the id must round-trip from the list response.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/collections/{}", featured.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["collection"]["id"], featured.id);
    assert_eq!(json["collection"]["title"], "Featured");

    // Unknown id is a Medusa-shaped 404, not a panic.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/store/collections/pcol_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = body_json(response).await;
    assert_eq!(json["type"], "not_found");

    db.drop().await;
}

#[tokio::test]
async fn test_store_product_variant_endpoint_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(ProductService::new(pool.clone()));

    let product = service
        .create_product(CreateProductInput {
            title: "Variant Test Tee".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    // Real option values, because a variant's option ids are stored as rows now rather
    // than only folded into a hash. Invented ids like "optval_m" are rejected by the
    // foreign key — which is the point: a variant referring to an option value that does
    // not exist is one no shopper can ever select.
    let size = service
        .create_option(&product.id.to_string(), "Size")
        .await
        .unwrap();
    let colour = service
        .create_option(&product.id.to_string(), "Colour")
        .await
        .unwrap();
    let size_m = service.create_option_value(&size, "M").await.unwrap();
    let colour_black = service.create_option_value(&colour, "Black").await.unwrap();

    let variant = service
        .create_variant(CreateVariantInput {
            product_id: product.id.to_string(),
            title: "Medium / Black".to_string(),
            sku: Some("TEE-M-BLK".to_string()),
            barcode: None,
            option_value_ids: vec![size_m, colour_black],
        })
        .await
        .unwrap();

    let app = store_catalog_routes(catalog_context(&pool, service));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/product-variants/{}", variant.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["variant"]["title"], "Medium / Black");
    assert_eq!(json["variant"]["sku"], "TEE-M-BLK");
    // The option-values hash is recovered from the uniqueness index, not stored on the model.
    assert!(
        json["variant"]["option_values_hash"]
            .as_str()
            .is_some_and(|h| !h.is_empty()),
        "option_values_hash must survive the round-trip"
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/store/product-variants/variant_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    db.drop().await;
}

#[tokio::test]
async fn test_store_customers_me_requires_bearer_token() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let app = store_customer_routes(
        Arc::new(CustomerService::new(pool.clone())),
        Arc::new(AuthService::new(pool.clone(), None)),
        Arc::new(stheno_notification::NotificationService::new(pool.clone())),
    );

    // No token is a 401, never a 404 - the storefront reads 404 as "customer deleted".
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/customers/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["type"], "unauthorized");

    // A garbage token is also a 401, not a 500.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/store/customers/me")
                .header("Authorization", "Bearer not-a-real-jwt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_store_locales_endpoint_lists_enabled_only() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(TranslationService::new(pool));

    service
        .create_locale(CreateLocaleInput {
            code: "en".to_string(),
            name: "English".to_string(),
            is_default: Some(true),
            is_enabled: Some(true),
        })
        .await
        .unwrap();
    service
        .create_locale(CreateLocaleInput {
            code: "xx".to_string(),
            name: "Disabled".to_string(),
            is_default: Some(false),
            is_enabled: Some(false),
        })
        .await
        .unwrap();

    let app = store_locale_routes(service);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/store/locales")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let locales = json["locales"].as_array().unwrap();
    assert_eq!(locales.len(), 1, "disabled locales must not be exposed");
    assert_eq!(locales[0]["code"], "en");
}

/// Products carry prices and stock now, so the catalog router needs those services too.
/// Built here rather than threaded through every test, none of which is about pricing.
fn catalog_context(
    pool: &stheno_db::DbPool,
    product_service: Arc<ProductService>,
) -> stheno_api_store::products::CatalogContext {
    stheno_api_store::products::CatalogContext::new(
        product_service,
        Arc::new(stheno_pricing::PricingService::new(pool.clone())),
        Arc::new(stheno_region::RegionService::new(pool.clone())),
        Arc::new(stheno_inventory::InventoryService::new(pool.clone())),
    )
}
