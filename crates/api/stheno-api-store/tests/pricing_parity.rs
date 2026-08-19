use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use stheno_api_store::{store_pricing_routes, StoreApiContext};
use stheno_currency::CurrencyService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_pricing::PricingService;
use stheno_product::{CreateProductInput, ProductService};
use stheno_region::{CreateRegionInput, RegionService};
use tower::ServiceExt;

#[tokio::test]
async fn test_store_pricing_endpoints_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let product_service = Arc::new(ProductService::new(pool.clone()));
    let region_service = Arc::new(RegionService::new(pool.clone()));
    let currency_service = Arc::new(CurrencyService::new(pool.clone()));
    let pricing_service = Arc::new(PricingService::new(pool.clone()));

    // 1. Seed Product
    let _prod = product_service
        .create_product(CreateProductInput {
            title: "Medusa Parity Silk Scarf".to_string(),
            handle: Some("medusa-parity-silk-scarf".to_string()),
            subtitle: Some("100% Mulberry Silk".to_string()),
            description: Some("Luxury scarf".to_string()),
            discountable: Some(true),
        })
        .await
        .unwrap();

    // 2. Seed Region
    let reg = region_service
        .create_region(CreateRegionInput {
            name: "European Union".to_string(),
            currency_code: "eur".to_string(),
            automatic_taxes: Some(true),
            countries: vec!["de".to_string(), "fr".to_string(), "it".to_string()],
        })
        .await
        .unwrap();

    let reg_id = reg.id.to_string();

    let cart_service = Arc::new(stheno_cart::CartService::new(pool.clone()));
    let payment_service = Arc::new(stheno_payment::PaymentService::new(pool.clone()));
    let order_service = Arc::new(stheno_order::OrderService::new(pool.clone()));
    let order_flows = Arc::new(stheno_flows::OrderWorkflowEngine::new(
        order_service.clone(),
    ));
    let return_flows = Arc::new(stheno_flows::ReturnWorkflowEngine::new(
        order_service.clone(),
        None,
    ));
    let fulfillment_service = Arc::new(stheno_fulfillment::FulfillmentService::new(pool.clone()));
    let customer_service = Arc::new(stheno_customer::CustomerService::new(pool.clone()));
    let auth_service = Arc::new(stheno_auth::AuthService::new(pool.clone(), None));
    let translation_service = Arc::new(stheno_translation::TranslationService::new(pool.clone()));

    let ctx = StoreApiContext {
        // Social checkout is not what this suite exercises, but the context needs them.
        social_flows: std::sync::Arc::new(stheno_flows_order::SocialDraftWorkflow::new(
            order_service.clone(),
        )),
        store_service: std::sync::Arc::new(stheno_store::StoreService::new(pool.clone())),
        store_name: "Test Store".to_string(),
        inventory_service: std::sync::Arc::new(stheno_inventory::InventoryService::new(
            pool.clone(),
        )),
        links: std::sync::Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        complete_flows: std::sync::Arc::new(stheno_flows::CompleteCartWorkflowEngine::new(
            cart_service.clone(),
            order_service.clone(),
            payment_service.clone(),
            std::sync::Arc::new(stheno_promotion::PromotionService::new(pool.clone())),
            std::sync::Arc::new(stheno_links::LinkStore::new()),
            std::sync::Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        )),
        product_service,
        region_service,
        currency_service,
        pricing_service,
        cart_service,
        payment_service,
        order_service,
        order_flows,
        return_flows,
        fulfillment_service,
        customer_service,
        auth_service,
        translation_service,
        promotion_service: None,
    };

    let app = store_pricing_routes(ctx);

    // ── Test 1: GET /store/regions ─────────────────────────────────────────
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/regions")
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
    assert_eq!(body["regions"][0]["name"], "European Union");
    assert_eq!(body["regions"][0]["currency_code"], "eur");
    assert_eq!(body["regions"][0]["countries"].as_array().unwrap().len(), 3);

    // ── Test 2: GET /store/regions/{id} ────────────────────────────────────
    let reg_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/regions/{}", reg_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reg_response.status(), StatusCode::OK);
    let reg_bytes = axum::body::to_bytes(reg_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let reg_body: Value = serde_json::from_slice(&reg_bytes).unwrap();
    assert_eq!(reg_body["region"]["name"], "European Union");

    // ── Test 3: GET /store/currencies ──────────────────────────────────────
    let currs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/currencies")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(currs_response.status(), StatusCode::OK);
    let currs_bytes = axum::body::to_bytes(currs_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let currs_body: Value = serde_json::from_slice(&currs_bytes).unwrap();
    assert!(currs_body["count"].as_u64().unwrap() >= 20);

    // ── Test 4: GET /store/currencies/eur ──────────────────────────────────
    let eur_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/currencies/eur")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(eur_response.status(), StatusCode::OK);
    let eur_bytes = axum::body::to_bytes(eur_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let eur_body: Value = serde_json::from_slice(&eur_bytes).unwrap();
    assert_eq!(eur_body["currency"]["code"], "eur");
    assert_eq!(eur_body["currency"]["symbol"], "€");
    assert_eq!(eur_body["currency"]["decimal_digits"], 2);

    // ── Test 5: GET /store/products ────────────────────────────────────────
    let prods_response = app
        .oneshot(
            Request::builder()
                .uri("/store/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(prods_response.status(), StatusCode::OK);
    let prods_bytes = axum::body::to_bytes(prods_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let prods_body: Value = serde_json::from_slice(&prods_bytes).unwrap();
    assert_eq!(prods_body["count"], 1);
    assert_eq!(
        prods_body["products"][0]["handle"],
        "medusa-parity-silk-scarf"
    );

    db.drop().await;
}
