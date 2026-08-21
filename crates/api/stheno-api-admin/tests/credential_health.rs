use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_admin::admin_config_routes;
use stheno_currency::CurrencyService;
use stheno_payment::{CredentialMap, PaymentService};
use stheno_payment_stripe::StripePaymentProviderFactory;
use stheno_region::RegionService;
use stheno_sales_channel::SalesChannelService;
use stheno_tax::TaxService;
use tower::ServiceExt;

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const PROVIDER: &str = "pp_stripe_stripe";

async fn call(app: &axum::Router) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/payment-providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

fn stripe_provider(body: &Value) -> &Value {
    body["payment_providers"]
        .as_array()
        .expect("payment providers")
        .iter()
        .find(|provider| provider["id"] == PROVIDER)
        .expect("stripe provider")
}

async fn app() -> (
    stheno_db::testing::TestDb,
    axum::Router,
    Arc<PaymentService>,
) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_factory(Arc::new(StripePaymentProviderFactory));
    let app = admin_config_routes(
        Arc::new(RegionService::new(pool.clone())),
        Arc::new(CurrencyService::new(pool.clone())),
        Arc::new(TaxService::new(pool.clone())),
        Arc::new(SalesChannelService::new(pool.clone())),
        payment_service.clone(),
        Arc::new(stheno_links::PgLinkStore::new(pool)),
    );
    (db, app, payment_service)
}

fn stripe_creds() -> CredentialMap {
    CredentialMap::from([
        ("api_key".to_string(), json!("sk_test_health")),
        ("webhook_secret".to_string(), json!("whsec_health")),
        ("base_url".to_string(), json!("https://api.stripe.test")),
        ("livemode".to_string(), json!(false)),
    ])
}

async fn insert_provider_request(db: &stheno_db::testing::TestDb, id: &str, state: &str) {
    stheno_db::testing::TestDb::execute(
        db,
        &format!(
            "INSERT INTO payment.payment_provider_request (
                 id, idempotency_key, provider_id, operation, resource_type, resource_id,
                 request_hash, state, response, error, attempts, first_sent_at, settled_at, deleted_at
             ) VALUES (
                 '{id}', '{id}_key', '{PROVIDER}', 'authorize', 'payment_session', '{id}_resource',
                 '{id}_hash', '{state}', '{{}}'::jsonb, NULL, 1, now(), now(), NULL
             )"
        ),
    )
    .await;
}

#[tokio::test]
async fn credential_health_comes_from_latest_provider_outcome_not_row_presence() {
    let (db, app, payment_service) = app().await;
    let tenant = stheno_db::tenant::current().expect("test tenant");

    payment_service
        .upsert_provider_credential(&tenant, PROVIDER, &stripe_creds())
        .await
        .expect("credential row");

    let body = call(&app).await;
    assert_eq!(stripe_provider(&body)["credential_health"], "unconfigured");

    insert_provider_request(&db, "ppreq_health_failed", "failed").await;
    let body = call(&app).await;
    assert_eq!(stripe_provider(&body)["credential_health"], "invalid");

    insert_provider_request(&db, "ppreq_health_succeeded", "succeeded").await;
    let body = call(&app).await;
    assert_eq!(stripe_provider(&body)["credential_health"], "configured");

    db.drop().await;
}
