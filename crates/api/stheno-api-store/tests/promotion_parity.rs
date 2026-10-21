use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::store_cart_routes;
use stheno_cart::CartService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_promotion::{CreateApplicationMethodInput, CreatePromotionInput, PromotionService};
use tower::ServiceExt;

#[tokio::test]
async fn test_store_cart_promotion_application_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let cart_service = Arc::new(CartService::new(pool.clone()));
    let promo_service = Arc::new(PromotionService::new(pool.clone()));

    // A priced variant to discount. Line items are priced from the catalogue now, so a
    // request naming only a title and a price is refused — which is the fix, not a
    // regression: it was how a shopper could set their own price.
    let product_service = stheno_product::ProductService::new(pool.clone());
    let pricing_service = stheno_pricing::PricingService::new(pool.clone());
    let product = product_service
        .create_product(stheno_product::CreateProductInput {
            title: "Winter Jacket".to_string(),
            handle: Some("winter-jacket".to_string()),
            ..Default::default()
        })
        .await
        .expect("product");
    let variant = product_service
        .create_variant(stheno_product::CreateVariantInput {
            product_id: product.id.to_string(),
            title: "Winter Jacket".to_string(),
            sku: Some("JACKET".to_string()),
            ..Default::default()
        })
        .await
        .expect("variant");
    pricing_service
        .set_variant_prices(
            &variant.id.to_string(),
            vec![stheno_pricing::CreatePriceInput {
                currency_code: "usd".to_string(),
                amount: rust_decimal_macros::dec!(100.00),
                min_quantity: None,
                max_quantity: None,
                price_list_id: None,
                rules: None,
            }],
        )
        .await
        .expect("price");

    // 1. Create a 20% discount promotion
    promo_service
        .create_promotion(CreatePromotionInput {
            code: "PROMO20".to_string(),
            promotion_type: Some("standard".to_string()),
            is_automatic: Some(false),
            campaign_id: None,
            status: Some("active".to_string()),
            application_method: Some(CreateApplicationMethodInput {
                method_type: "percentage".to_string(),
                target_type: "items".to_string(),
                allocation: Some("across".to_string()),
                value: dec!(20),
                currency_code: None,
                max_quantity: None,
                apply_to_quantity: None,
                buy_rules_min_quantity: None,
                target_rules: None,
                buy_rules: None,
                metadata: None,
            }),
            rules: None,
            metadata: None,
        })
        .await
        .expect("promotion created");

    let app = store_cart_routes(cart_context(
        &pool,
        cart_service.clone(),
        Some(promo_service),
    ));

    // 2. Create Cart: POST /store/carts
    let create_cart_payload = json!({
        "currency_code": "usd",
        "email": "customer@example.com"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/carts")
                .header("Content-Type", "application/json")
                .body(Body::from(create_cart_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let cart_id = body["cart"]["id"].as_str().unwrap().to_string();

    // 3. Add Line Item: POST /store/carts/{id}/line-items ($100 x 2 = $200)
    let add_item_payload = json!({
        "variant_id": variant.id.to_string(),
        "quantity": 2,
    });

    let item_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{}/line-items", cart_id))
                .header("Content-Type", "application/json")
                .body(Body::from(add_item_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(item_resp.status(), StatusCode::OK);

    // 4. Apply Promotion: POST /store/carts/{id}/promotions
    let promo_payload = json!({
        "promo_codes": ["PROMO20"]
    });

    let promo_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{}/promotions", cart_id))
                .header("Content-Type", "application/json")
                .body(Body::from(promo_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(promo_resp.status(), StatusCode::OK);
    let promo_bytes = axum::body::to_bytes(promo_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: stheno_api_store::carts::CartResponse =
        serde_json::from_slice(&promo_bytes).expect("deserialise CartResponse");

    assert_eq!(res.cart.base.items.len(), 1);
    assert_eq!(res.cart.base.items[0].adjustments.len(), 1);
    assert_eq!(
        res.cart.base.items[0].adjustments[0].code.as_deref(),
        Some("PROMO20")
    );
    assert_eq!(res.cart.base.items[0].adjustments[0].amount, dec!(40));

    // Totals verification
    assert_eq!(res.cart.base.totals.discount_total, dec!(40));
    assert_eq!(res.cart.base.totals.total, dec!(160));
}

/// Line items are priced from the catalogue now, so the cart router needs pricing, region
/// and product services alongside the cart. Built here so tests that are not about
/// pricing do not each have to assemble one.
fn cart_context(
    pool: &stheno_db::DbPool,
    cart_service: std::sync::Arc<CartService>,
    promotion_service: Option<std::sync::Arc<stheno_promotion::PromotionService>>,
) -> stheno_api_store::StoreCartContext {
    stheno_api_store::StoreCartContext {
        cart_service: cart_service.clone(),
        promotion_service,
        pricing_service: std::sync::Arc::new(stheno_pricing::PricingService::new(pool.clone())),
        region_service: std::sync::Arc::new(stheno_region::RegionService::new(pool.clone())),
        product_service: std::sync::Arc::new(stheno_product::ProductService::new(pool.clone())),
        fulfillment_service: std::sync::Arc::new(stheno_fulfillment::FulfillmentService::new(
            pool.clone(),
        )),
        links: std::sync::Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        complete_flows: std::sync::Arc::new(stheno_flows::CompleteCartWorkflowEngine::new(
            cart_service.clone(),
            std::sync::Arc::new(stheno_order::OrderService::new(pool.clone())),
            std::sync::Arc::new(stheno_payment::PaymentService::new(pool.clone())),
            std::sync::Arc::new(stheno_promotion::PromotionService::new(pool.clone())),
            std::sync::Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
            std::sync::Arc::new(stheno_workflow::WorkflowStore::from_pool(pool.clone())),
        )),
        payment_service: std::sync::Arc::new(stheno_payment::PaymentService::new(pool.clone())),
    }
}
