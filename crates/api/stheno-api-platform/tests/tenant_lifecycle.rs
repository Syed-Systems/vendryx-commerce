//! P31-01: tenant suspension/reactivation is explicit, audited, and reasoned.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;
use tower::ServiceExt;

const BOOTSTRAP: &str = "lifecycle-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let router = platform_routes(PlatformApiContext {
        platform: Arc::new(PlatformService::new(pool)),
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
        .oneshot(builder.body(body).expect("request builds"))
        .await
        .expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn tenant_status(router: &axum::Router, tenant_id: &str) -> String {
    let (status, body) = call(router, "GET", "/platform/tenants", None).await;
    assert_eq!(status, StatusCode::OK, "tenant list reads: {body}");
    body["tenants"]
        .as_array()
        .expect("tenants")
        .iter()
        .find(|tenant| tenant["id"] == tenant_id)
        .unwrap_or_else(|| panic!("tenant {tenant_id} not found in {body}"))["status"]
        .as_str()
        .expect("status")
        .to_string()
}

#[tokio::test]
async fn suspend_and_reactivate_write_tenant_status_and_audit_reason() {
    let (db, router) = app().await;

    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_test/suspend",
        Some(json!({ "reason": "invoice overdue" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "suspend succeeds: {body}");
    assert_eq!(body["tenant"]["status"], "suspended");
    assert_eq!(tenant_status(&router, "ten_test").await, "suspended");

    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_test/reactivate",
        Some(json!({ "reason": "payment received" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "reactivate succeeds: {body}");
    assert_eq!(body["tenant"]["status"], "active");

    let (status, body) = call(&router, "GET", "/platform/audit", None).await;
    assert_eq!(status, StatusCode::OK, "audit reads: {body}");
    let entries = body["entries"].as_array().expect("entries");
    assert!(
        entries.iter().any(|entry| {
            entry["action"] == "tenant.suspend"
                && entry["entity_id"] == "ten_test"
                && entry["diff"]["reason"] == "invoice overdue"
        }),
        "suspension reason must be audited: {entries:#?}"
    );
    assert!(
        entries.iter().any(|entry| {
            entry["action"] == "tenant.reactivate"
                && entry["entity_id"] == "ten_test"
                && entry["diff"]["reason"] == "payment received"
        }),
        "reactivation reason must be audited: {entries:#?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn suspend_requires_a_reason() {
    let (db, router) = app().await;

    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_test/suspend",
        Some(json!({ "reason": "   " })),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "blank reason refused: {body}"
    );
    assert_eq!(tenant_status(&router, "ten_test").await, "active");

    db.drop().await;
}

#[tokio::test]
async fn close_tenant_requires_confirmation_and_is_audited() {
    let (db, router) = app().await;

    // 1. Mismatched confirmation id is refused
    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_test/close",
        Some(json!({
            "confirm_id": "wrong_id",
            "reason": "Merchant requested account deletion"
        })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "mismatched confirmation refused: {body}"
    );

    // 2. Matching confirmation id closes tenant
    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_test/close",
        Some(json!({
            "confirm_id": "ten_test",
            "reason": "Merchant requested account deletion"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "close succeeds: {body}");
    assert_eq!(body["tenant"]["status"], "closed");
    assert_eq!(tenant_status(&router, "ten_test").await, "closed");

    // 3. Audit trail contains close action and reason
    let (status, body) = call(&router, "GET", "/platform/audit", None).await;
    assert_eq!(status, StatusCode::OK, "audit reads: {body}");
    let entries = body["entries"].as_array().expect("entries");
    assert!(
        entries.iter().any(|entry| {
            entry["action"] == "tenant.close"
                && entry["entity_id"] == "ten_test"
                && entry["diff"]["reason"] == "Merchant requested account deletion"
        }),
        "close reason must be audited: {entries:#?}"
    );

    db.drop().await;
}
