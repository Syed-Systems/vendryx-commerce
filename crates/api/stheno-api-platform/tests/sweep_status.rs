//! P32-06: Sweep outcome surfaced: last run, rows converged, rows still stranded.

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
use stheno_payment::PaymentService;
use stheno_payment_bkash::BkashPaymentProviderFactory;
use stheno_payment_offline::OfflinePaymentProviderFactory;
use stheno_payment_sslcommerz::SslcommerzPaymentProviderFactory;
use stheno_payment_stripe::StripePaymentProviderFactory;

const BOOTSTRAP: &str = "sweepstatus-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool.clone()));

    // The oversight routes report on the payment registry, so a context without one has
    // nothing to report and says so. These tests previously passed `payment_service: None`
    // and asserted on values the handlers used to invent — see the note on each assertion.
    // This mirrors what `stheno-server` registers at boot.
    let payment_service = Arc::new(PaymentService::new(pool));
    payment_service.register_factory(Arc::new(OfflinePaymentProviderFactory));
    payment_service.register_factory(Arc::new(StripePaymentProviderFactory));
    payment_service.register_factory(Arc::new(SslcommerzPaymentProviderFactory));
    payment_service.register_factory(Arc::new(BkashPaymentProviderFactory));

    let router = platform_routes(PlatformApiContext {
        platform,
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: Some(payment_service),
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
async fn sweep_status_and_manual_trigger() {
    let (_db, router) = app().await;

    // 1. Check sweep status
    let (status, body) = call(&router, "GET", "/platform/ledger/sweep-status").await;
    assert_eq!(status, StatusCode::OK);
    // False, and that is the point of the field. `is_scheduled` reported `true`
    // unconditionally while CONTROL_PLANE.md D4 records the opposite — no scheduler
    // registers this sweep. An oversight flag that asserts its own health is worst
    // precisely when an operator checks it because they suspect the sweep has stopped.
    assert_eq!(body["sweep_status"]["is_scheduled"], false);

    // 2. Trigger manual sweep run
    let (status, body) = call(&router, "POST", "/platform/ledger/sweep").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["sweep_result"]["scanned_count"].as_u64().is_some());
}
