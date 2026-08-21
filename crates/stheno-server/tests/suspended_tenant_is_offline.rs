//! P31-02: A suspended tenant's /store/* requests are refused at the tenant layer.
//! Asserts the storefront actually stops serving when suspended, not just that the database column changed.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use stheno_server::TestApp;
use tower::ServiceExt;

const TOKEN: &str = "suspended-offline-token";
const HOST: &str = "offline-shop.localhost";
const TENANT_ID: &str = "ten_offline_store";

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

#[tokio::test]
async fn suspended_tenant_is_offline() {
    std::env::set_var("STHENO_PLATFORM_TOKEN", TOKEN);
    let harness = TestApp::spawn().await;
    let router = harness.app.clone();

    // 1. Provision active merchant
    let (status, body) = operator(
        &router,
        "POST",
        "/platform/tenants",
        Some(json!({
            "id": TENANT_ID,
            "name": "Offline Test Shop",
            "tier": "free",
            "domain": HOST,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "tenant created: {body}");

    // 2. Assert storefront is serving normally when active
    let (status, _body) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri("/store/regions")
            .header("host", HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "storefront should be active");

    // 3. Operator suspends the merchant
    let (status, body) = operator(
        &router,
        "POST",
        &format!("/platform/tenants/{TENANT_ID}/suspend"),
        Some(json!({
            "reason": "Non-payment of subscription tier fee"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tenant suspended: {body}");

    // 4. Assert storefront refuses requests and is offline (503 Service Unavailable)
    let (status, body) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri("/store/regions")
            .header("host", HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "suspended storefront must be offline"
    );
    assert_eq!(body["type"], "tenant_suspended");

    // 5. Operator reactivates the merchant
    let (status, body) = operator(
        &router,
        "POST",
        &format!("/platform/tenants/{TENANT_ID}/reactivate"),
        Some(json!({ "reason": "Payment received in full" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tenant reactivated: {body}");

    // 6. Assert storefront resumes serving
    let (status, _body) = call(
        &router,
        Request::builder()
            .method("GET")
            .uri("/store/regions")
            .header("host", HOST)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "storefront should be online again");
}
