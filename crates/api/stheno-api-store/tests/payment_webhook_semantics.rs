//! P21-02: Route returns 401 on signature failure and writes no row; 200 on replay without reprocessing.

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
const WEBHOOK_SECRET: &str = "whsec_test_semantics_secret_123";

async fn app() -> (TestDb, axum::Router, DbPool, Arc<PaymentService>) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    seed_stripe_credential(pool.clone()).await;
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    payment_service.register_factory(Arc::new(StripePaymentProviderFactory));

    let router = hooks_routes(HooksContext {
        payment_service: payment_service.clone(),
        fulfillment_service: None,
    });

    (db, router, pool, payment_service)
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
                .bind::<Text, _>("ppcred_stripe_semantics")
                .bind::<Text, _>(PROVIDER)
                .bind::<Text, _>(hex::encode(encrypted.ciphertext))
                .bind::<Text, _>(hex::encode(encrypted.nonce))
                .bind::<Integer, _>(1)
                .bind::<Text, _>("hook_stripe")
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

async fn count_stored_events(pool: &DbPool) -> i64 {
    tenant::scope(TENANT.to_string(), {
        let pool = pool.clone();
        async move {
            tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    use stheno_db::diesel::sql_types::BigInt;
                    #[derive(stheno_db::diesel::QueryableByName)]
                    struct CountRow {
                        #[diesel(sql_type = BigInt)]
                        count: i64,
                    }
                    let rows: Vec<CountRow> = diesel::sql_query(
                        "SELECT count(*) as count FROM payment.payment_webhook_event WHERE provider_id = $1 AND deleted_at IS NULL",
                    )
                    .bind::<Text, _>(PROVIDER)
                    .load(conn)
                    .await
                    .map_err(|e| stheno_db::Error::Database(e.to_string()))?;

                    let count = rows.get(0).map(|r| r.count).unwrap_or(0);
                    Ok(count)
                })
            })
            .await
            .unwrap_or(0)
        }
    })
    .await
}

fn sign_stripe_payload(payload: &[u8], secret: &str, timestamp: i64) -> String {
    let signed_payload = format!("{}.{}", timestamp, std::str::from_utf8(payload).unwrap());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signed_payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("t={},v1={}", timestamp, sig)
}

#[tokio::test]
async fn invalid_signature_returns_401_and_writes_no_row() {
    let (db, router, pool, _svc) = app().await;

    let payload =
        br#"{"id":"evt_invalid_sig_1","type":"payment_intent.succeeded","livemode":false}"#;
    let now = Utc::now().timestamp();
    let forged_header = format!(
        "t={},v1=0000000000000000000000000000000000000000000000000000000000000000",
        now
    );

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/hooks/payment/{PROVIDER}"))
                .header("content-type", "application/json")
                .header("stripe-signature", forged_header)
                .body(Body::from(payload.to_vec()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "an invalid signature must return 401 Unauthorized"
    );

    let count = count_stored_events(&pool).await;
    assert_eq!(
        count, 0,
        "a webhook with invalid signature must NOT write any row to payment_webhook_event"
    );

    db.drop().await;
}

#[tokio::test]
async fn replay_returns_200_without_reprocessing_or_duplicate_rows() {
    let (db, router, pool, _svc) = app().await;

    let payload =
        br#"{"id":"evt_replay_test_1","type":"payment_intent.succeeded","livemode":false}"#;
    let now = Utc::now().timestamp();
    let valid_header = sign_stripe_payload(payload, WEBHOOK_SECRET, now);

    // First delivery -> 200 OK, state = received
    let response1 = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/hooks/payment/{PROVIDER}"))
                .header("content-type", "application/json")
                .header("stripe-signature", valid_header.clone())
                .body(Body::from(payload.to_vec()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response1.status(), StatusCode::OK);
    let bytes1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .expect("body");
    let body1: Value = serde_json::from_slice(&bytes1).expect("json");
    assert_eq!(body1["state"], "received");

    let count_after_first = count_stored_events(&pool).await;
    assert_eq!(
        count_after_first, 1,
        "first delivery writes exactly one row"
    );

    // Second delivery (replay) -> 200 OK, state = ignored
    let response2 = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/hooks/payment/{PROVIDER}"))
                .header("content-type", "application/json")
                .header("stripe-signature", valid_header)
                .body(Body::from(payload.to_vec()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response2.status(), StatusCode::OK);
    let bytes2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .expect("body");
    let body2: Value = serde_json::from_slice(&bytes2).expect("json");
    assert_eq!(body2["state"], "ignored", "replay is marked ignored");

    let count_after_replay = count_stored_events(&pool).await;
    assert_eq!(
        count_after_replay, 1,
        "replay must NOT write an additional row"
    );

    db.drop().await;
}
