//! Ownership checks on the single-order store routes.
//!
//! `list_orders` was hardened against unauthenticated order enumeration, and its own doc
//! comment records why. `get_order` and the eight transfer routes were left taking a bare
//! `Path(id)` with no token at all, so anyone who learned an order id could read that order
//! or move it onto their own customer account.
//!
//! These pin the fix. The important assertion is the 404: an order that exists but belongs
//! to someone else must be indistinguishable from one that does not exist, because 403 would
//! confirm it is real (AGENTS.md 2.10).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_api_store::{store_order_routes, StoreOrderContext};
use stheno_auth::{AuthenticateInput, AuthService, CreateIdentityInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderWorkflowEngine;
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};
use tower::ServiceExt;

const ALICE: &str = "cus_authz_alice";
const BOB: &str = "cus_authz_bob";

async fn make_order(order_service: &OrderService, customer_id: &str) -> String {
    order_service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some(customer_id.to_string()),
            sales_channel_id: None,
            email: Some(format!("{customer_id}@example.com")),
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
        .expect("order created")
        .id
        .to_string()
}

async fn bearer_for(auth: &AuthService, customer_id: &str) -> String {
    let email = format!("{customer_id}@example.com");
    auth.create_identity(CreateIdentityInput {
        provider_id: "emailpass".to_string(),
        entity_id: email.clone(),
        password: Some("Sup3rSecret!".to_string()),
        actor_id: Some(customer_id.to_string()),
        actor_type: Some("customer".to_string()),
        provider_metadata: None,
        app_metadata: None,
        user_metadata: None,
    })
    .await
    .expect("identity created");

    let (_, tokens) = auth
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: email,
            password: Some("Sup3rSecret!".to_string()),
        })
        .await
        .expect("authenticated");
    format!("Bearer {}", tokens.access_token)
}

#[tokio::test]
async fn get_order_rejects_anonymous_callers() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let order_flows = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let app = store_order_routes(StoreOrderContext {
        order_service: order_service.clone(),
        order_flows: order_flows.clone(),
        links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
        auth_service: auth_service.clone(),
        notification_service: Arc::new(stheno_notification::NotificationService::new(pool.clone())),
    });

    let order_id = make_order(&order_service, ALICE).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/store/orders/{order_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "an unauthenticated caller must not read an order"
    );
    db.drop().await;
}

#[tokio::test]
async fn get_order_returns_404_for_another_customers_order() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let order_flows = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let app = store_order_routes(StoreOrderContext {
        order_service: order_service.clone(),
        order_flows: order_flows.clone(),
        links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
        auth_service: auth_service.clone(),
        notification_service: Arc::new(stheno_notification::NotificationService::new(pool.clone())),
    });

    let alice_order = make_order(&order_service, ALICE).await;
    let bob = bearer_for(&auth_service, BOB).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/store/orders/{alice_order}"))
                .header("authorization", &bob)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 404, not 403. A 403 would confirm the order exists, which is the existence oracle.
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "another customer's order must be indistinguishable from a missing one"
    );
    db.drop().await;
}

#[tokio::test]
async fn owner_can_read_their_own_order() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let order_flows = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let app = store_order_routes(StoreOrderContext {
        order_service: order_service.clone(),
        order_flows: order_flows.clone(),
        links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
        auth_service: auth_service.clone(),
        notification_service: Arc::new(stheno_notification::NotificationService::new(pool.clone())),
    });

    let alice_order = make_order(&order_service, ALICE).await;
    let alice = bearer_for(&auth_service, ALICE).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/store/orders/{alice_order}"))
                .header("authorization", &alice)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "the owner must still be able to read their own order"
    );
    db.drop().await;
}

#[tokio::test]
async fn transfer_rejects_a_caller_who_does_not_own_the_order() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let order_flows = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let app = store_order_routes(StoreOrderContext {
        order_service: order_service.clone(),
        order_flows: order_flows.clone(),
        links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
        auth_service: auth_service.clone(),
        notification_service: Arc::new(stheno_notification::NotificationService::new(pool.clone())),
    });

    let alice_order = make_order(&order_service, ALICE).await;
    let bob = bearer_for(&auth_service, BOB).await;

    // Bob tries to move Alice's order onto his own account. Unauthenticated, this was
    // accepted and, chained with the get_order hole, was a full order takeover.
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/orders/transfer")
                .header("authorization", &bob)
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"order_id":"{alice_order}","target_customer_id":"{BOB}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "a non-owner must not be able to transfer someone else's order"
    );
    db.drop().await;
}

#[tokio::test]
async fn transfer_rejects_anonymous_callers() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let order_flows = Arc::new(OrderWorkflowEngine::new(order_service.clone()));
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let app = store_order_routes(StoreOrderContext {
        order_service: order_service.clone(),
        order_flows: order_flows.clone(),
        links: Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
        auth_service: auth_service.clone(),
        notification_service: Arc::new(stheno_notification::NotificationService::new(pool.clone())),
    });

    let alice_order = make_order(&order_service, ALICE).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/store/orders/transfer")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"order_id":"{alice_order}","target_customer_id":"{BOB}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    db.drop().await;
}