use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::{store_order_routes, store_shipping_option_routes};
use stheno_auth::{AuthService, AuthenticateInput, CreateIdentityInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderWorkflowEngine;
use stheno_fulfillment::{
    CreateFulfillmentSetInput, CreateServiceZoneInput, CreateShippingOptionInput,
    CreateShippingProfileInput, FulfillmentService,
};
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};
use tower::ServiceExt;

#[tokio::test]
async fn test_store_order_and_shipping_option_endpoints_parity() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let order_flows = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
    let fulfillment_service = Arc::new(FulfillmentService::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));

    let app = axum::Router::new()
        // Orders expand their payment collections now, so the router needs the link store
        // and payment service alongside the order service.
        .merge(store_order_routes(stheno_api_store::StoreOrderContext {
            order_service: order_service.clone(),
            order_flows: order_flows.clone(),
            links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
            payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
            notification_service: Arc::new(stheno_notification::NotificationService::new(
                pool.clone(),
            )),
            auth_service: auth_service.clone(),
        }))
        .merge(store_shipping_option_routes(fulfillment_service.clone()));

    // Seed Order
    let order = order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_store_alice".to_string()),
            sales_channel_id: None,
            email: Some("alice@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Leather Handbag".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(220.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: None,
                product_id: None,
                product_title: None,
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: None,
                variant_sku: None,
                variant_barcode: None,
                variant_title: None,
                variant_option_values: None,
                metadata: None,
            }]),
            shipping_methods: None,
            is_draft_order: None,
            metadata: None,
        })
        .await
        .expect("order created");

    // A shopper's own token. `/store/orders` is per-customer, so the listing needs an
    // identity bound to the order's customer to return anything at all.
    auth_service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "alice@example.com".to_string(),
            password: Some("Sup3rSecret!".to_string()),
            actor_id: Some("cus_store_alice".to_string()),
            actor_type: Some("customer".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .expect("identity created");
    let (_, tokens) = auth_service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "alice@example.com".to_string(),
            password: Some("Sup3rSecret!".to_string()),
        })
        .await
        .expect("authenticated");
    let bearer = format!("Bearer {}", tokens.access_token);

    // 0. The listing is not public. It used to answer with every order in the tenant to
    // anyone holding the publishable key, which is shipped in the storefront bundle.
    let anon_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        anon_resp.status(),
        StatusCode::UNAUTHORIZED,
        "an unauthenticated caller must not be able to enumerate orders"
    );

    // 1. GET /store/orders
    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/orders")
                .header("authorization", &bearer)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(list_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(body["orders"][0]["email"], "alice@example.com");

    // 2. GET /store/orders/{id}
    let get_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/orders/{}", order.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(get_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["order"]["id"], order.id);
    assert_eq!(body["order"]["status"], "pending");

    // 3. POST /store/orders/transfer
    let transfer_payload = json!({
        "order_id": order.id,
        "target_customer_id": "cus_store_bob"
    });

    let trans_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/orders/transfer")
                .header("Content-Type", "application/json")
                .body(Body::from(transfer_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(trans_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(trans_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "pending");

    // 4. POST /store/orders/transfer/accept
    let accept_payload = json!({
        "order_id": order.id
    });

    let accept_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/orders/transfer/accept")
                .header("Content-Type", "application/json")
                .body(Body::from(accept_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(accept_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(accept_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["order"]["customer_id"], "cus_store_bob");

    // 5. Setup and test GET /store/shipping-options
    let set = fulfillment_service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "Default Store Set".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .unwrap();

    let profile = fulfillment_service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Default Profile".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .unwrap();

    let zone = fulfillment_service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id,
            name: "US Continental".to_string(),
            metadata: None,
        })
        .await
        .unwrap();

    let _so = fulfillment_service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Standard Ground Delivery".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone.id,
            shipping_profile_id: profile.id,
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: None,
            metadata: None,
        })
        .await
        .unwrap();

    let so_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/store/shipping-options")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(so_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(so_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["count"], 1);
    assert_eq!(
        body["shipping_options"][0]["name"],
        "Standard Ground Delivery"
    );

    db.drop().await;
}
