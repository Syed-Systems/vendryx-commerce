//! Quota-bearing admin writes must be reachable by staff only.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use stheno_platform::{PlatformService, SubscriptionPatch};
use stheno_server::TestApp;
use tower::ServiceExt;

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body reads");
    let json = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, json)
}

async fn register_token(app: &axum::Router, path: &str, email: &str) -> String {
    let (status, body) = call(
        app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "email": email, "password": "Sup3rSecret!" }).to_string(),
            ))
            .expect("registration request"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "registration failed: {body}");
    body["token"]
        .as_str()
        .expect("registration issues a token")
        .to_string()
}

#[tokio::test]
async fn quota_bearing_product_write_rejects_customers_and_admits_staff() {
    let previous_tenant = std::env::var("STHENO_TENANT").ok();
    std::env::set_var("STHENO_TENANT", stheno_db::testing::TestDb::TEST_TENANT);
    let harness = TestApp::spawn().await;
    match previous_tenant {
        Some(tenant) => std::env::set_var("STHENO_TENANT", tenant),
        None => std::env::remove_var("STHENO_TENANT"),
    }

    let platform = PlatformService::new(
        stheno_db::create_pool(&harness.database_url, 5).expect("test database pool"),
    );
    platform
        .update_subscription(
            stheno_db::testing::TestDb::TEST_TENANT,
            SubscriptionPatch {
                feature_overrides: Some(json!({ "buyer_accounts": true })),
                ..Default::default()
            },
        )
        .await
        .expect("enable customer authentication for role coverage");

    let path = "/admin/products";
    let payload = json!({ "title": "Staff-only quota product" }).to_string();
    let (anonymous_status, _) = call(
        &harness.app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(payload.clone()))
            .expect("anonymous product request"),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);

    let customer_token = register_token(
        &harness.app,
        "/auth/customer/emailpass/register",
        "quota-buyer@example.test",
    )
    .await;
    let (customer_status, _) = call(
        &harness.app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {customer_token}"))
            .body(Body::from(payload.clone()))
            .expect("customer product request"),
    )
    .await;
    assert_eq!(customer_status, StatusCode::FORBIDDEN);

    let staff_token = register_token(
        &harness.app,
        "/auth/user/emailpass/register",
        "quota-staff@example.test",
    )
    .await;
    let (staff_status, staff_body) = call(
        &harness.app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {staff_token}"))
            .body(Body::from(payload))
            .expect("staff product request"),
    )
    .await;
    assert_eq!(
        staff_status,
        StatusCode::CREATED,
        "staff write failed: {staff_body}"
    );

    let (list_status, products) = call(
        &harness.app,
        Request::builder()
            .uri(path)
            .header("authorization", format!("Bearer {staff_token}"))
            .body(Body::empty())
            .expect("product list request"),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(products["count"], 1, "rejected requests create no product");

    harness.teardown().await;
}
