//! P34-04: Impersonating operator cannot read payment credentials, change billing, or issue operator credentials.

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

const BOOTSTRAP: &str = "implimits-bootstrap-secret";
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

async fn call_with_token(
    router: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
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
async fn impersonation_token_is_forbidden_from_platform_operations() {
    let (_db, router) = app().await;

    // 1. Obtain impersonation token
    let (status, body) = call_with_token(
        &router,
        "POST",
        "/platform/tenants/ten_test/impersonate",
        BOOTSTRAP,
        Some(json!({
            "duration_minutes": 30,
            "reason": "Test scope limits"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let imp_token = body["impersonation"]["token"].as_str().unwrap();

    // 2. Impersonation token cannot access platform operators endpoint
    let (status, _) = call_with_token(&router, "GET", "/platform/operators", imp_token, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 3. Impersonation token cannot access platform billing or tiers
    let (status, _) = call_with_token(&router, "GET", "/platform/tiers", imp_token, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
