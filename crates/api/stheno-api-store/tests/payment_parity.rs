//! Medusa v2 Store API Payment Parity Fixture (TASKS P13-13, P4-29).
//! Tests payment collection sessions, authorizations, and provider listing.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_store::store_payment_routes;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::{CreatePaymentCollectionInput, CredentialMap, PaymentService};
use stheno_payment_offline::OfflinePaymentProviderFactory;
use tower::ServiceExt;

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const PROVIDER: &str = "pp_system_default";
const REGION: &str = "reg_payment_parity";

#[tokio::test]
async fn test_store_payment_parity_fixture() {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_factory(Arc::new(OfflinePaymentProviderFactory));
    let tenant = stheno_db::tenant::current().expect("test tenant");
    payment_service
        .upsert_provider_credential(&tenant, PROVIDER, &CredentialMap::new())
        .await
        .expect("offline credentials");
    let links = Arc::new(stheno_links::PgLinkStore::new(pool.clone()));
    links
        .link("region_payment_provider", REGION, PROVIDER)
        .await
        .expect("offline enabled");

    // The router opens collections for carts now, so it needs the cart service and the
    // link store alongside payments.
    let app = store_payment_routes(stheno_api_store::StorePaymentContext {
        payment_service: payment_service.clone(),
        cart_service: Arc::new(stheno_cart::CartService::new(pool.clone())),
        links: links.clone(),
    });

    // 1. Seed payment collection
    let col = payment_service
        .create_payment_collection(CreatePaymentCollectionInput {
            currency_code: "usd".to_string(),
            amount: dec!(120.00),
            metadata: None,
        })
        .await
        .expect("payment collection created");

    // 2. GET /store/payment-providers
    let req = Request::builder()
        .method("GET")
        .uri("/store/payment-providers")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(val.get("payment_providers").is_some());
    assert!(!val["payment_providers"].as_array().unwrap().is_empty());

    // 3. POST /store/payment-collections/:id/payment-sessions
    let session_payload = json!({
        "provider_id": PROVIDER
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/store/payment-collections/{}/payment-sessions",
            col.id
        ))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&session_payload).unwrap()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(val.get("payment_collection").is_some());
    let sessions = val["payment_collection"]["payment_sessions"]
        .as_array()
        .unwrap();
    assert_eq!(sessions.len(), 1);
    let session_id = sessions[0]["id"].as_str().unwrap().to_string();

    // 4. POST /store/payment-collections/:id/payment-sessions/:session_id (Authorize)
    let auth_payload = json!({});
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/store/payment-collections/{}/payment-sessions/{}",
            col.id, session_id
        ))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&auth_payload).unwrap()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(val.get("payment").is_some());
    let amt: rust_decimal::Decimal = val["payment"]["amount"].as_str().unwrap().parse().unwrap();
    assert_eq!(amt, dec!(120.00));
}
