//! P21-03: [BP] Raw-body preservation assertion — a byte-exact fixture survives
//! the middleware stack unparsed (PHASE3_TASKS.md P21-03).
//!
//! A middleware that deserialises and re-serialises JSON changes whitespace, key ordering,
//! and float formatting, which silently breaks HMAC signature verification in production.
//! This test asserts that raw bytes survive unmolested across HTTP extraction.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_store::hooks::{hooks_routes, HooksContext};
use stheno_db::diesel::sql_types::{Integer, Text};
use stheno_db::diesel::{self};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant, DbPool};
use stheno_payment::{encrypt_credential_map, CredentialMap, EncryptedCredential, PaymentService};
use stheno_payment_stripe::StripePaymentProviderFactory;

type HmacSha256 = Hmac<Sha256>;

const PROVIDER: &str = "pp_stripe_stripe";
const TENANT: &str = TestDb::TEST_TENANT;
const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const WEBHOOK_SECRET: &str = "whsec_raw_body_secret_key_456";

async fn app() -> (TestDb, axum::Router, Arc<PaymentService>) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    seed_stripe_credential(pool.clone()).await;
    let payment_service = Arc::new(PaymentService::new(pool));
    payment_service.register_factory(Arc::new(StripePaymentProviderFactory));

    let router = hooks_routes(HooksContext {
        payment_service: payment_service.clone(),
        fulfillment_service: None,
    });

    (db, router, payment_service)
}

async fn seed_stripe_credential(pool: DbPool) {
    let creds = CredentialMap::from([
        ("api_key".to_string(), json!("sk_test_mock")),
        ("webhook_secret".to_string(), json!(WEBHOOK_SECRET)),
        ("base_url".to_string(), json!("https://api.stripe.com")),
        ("livemode".to_string(), json!(false)),
    ]);
    let encrypted = encrypt_credential_map(TENANT, PROVIDER, &creds).expect("credentials encrypt");
    insert_credential(pool, encrypted).await;
}

async fn insert_credential(pool: DbPool, encrypted: EncryptedCredential) {
    tenant::scope(TENANT.to_string(), async move {
        tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "INSERT INTO payment.payment_provider_credential (
                         id, provider_id, ciphertext, nonce, credential_version, webhook_slug
                     ) VALUES (
                         $1, $2, decode($3, 'hex'), decode($4, 'hex'), $5, $6
                     )",
                )
                .bind::<Text, _>("ppcred_stripe_raw_body")
                .bind::<Text, _>(PROVIDER)
                .bind::<Text, _>(hex::encode(encrypted.ciphertext))
                .bind::<Text, _>(hex::encode(encrypted.nonce))
                .bind::<Integer, _>(1)
                .bind::<Text, _>("hook_stripe_raw")
                .execute(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
        .expect("insert stripe credential");
    })
    .await;
}

fn sign_stripe_payload(payload: &[u8], secret: &str, timestamp: i64) -> String {
    let signed_payload = format!("{}.{}", timestamp, std::str::from_utf8(payload).unwrap());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signed_payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("t={},v1={}", timestamp, sig)
}

#[tokio::test]
async fn raw_body_preservation_survives_middleware_unparsed() {
    let (db, router, _svc) = app().await;

    // A byte fixture deliberately shaped with non-standard formatting, tab characters,
    // intentional trailing spaces, and inverted field order.
    // If parsed to a JSON Value and serialized back, the byte stream changes and fails HMAC.
    let raw_payload = b"{\n\t\"livemode\": false ,\n\t\"data\": {\n\t\t\"object\": { \"amount\": 4200 }\n\t},\n\t\"id\": \"evt_exact_byte_fixture_777\",\n\t\"type\": \"payment_intent.succeeded\"\n}";

    // Proving the Breaking Point (BP): verify that canonical re-serialization DOES NOT match raw bytes
    let parsed: Value = serde_json::from_slice(raw_payload).expect("valid json");
    let re_serialized = serde_json::to_vec(&parsed).expect("serialized");
    assert_ne!(
        raw_payload.to_vec(),
        re_serialized,
        "Breaking point check: re-serialized JSON must differ from raw byte fixture"
    );

    let now = Utc::now().timestamp();
    let signature_header = sign_stripe_payload(raw_payload, WEBHOOK_SECRET, now);

    // Send the raw byte fixture through HTTP router
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/hooks/payment/{PROVIDER}"))
                .header("content-type", "application/json")
                .header("stripe-signature", signature_header)
                .body(Body::from(raw_payload.to_vec()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "raw body must reach the signature verifier bit-for-bit without middleware re-serialization"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");

    assert_eq!(body["state"], "received");
    assert_eq!(body["provider_id"], PROVIDER);

    db.drop().await;
}
