//! P36-03: MFA on the control plane with TOTP secret setup and verification.

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

const BOOTSTRAP: &str = "mfa-bootstrap-secret";
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
async fn operator_mfa_setup_and_verify_flow() {
    let (_db, router) = app().await;

    // 1. Issue operator
    let (status, op_body) = call(
        &router,
        "POST",
        "/platform/operators",
        Some(json!({ "name": "Security Officer" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let op_id = op_body["operator"]["id"].as_str().expect("id");

    // 2. Setup MFA
    let (status, mfa_setup) = call(
        &router,
        "POST",
        &format!("/platform/operators/{op_id}/mfa/setup"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(mfa_setup["secret"].is_string());
    assert!(mfa_setup["qr_uri"].is_string());

    // 3. Verify MFA code
    let (status, verify_body) = call(
        &router,
        "POST",
        &format!("/platform/operators/{op_id}/mfa/verify"),
        Some(json!({ "code": "123456" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(verify_body["mfa_enabled"], true);
}
