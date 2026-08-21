//! P34-09: Put one merchant read-only during an incident: writes refused, reads served, reversible.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use stheno_platform::PlatformService;
use stheno_server::TestApp;
use tower::ServiceExt;

const TOKEN: &str = "readonly-test-token";
const TENANT_ID: &str = "ten_incident_readonly";

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
async fn read_only_tenant_mode_toggles_and_enforces_locking() {
    std::env::set_var("STHENO_PLATFORM_TOKEN", TOKEN);
    let harness = TestApp::spawn().await;
    let router = harness.app.clone();

    // P37-04: read-only is a column on `platform.tenant` now, not a process-global map, so
    // the assertions read the row. They are otherwise unchanged and still only prove the
    // toggle round-trips -- P37-05 is the task that replaces this file with one that
    // attempts a write and asserts it is refused, which is what the title above claims.
    let pool = stheno_db::create_pool(&harness.database_url, 5).expect("pool");
    let platform = PlatformService::new(pool);

    // The merchant has to exist before it can be locked. The map this replaced accepted any
    // string and recorded read-only state for a tenant that was never registered, so a typo
    // in the path produced a lock nobody could find again.
    let (status, body) = operator(
        &router,
        "POST",
        "/platform/tenants",
        Some(json!({
            "id": TENANT_ID,
            "name": "Incident Test Merchant",
            "tier": "free"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "tenant is provisioned: {body}");

    // 1. Set tenant to read-only via control-plane API
    let (status, body) = operator(
        &router,
        "POST",
        &format!("/platform/tenants/{TENANT_ID}/read-only"),
        Some(json!({
            "read_only": true,
            "reason": "Database migration in progress for this tenant"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_read_only"], true);
    assert!(platform
        .is_tenant_read_only(TENANT_ID)
        .await
        .expect("read-only state is readable"));

    // 2. Revert read-only mode
    let (status, body) = operator(
        &router,
        "POST",
        &format!("/platform/tenants/{TENANT_ID}/read-only"),
        Some(json!({
            "read_only": false,
            "reason": "Migration complete"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_read_only"], false);
    assert!(!platform
        .is_tenant_read_only(TENANT_ID)
        .await
        .expect("read-only state is readable"));
}
