//! P32-01: Provider oversight: real provider registry with per-region & per-tenant enablement.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::PaymentService;
use stheno_payment_bkash::BkashPaymentProviderFactory;
use stheno_payment_offline::OfflinePaymentProviderFactory;
use stheno_payment_sslcommerz::SslcommerzPaymentProviderFactory;
use stheno_payment_stripe::StripePaymentProviderFactory;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "oversight-bootstrap-secret";
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
async fn provider_oversight_returns_registered_providers() {
    let (_db, router) = app().await;

    let (status, body) = call(&router, "GET", "/platform/providers").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["count"].as_u64().unwrap() >= 4);

    // The ids the factories actually report, not the shortened ones the handler used to
    // invent. `pp_bkash_bkash` is `pp_<provider>_<method>`, which is the id credentials and
    // the outbound ledger are keyed by — asserting `pp_bkash` here passed only while this
    // endpoint answered from a literal.
    let providers = body["providers"].as_array().expect("array");
    assert!(providers.iter().any(|p| p["id"] == "pp_bkash_bkash"));
    assert!(providers
        .iter()
        .any(|p| p["id"] == "pp_sslcommerz_sslcommerz"));
    assert!(providers.iter().any(|p| p["id"] == "pp_stripe_stripe"));
    assert!(providers.iter().any(|p| p["id"] == "pp_system_default"));

    // No courier here. `fp_steadfast` is a fulfillment provider and is reported by
    // `/platform/couriers`, which `courier_oversight.rs` covers. The old literal mixed the
    // two registries into one list, and this assertion followed it there.
    assert!(providers.iter().all(|p| p["kind"] == "payment"));
}
