//! P31-03: Suspension is visible to the merchant with a reason and a way to pay, not a blank 404.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use stheno_server::TestApp;
use tower::ServiceExt;

const TOKEN: &str = "suspension-notice-token";
const HOST: &str = "notice-shop.localhost";
const TENANT_ID: &str = "ten_notice_store";

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

async fn operator(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {TOKEN}"));
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    call(app, builder.body(body).unwrap()).await
}

async fn staff_token(app: &axum::Router, host: &str, email: &str) -> String {
    let (status, body) = call(
        app,
        Request::builder()
            .method("POST")
            .uri("/auth/user/emailpass/register")
            .header("host", host)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "email": email, "password": "Sup3rSecret!" }).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert!(
        status.is_success(),
        "staff registration for {host} failed: {status} {body}"
    );
    body["token"]
        .as_str()
        .expect("registration issues a token")
        .to_string()
}

#[tokio::test]
async fn suspension_notice_is_visible_with_reason_and_pay_url() {
    std::env::set_var("STHENO_PLATFORM_TOKEN", TOKEN);
    let harness = TestApp::spawn().await;
    let router = harness.app.clone();

    // 1. Provision merchant
    let (status, body) = operator(
        &router,
        "POST",
        "/platform/tenants",
        Some(json!({
            "id": TENANT_ID,
            "name": "Notice Test Shop",
            "tier": "free",
            "domain": HOST,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "tenant created: {body}");

    // 2. Staff user registers
    let merchant_jwt = staff_token(&router, HOST, "owner@noticeshop.com").await;

    // 3. Suspend tenant with a specific reason
    let suspend_reason = "Invoice #inv_2026_01 is overdue past 14-day grace period";
    let (status, body) = operator(
        &router,
        "POST",
        &format!("/platform/tenants/{TENANT_ID}/suspend"),
        Some(json!({ "reason": suspend_reason })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tenant suspended: {body}");

    // 4. Merchant accesses admin API -> Returns 403 with structured suspension notice
    let (status, body) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri("/admin/stores")
            .header("host", HOST)
            .header("authorization", format!("Bearer {merchant_jwt}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "suspended merchant should receive 403 Forbidden with notice"
    );
    assert_eq!(body["type"], "tenant_suspended");
    assert_eq!(
        body["reason"], suspend_reason,
        "suspension reason must be surfaced"
    );
    assert!(
        body["pay_url"].as_str().is_some(),
        "pay_url must be provided"
    );
}
