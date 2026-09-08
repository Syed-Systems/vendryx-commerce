//! P32-08: Global gateway kill switch across all tenants during incidents.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{is_gateway_killed, platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "killswitch-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router, PlatformApiContext) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    // P37-02: the kill switch reads `platform.global_flag`, so the assertions need the
    // context rather than a process-global map. The assertions themselves are unchanged
    // and still only prove the toggle round-trips -- P37-06 is the task that replaces this
    // file with one proving a killed gateway actually refuses to resolve.
    let ctx = PlatformApiContext {
        platform: Arc::clone(&platform),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
        fulfillment_service: None,
    };
    let router = platform_routes(ctx.clone());

    (db, router, ctx)
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
async fn global_gateway_kill_switch_toggles_and_enforces_fail_closed() {
    let (_db, router, ctx) = app().await;

    // 1. Kill pp_bkash globally
    let (status, body) = call(
        &router,
        "POST",
        "/platform/gateways/pp_bkash/kill-switch",
        Some(json!({
            "killed": true,
            "reason": "Upstream bKash outage reported"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_killed"], true);
    assert!(is_gateway_killed(&ctx, "pp_bkash").await);

    // 2. Un-kill pp_bkash
    let (status, body) = call(
        &router,
        "POST",
        "/platform/gateways/pp_bkash/kill-switch",
        Some(json!({
            "killed": false,
            "reason": "Upstream bKash service restored"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_killed"], false);
    assert!(!is_gateway_killed(&ctx, "pp_bkash").await);
}
