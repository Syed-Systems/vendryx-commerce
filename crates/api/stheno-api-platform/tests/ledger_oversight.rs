//! P32-04: Cross-tenant outbound-ledger view.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "ledger-bootstrap-secret";
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

async fn call(router: &axum::Router, method: &str, path: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {BOOTSTRAP}"))
        .body(Body::empty())
        .expect("request");
    let response = router.clone().oneshot(request).await.expect("response");
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
async fn ledger_oversight_returns_outbound_requests() {
    let (_db, router) = app().await;

    let (status, body) = call(&router, "GET", "/platform/ledger/outbound").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["ledger_requests"].is_array());
}
