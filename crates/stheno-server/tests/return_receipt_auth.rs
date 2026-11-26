//! Return-receipt mutations are available only through the staff-authenticated admin surface.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use stheno_platform::{PlatformService, SubscriptionPatch};
use stheno_server::TestApp;
use tower::ServiceExt;

async fn token_for(app: &TestApp, path: &str, email: &str) -> String {
    let response = app
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"Sup3rSecret!"}}"#
                )))
                .expect("registration request"),
        )
        .await
        .expect("registration response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("registration body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("registration JSON");
    value["token"]
        .as_str()
        .expect("registration token")
        .to_string()
}

#[tokio::test]
async fn staged_return_receipt_requires_staff_credentials() {
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
                feature_overrides: Some(serde_json::json!({"buyer_accounts": true})),
                ..Default::default()
            },
        )
        .await
        .expect("enable buyer-account auth for the customer-role assertion");

    let path = "/admin/returns/return_receipt_auth_missing/receive/confirm";
    let body = r#"{"no_notification":true}"#;
    let missing = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("unauthenticated request"),
        )
        .await
        .expect("unauthenticated response");
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);

    let customer_token = token_for(
        &harness,
        "/auth/customer/emailpass/register",
        "buyer@return-receipt-auth.test",
    )
    .await;
    let customer = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {customer_token}"))
                .body(Body::from(body))
                .expect("customer request"),
        )
        .await
        .expect("customer response");
    assert_eq!(customer.status(), StatusCode::FORBIDDEN);

    let staff_token = token_for(
        &harness,
        "/auth/user/emailpass/register",
        "staff@return-receipt-auth.test",
    )
    .await;
    let staff = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {staff_token}"))
                .body(Body::from(body))
                .expect("staff request"),
        )
        .await
        .expect("staff response");
    assert_eq!(staff.status(), StatusCode::BAD_REQUEST);

    harness.teardown().await;
}
