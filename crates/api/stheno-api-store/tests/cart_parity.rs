use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::{store_cart_routes, store_payment_routes};
use stheno_cart::CartService;
use stheno_payment::{CredentialMap, PaymentService};
use stheno_payment_offline::OfflinePaymentProviderFactory;
use tower::ServiceExt;

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const PROVIDER: &str = "pp_system_default";
const REGION: &str = "reg_cart_parity";

#[tokio::test]
async fn test_store_cart_and_payment_endpoints_parity() {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let cart_service = Arc::new(CartService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_factory(Arc::new(OfflinePaymentProviderFactory));
    let tenant = stheno_db::tenant::current().expect("test tenant");
    payment_service
        .upsert_provider_credential(&tenant, PROVIDER, &CredentialMap::new())
        .await
        .expect("offline credentials");
    let links = Arc::new(stheno_links::PgLinkStore::new(pool.clone()));
    links
        .link("region_payment_provider", REGION, PROVIDER)
        .await
        .expect("offline enabled");

    let app = axum::Router::new()
        .merge(store_cart_routes(cart_context(
            &pool,
            cart_service.clone(),
            None,
        )))
        .merge(store_payment_routes(
            stheno_api_store::StorePaymentContext {
                payment_service: payment_service.clone(),
                cart_service: cart_service.clone(),
                links: links.clone(),
            },
        ));

    // A priced variant to add. The old payload named a title and a `unit_price` and no
    // variant at all, which passed only because the endpoint took the price from the
    // request body — a shopper could name their own. Prices come from the catalogue now,
    // so the fixture has to exist.
    let product_service = stheno_product::ProductService::new(pool.clone());
    let pricing_service = stheno_pricing::PricingService::new(pool.clone());
    let product = product_service
        .create_product(stheno_product::CreateProductInput {
            title: "Merino Wool Beanie".to_string(),
            handle: Some("merino-wool-beanie".to_string()),
            ..Default::default()
        })
        .await
        .expect("product");
    let variant = product_service
        .create_variant(stheno_product::CreateVariantInput {
            product_id: product.id.to_string(),
            title: "Merino Wool Beanie".to_string(),
            sku: Some("BEANIE".to_string()),
            ..Default::default()
        })
        .await
        .expect("variant");
    pricing_service
        .set_variant_prices(
            &variant.id.to_string(),
            vec![stheno_pricing::CreatePriceInput {
                currency_code: "usd".to_string(),
                amount: rust_decimal_macros::dec!(25.00),
                min_quantity: None,
                max_quantity: None,
                price_list_id: None,
                rules: None,
            }],
        )
        .await
        .expect("price");

    // 1. POST /store/carts
    let create_payload = json!({
        "region_id": "reg_us",
        "currency_code": "usd",
        "email": "test@example.com"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/carts")
                .header("Content-Type", "application/json")
                .body(Body::from(create_payload.to_string()))
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
    assert_eq!(body["cart"]["currency_code"], "usd");

    // 2. GET /store/carts/{id}
    let get_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/carts/{cart_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::OK);

    // 3. POST /store/carts/{id}/line-items
    // No `unit_price`: the server resolves it, and a client-supplied one is ignored.
    let add_item_payload = json!({
        "variant_id": variant.id.to_string(),
        "quantity": 2,
    });

    let item_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{cart_id}/line-items"))
                .header("Content-Type", "application/json")
                .body(Body::from(add_item_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(item_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(item_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let item_body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(item_body["cart"]["items"].as_array().unwrap().len(), 1);
    // The price came from the price set, not from the request.
    //
    // Compared by value, not by JSON token: decimals serialize as strings across this API
    // and the cart does not normalize scale the way the pricing DTO does, so this arrives
    // as "25.000000" while the same price on a product reads "25". Asserting the literal
    // would be asserting the scale, which is a property of the column.
    let unit_price: rust_decimal::Decimal = item_body["cart"]["items"][0]["unit_price"]
        .as_str()
        .expect("unit_price is serialized as a string")
        .parse()
        .expect("decimal");
    assert_eq!(unit_price, rust_decimal_macros::dec!(25));
    let line_id = item_body["cart"]["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 4. POST /store/carts/{id}/line-items/{line_id} (update quantity)
    let update_item_payload = json!({
        "quantity": 3
    });

    let upd_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/store/carts/{cart_id}/line-items/{line_id}"))
                .header("Content-Type", "application/json")
                .body(Body::from(update_item_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(upd_resp.status(), StatusCode::OK);

    // 5. GET /store/payment-providers
    let pp_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/payment-providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(pp_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(pp_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let pp_body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(!pp_body["payment_providers"].as_array().unwrap().is_empty());
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
        store_name: "test store".to_string(),
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
        notification_service: std::sync::Arc::new(stheno_notification::NotificationService::new(
            pool.clone(),
        )),
    }
}
