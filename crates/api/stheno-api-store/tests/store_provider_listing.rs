use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::{store_payment_routes, StorePaymentContext};
use stheno_payment::{CredentialMap, PaymentService};
use stheno_payment_stripe::StripePaymentProviderFactory;
use tower::ServiceExt;

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const PROVIDER: &str = "pp_stripe_stripe";
const REGION_A: &str = "reg_store_a";
const REGION_B: &str = "reg_store_b";

async fn app() -> (
    stheno_db::testing::TestDb,
    axum::Router,
    Arc<PaymentService>,
    Arc<stheno_links::PgLinkStore>,
) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_factory(Arc::new(StripePaymentProviderFactory));
    let links = Arc::new(stheno_links::PgLinkStore::new(pool.clone()));
    let router = store_payment_routes(StorePaymentContext {
        payment_service: payment_service.clone(),
        cart_service: Arc::new(stheno_cart::CartService::new(pool)),
        links: links.clone(),
    });
    (db, router, payment_service, links)
}

async fn listed_ids(app: &axum::Router, region_id: &str) -> Vec<String> {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/store/payment-providers?region_id={region_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    body["payment_providers"]
        .as_array()
        .expect("payment providers")
        .iter()
        .map(|provider| provider["id"].as_str().unwrap().to_string())
        .collect()
}

fn stripe_creds() -> CredentialMap {
    CredentialMap::from([
        ("api_key".to_string(), json!("sk_test_store")),
        ("webhook_secret".to_string(), json!("whsec_store")),
        ("base_url".to_string(), json!("https://api.stripe.test")),
        ("livemode".to_string(), json!(false)),
    ])
}

#[tokio::test]
async fn store_lists_only_configured_providers_enabled_for_the_requested_region() {
    let (db, app, payment_service, links) = app().await;
    let tenant = stheno_db::tenant::current().expect("test tenant");

    links
        .link("region_payment_provider", REGION_A, PROVIDER)
        .await
        .expect("enable provider before credentials");
    assert!(
        listed_ids(&app, REGION_A).await.is_empty(),
        "an enabled but unconfigured provider is absent"
    );

    payment_service
        .upsert_provider_credential(&tenant, PROVIDER, &stripe_creds())
        .await
        .expect("credentials");

    assert_eq!(listed_ids(&app, REGION_A).await, vec![PROVIDER.to_string()]);
    assert!(
        listed_ids(&app, REGION_B).await.is_empty(),
        "a provider enabled for one region is absent from another"
    );

    db.drop().await;
}
