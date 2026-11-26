//! P30-14: platform billing routes expose invoices, manual retry and credit.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{GenerateInvoiceInput, PlatformService};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;
use tower::ServiceExt;

const BOOTSTRAP: &str = "billing-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router, Arc<PlatformService>) {
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
    (db, router, platform)
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

async fn invoice(platform: &PlatformService) -> String {
    platform
        .generate_invoice(GenerateInvoiceInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            tier: "pro".to_string(),
            period_start: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
            amount: Money::from_minor_units(150_000, CurrencyCode::Bdt),
            // Added to the DTO in 9f9cb42 (FX-rates work) without updating this test.
            fx_metadata: None,
            payable_amount: None,
        })
        .await
        .expect("invoice")
        .id
}

#[tokio::test]
async fn invoices_are_listed_globally_and_for_one_tenant() {
    let (db, router, platform) = app().await;
    let invoice_id = invoice(&platform).await;

    let (status, body) = call(&router, "GET", "/platform/invoices", None).await;
    assert_eq!(status, StatusCode::OK, "global invoice list: {body}");
    assert_eq!(body["invoices"][0]["id"], invoice_id);

    let (status, body) = call(
        &router,
        "GET",
        &format!("/platform/tenants/{}/invoices", TestDb::TEST_TENANT),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "tenant invoice list: {body}");
    assert_eq!(body["invoices"][0]["tenant_id"], TestDb::TEST_TENANT);

    db.drop().await;
}

#[tokio::test]
async fn retry_and_credit_routes_write_audit_rows() {
    let (db, router, platform) = app().await;
    let invoice_id = invoice(&platform).await;

    let (status, body) = call(
        &router,
        "POST",
        &format!("/platform/invoices/{invoice_id}/retry"),
        Some(json!({
            "provider_id": "bkash",
            "idempotency_key": format!("manual:{invoice_id}:1"),
            "request_hash": "sha256:manual-retry"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "retry invoice: {body}");
    assert_eq!(body["attempt"]["invoice_id"], invoice_id);

    let (status, body) = call(
        &router,
        "POST",
        &format!("/platform/invoices/{invoice_id}/credit"),
        Some(json!({
            "credit_minor": 20_000,
            "discount_minor": 0,
            "coupon_minor": 5_000,
            "currency": "bdt",
            "reason": "billing adjustment"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "credit invoice: {body}");
    assert_eq!(body["invoice"]["id"], invoice_id);

    let (status, body) = call(&router, "GET", "/platform/audit", None).await;
    assert_eq!(status, StatusCode::OK, "audit trail: {body}");
    let entries = body["entries"].as_array().expect("entries");
    assert!(
        entries
            .iter()
            .any(|e| e["action"] == "invoice.retry" && e["entity_id"] == invoice_id),
        "retry must be audited: {entries:#?}"
    );
    assert!(
        entries
            .iter()
            .any(|e| e["action"] == "invoice.credit" && e["entity_id"] == invoice_id),
        "credit must be audited: {entries:#?}"
    );

    db.drop().await;
}
