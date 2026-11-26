use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use stheno_api_admin::admin_order_routes;
use stheno_auth::JwtClaims;
use stheno_db::testing::TestDb;
use stheno_fulfillment::FulfillmentService;
use stheno_order::{CreateOrderInput, OrderService};
use stheno_payment::PaymentService;
use tower::ServiceExt;

#[tokio::test]
async fn staff_reconciles_unknown_social_courier_request_once_over_admin_api() {
    let db = TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("test pool");
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let fulfillment_service = Arc::new(FulfillmentService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool));
    let order = order_service
        .create_order(CreateOrderInput {
            region_id: None,
            customer_id: None,
            sales_channel_id: None,
            email: None,
            currency_code: "bdt".to_string(),
            shipping_address: None,
            billing_address: None,
            items: None,
            shipping_methods: None,
            is_draft_order: Some(false),
            metadata: Some(json!({"source": stheno_flows_order::SOCIAL_SOURCE})),
        })
        .await
        .expect("social order fixture");
    let stable_reference = format!("social-order:{}", order.id);
    let idempotency_key = format!("create_fulfillment:{stable_reference}");
    TestDb::execute(
        &db,
        &format!(
            "INSERT INTO fulfillment.fulfillment_provider_request \
             (id, idempotency_key, provider_id, operation, resource_type, resource_id, request_hash, state) \
             VALUES ('fpreq_01ARZ3NDEKTSV4RRFFQ69G5FAV', '{idempotency_key}', 'pathao', \
                     'create_fulfillment', 'fulfillment', 'ful_route_fixture', 'fixture-hash', 'unknown')"
        ),
    )
    .await;

    let claims = JwtClaims {
        sub: "usr_social_dispatch_staff".to_string(),
        actor_id: Some("usr_social_dispatch_staff".to_string()),
        actor_type: Some("user".to_string()),
        token_version: 0,
        exp: i64::MAX,
        iat: 0,
        jti: "social-dispatch-route-test".to_string(),
    };
    let app = admin_order_routes(order_service, fulfillment_service, payment_service)
        .layer(axum::Extension(claims));
    let path = format!("/admin/orders/{}/courier-dispatch/reconcile", order.id);
    let request = || {
        Request::builder()
            .method("POST")
            .uri(&path)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "outcome": "accepted",
                    "evidence_reference": "courier-ticket-2026-10",
                    "courier_reference": "pathao-consignment-123",
                    "tracking_number": "PATHAO-TRACK-123",
                    "tracking_url": "https://courier.example/track/PATHAO-TRACK-123"
                })
                .to_string(),
            ))
            .expect("valid request")
    };

    let response = app
        .clone()
        .oneshot(request())
        .await
        .expect("router responds");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body reads");
    let response: Value = serde_json::from_slice(&body).expect("valid JSON response");
    assert_eq!(status, StatusCode::OK, "unexpected response: {response}");
    assert_eq!(response["reconciled"], true);
    assert_eq!(response["outcome"], "accepted");

    let repeated = app
        .oneshot(request())
        .await
        .expect("router responds to repeated settlement");
    assert_eq!(repeated.status(), StatusCode::BAD_REQUEST);

    db.drop().await;
}
