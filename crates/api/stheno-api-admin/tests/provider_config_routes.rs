use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_admin::admin_config_routes;
use stheno_currency::CurrencyService;
use stheno_payment::PaymentService;
use stheno_payment_offline::OfflinePaymentProviderFactory;
use stheno_payment_stripe::StripePaymentProviderFactory;
use stheno_region::RegionService;
use stheno_sales_channel::SalesChannelService;
use stheno_tax::TaxService;
use tower::ServiceExt;

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const REGION: &str = "reg_provider_config";
const STRIPE: &str = "pp_stripe_stripe";

async fn call(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

fn provider<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["payment_providers"]
        .as_array()
        .expect("payment_providers is an array")
        .iter()
        .find(|provider| provider["id"] == id)
        .unwrap_or_else(|| panic!("{id} provider is listed"))
}

async fn app() -> (stheno_db::testing::TestDb, axum::Router) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");

    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_factory(Arc::new(OfflinePaymentProviderFactory));
    payment_service.register_factory(Arc::new(StripePaymentProviderFactory));

    let router = admin_config_routes(
        Arc::new(RegionService::new(pool.clone())),
        Arc::new(CurrencyService::new(pool.clone())),
        Arc::new(TaxService::new(pool.clone())),
        Arc::new(SalesChannelService::new(pool.clone())),
        payment_service,
        Arc::new(stheno_links::PgLinkStore::new(pool)),
    );

    (db, router)
}

#[tokio::test]
async fn payment_provider_config_lists_registry_credentials_and_region_enablement() {
    let (db, app) = app().await;

    let (status, body) = call(
        &app,
        Request::builder()
            .uri("/admin/payment-providers")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 2);
    assert!(body.to_string().contains("pp_system_default"));
    assert!(body.to_string().contains(STRIPE));
    assert!(!body.to_string().contains("\"stripe\""));
    assert!(!body.to_string().contains("manual_payment"));
    assert_eq!(provider(&body, STRIPE)["configured"], false);
    assert_eq!(
        provider(&body, STRIPE)["credential_schema"][0]["key"],
        "api_key"
    );

    let credentials = json!({
        "credentials": {
            "api_key": "sk_test_admin",
            "webhook_secret": "whsec_admin",
            "base_url": "https://api.stripe.test",
            "livemode": false
        }
    });
    let (status, body) = call(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/admin/payment-providers/{STRIPE}/credentials"))
            .header("content-type", "application/json")
            .body(Body::from(credentials.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !body.to_string().contains("sk_test_admin") && !body.to_string().contains("whsec_admin"),
        "credential responses must stay structurally opaque"
    );
    assert_eq!(body["payment_provider"]["configured"], true);

    let (status, _) = call(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!(
                "/admin/regions/{REGION}/payment-providers/{STRIPE}"
            ))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = call(
        &app,
        Request::builder()
            .uri("/admin/payment-providers")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(provider(&body, STRIPE)["configured"], true);
    assert_eq!(provider(&body, STRIPE)["enabled_regions"][0], REGION);

    let (status, _) = call(
        &app,
        Request::builder()
            .method("DELETE")
            .uri(format!(
                "/admin/regions/{REGION}/payment-providers/{STRIPE}"
            ))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = call(
        &app,
        Request::builder()
            .uri("/admin/payment-providers")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert!(provider(&body, STRIPE)["enabled_regions"]
        .as_array()
        .expect("enabled regions")
        .is_empty());
    assert_eq!(
        provider(&body, STRIPE)["callback_url"],
        format!("https://api.vendryx.com/hooks/payment/{STRIPE}")
    );

    // Verify offline provider endpoint returns healthy
    let (status, verify_body) = call(
        &app,
        Request::builder()
            .method("POST")
            .uri("/admin/payment-providers/pp_system_default/verify")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(verify_body["status"], "healthy");
    assert_eq!(verify_body["verified"], true);

    db.drop().await;
}
