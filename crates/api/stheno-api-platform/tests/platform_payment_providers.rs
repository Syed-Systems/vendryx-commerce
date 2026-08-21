use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::PaymentService;
use stheno_payment_bkash::BkashPaymentProviderFactory;
use stheno_payment_sslcommerz::SslcommerzPaymentProviderFactory;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "platform-gateways-secret";
const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router, Arc<PaymentService>) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool));

    payment_service.register_factory(Arc::new(BkashPaymentProviderFactory));
    payment_service.register_factory(Arc::new(SslcommerzPaymentProviderFactory));

    let router = platform_routes(PlatformApiContext {
        platform,
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: Some(Arc::clone(&payment_service)),
    });

    (db, router, payment_service)
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
async fn platform_payment_providers_listing_and_credential_upsert() {
    let (_db, router, _payment) = app().await;

    // 1. List platform payment providers before configuration
    let (status, body) = call(&router, "GET", "/platform/payment-providers", None).await;
    assert_eq!(status, StatusCode::OK);
    let providers = body["payment_providers"].as_array().expect("array");
    assert_eq!(providers.len(), 2);

    let bkash = providers
        .iter()
        .find(|p| p["id"] == "pp_bkash_bkash")
        .expect("bkash provider");
    assert_eq!(bkash["configured"], false);
    assert_eq!(bkash["credential_health"], "unconfigured");
    assert!(!bkash["credential_schema"].as_array().unwrap().is_empty());

    // 2. Upsert platform credentials for bKash
    let (status, save_body) = call(
        &router,
        "POST",
        "/platform/payment-providers/pp_bkash_bkash/credentials",
        Some(json!({
            "credentials": {
                "base_url": "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
                "app_key": "plat_bkash_key",
                "app_secret": "plat_bkash_secret",
                "username": "plat_bkash_user",
                "password": "plat_bkash_password"
            }
        })),
    )
    .await;
    eprintln!("save_body: {save_body:?}");
    assert_eq!(status, StatusCode::OK, "save_body: {save_body:?}");
    assert_eq!(save_body["payment_provider"]["configured"], true);

    // 3. Upsert platform credentials for SSLCommerz
    let (status, ssl_body) = call(
        &router,
        "POST",
        "/platform/payment-providers/pp_sslcommerz_sslcommerz/credentials",
        Some(json!({
            "credentials": {
                "store_id": "plat_ssl_store",
                "store_passwd": "plat_ssl_passwd",
                "is_sandbox": true
            }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ssl_body["payment_provider"]["configured"], true);

    // 4. List platform payment providers again — both configured
    let (status, list_body) = call(&router, "GET", "/platform/payment-providers", None).await;
    assert_eq!(status, StatusCode::OK);
    let providers = list_body["payment_providers"].as_array().expect("array");
    for p in providers {
        assert_eq!(p["configured"], true);
    }
}
