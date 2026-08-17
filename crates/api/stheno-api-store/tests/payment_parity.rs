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
use stheno_payment::{CreatePaymentCollectionInput, PaymentService};
use stheno_payment_offline::OfflinePaymentProvider;
use tower::ServiceExt;

#[tokio::test]
async fn test_store_payment_parity_fixture() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let payment_service = Arc::new(PaymentService::new(pool));
    let offline_provider = Arc::new(OfflinePaymentProvider::new(Some("pp_system_default")));
    payment_service.register_provider(offline_provider);

    let app = store_payment_routes(payment_service.clone());

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
        "provider_id": "pp_system_default"
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
