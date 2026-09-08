//! P21-01: `POST /hooks/payment/{provider_id}` reaches `PaymentService::receive_webhook`.
//!
//! The service half has been finished since P4-19 and unreachable ever since. These
//! assert the wire actually arrives at it, and that an unknown provider is refused
//! before any work happens.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use stheno_api_store::hooks::{hooks_routes, HooksContext};
use stheno_db::diesel::sql_types::{Integer, Text};
use stheno_db::diesel::{self};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant, DbPool};
use stheno_payment::{encrypt_credential_map, CredentialMap, EncryptedCredential, PaymentService};
use stheno_payment_offline::OfflinePaymentProviderFactory;
use tower::ServiceExt;

const PROVIDER: &str = "pp_system_default";
const TENANT: &str = TestDb::TEST_TENANT;
const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

async fn app() -> (TestDb, axum::Router, Arc<PaymentService>) {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    seed_offline_credential(pool.clone()).await;
    let payment_service = Arc::new(PaymentService::new(pool));
    payment_service.register_factory(Arc::new(OfflinePaymentProviderFactory));

    let router = hooks_routes(HooksContext {
        payment_service: payment_service.clone(),
        fulfillment_service: None,
    });

    (db, router, payment_service)
}

async fn seed_offline_credential(pool: DbPool) {
    let encrypted = encrypt_credential_map(TENANT, PROVIDER, &CredentialMap::new())
        .expect("empty COD credentials encrypt");
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
                .bind::<Text, _>("ppcred_hook_offline")
                .bind::<Text, _>(PROVIDER)
                .bind::<Text, _>(hex::encode(encrypted.ciphertext))
                .bind::<Text, _>(hex::encode(encrypted.nonce))
                .bind::<Integer, _>(1)
                .bind::<Text, _>("hook_offline")
                .execute(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
        .expect("insert offline credential");
    })
    .await;
}

#[tokio::test]
async fn a_posted_webhook_reaches_the_service_and_is_recorded() {
    let (db, router, _svc) = app().await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/hooks/payment/{PROVIDER}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"event":"payment.captured"}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "a signed-enough webhook for a registered provider must be accepted"
    );

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");

    assert_eq!(body["provider_id"], PROVIDER);
    assert_eq!(
        body["state"], "received",
        "a first delivery is processed, not ignored"
    );
    assert!(
        body["id"].as_str().is_some_and(|id| !id.is_empty()),
        "the route must return the stored event id, which is the proof a row was written"
    );

    db.drop().await;
}

#[tokio::test]
async fn an_unknown_provider_is_refused() {
    let (db, router, _svc) = app().await;

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/hooks/payment/pp_not_registered")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"event":"payment.captured"}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "an unregistered provider id must not reach the service"
    );

    db.drop().await;
}
