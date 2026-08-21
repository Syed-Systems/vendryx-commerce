//! P34-02: Support impersonation token is time-boxed and scope-limited; expiry enforced server-side.

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

const BOOTSTRAP: &str = "imp-bootstrap-secret";
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
async fn impersonation_token_issued_with_expiry_and_scopes() {
    let (_db, router) = app().await;

    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_test/impersonate",
        Some(json!({
            "duration_minutes": 30,
            "reason": "Merchant asked for assistance with inventory sync"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // P37-03. This used to assert `starts_with("imp_ten_test_")`, which pinned the old
    // token format in place — and that format was `imp_{tenant_id}_{timestamp_nanos}`,
    // derived entirely from values an attacker already knows. The assertion was holding
    // the vulnerability still, so it is inverted here: the token must carry the prefix and
    // must NOT contain the tenant id.
    let token = body["impersonation"]["token"].as_str().unwrap();
    assert!(token.starts_with("imp_"), "token is prefixed: {token}");
    assert!(
        !token.contains("ten_test"),
        "a support token must not be derivable from the tenant id: {token}"
    );
    assert!(
        token.len() > 32,
        "token must carry real entropy, not a timestamp: {token}"
    );
    assert_eq!(body["impersonation"]["tenant_id"], "ten_test");
    assert!(body["impersonation"]["expires_at"].is_string());
    assert!(body["impersonation"]["scopes"].is_array());
}
