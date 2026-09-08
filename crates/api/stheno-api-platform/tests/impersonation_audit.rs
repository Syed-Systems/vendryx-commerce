//! P34-03: Impersonation start, write, and end are audited in operator_audit.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "impaudit-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    let router = platform_routes(PlatformApiContext {
        platform: Arc::clone(&platform),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
        fulfillment_service: None,
    });

    (db, router)
}

async fn call(
    router: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {BOOTSTRAP}"));
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = router
        .clone()
        .oneshot(builder.body(body).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("bytes");
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

#[tokio::test]
async fn impersonation_lifecycle_writes_audit_rows() {
    let (_db, router) = app().await;

    // 1. Start impersonation
    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_audit_test/impersonate",
        Some(json!({
            "duration_minutes": 15,
            "reason": "Fix tax configuration issue"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = body["impersonation"]["token"].as_str().unwrap();

    // 2. End impersonation
    let (status, _) = call(
        &router,
        "POST",
        "/platform/tenants/ten_audit_test/impersonate/end",
        Some(json!({ "token": token })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. Verify audit trail contains start and end actions
    let (status, audit_body) = call(&router, "GET", "/platform/audit", None).await;
    assert_eq!(status, StatusCode::OK);
    let rows = audit_body["entries"].as_array().expect("array");
    assert!(rows.iter().any(|r| r["action"] == "impersonation.start"));
    assert!(rows.iter().any(|r| r["action"] == "impersonation.end"));
}
