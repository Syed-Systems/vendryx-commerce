//! P32-02: Credential health per tenant per provider derived from call outcomes.

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

const BOOTSTRAP: &str = "credhealth-bootstrap-secret";
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
async fn credential_health_verdict_returned() {
    let (_db, router) = app().await;

    let (status, body) = call(&router, "GET", "/platform/providers/pp_bkash/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["health"]["provider_id"], "pp_bkash");

    // "unconfigured", not "valid". The handler used to return `total_calls_24h: 124` and a
    // `last_successful_call` of "now" as literals, so it reported a healthy, busy gateway
    // for a provider that had never been called; this test asserted that fiction. Counts
    // now come from the outbound ledger, and a registered provider with no traffic is
    // genuinely unconfigured. Restoring "valid" here would mean restoring the invented
    // traffic, which is the bug rather than the fix.
    assert_eq!(body["health"]["health_verdict"], "unconfigured");
    assert_eq!(body["health"]["total_calls_24h"], 0);
    assert_eq!(body["health"]["failed_auth_count_24h"], 0);
    assert!(body["health"]["last_successful_call"].is_null());
}
