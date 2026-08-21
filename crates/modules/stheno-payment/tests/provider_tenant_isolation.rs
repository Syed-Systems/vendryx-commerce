use std::sync::Arc;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use stheno_db::diesel::sql_types::{BigInt, Integer, Text};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{tenant, DbPool};
use stheno_payment::{
    encrypt_credential_map, CredentialMap, EncryptedCredential, PaymentProvider, PaymentService,
};
use stheno_payment_stripe::StripePaymentProviderFactory;

type HmacSha256 = Hmac<Sha256>;

const PROVIDER: &str = "pp_stripe_stripe";
const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    value: i64,
}

fn install_key() {
    std::env::set_var("STHENO_CREDENTIAL_KEY", TEST_KEY_HEX);
}

fn stripe_creds(secret: &str) -> CredentialMap {
    CredentialMap::from([
        ("api_key".to_string(), json!("sk_test_tenant")),
        ("webhook_secret".to_string(), json!(secret)),
        ("base_url".to_string(), json!("https://api.stripe.test")),
        ("livemode".to_string(), json!(false)),
    ])
}

async fn insert_credential(
    pool: DbPool,
    tenant_id: &str,
    provider_id: &str,
    version: i32,
    encrypted: EncryptedCredential,
) {
    let tenant_id = tenant_id.to_string();
    let provider_id = provider_id.to_string();
    tenant::scope(tenant_id.clone(), async move {
        tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "INSERT INTO payment.payment_provider_credential (
                         id, provider_id, ciphertext, nonce, credential_version, webhook_slug
                     ) VALUES (
                         $1, $2, decode($3, 'hex'), decode($4, 'hex'), $5, $6
                     )",
                )
                .bind::<Text, _>(format!("ppcred_{tenant_id}_{provider_id}_{version}"))
                .bind::<Text, _>(&provider_id)
                .bind::<Text, _>(hex::encode(encrypted.ciphertext))
                .bind::<Text, _>(hex::encode(encrypted.nonce))
                .bind::<Integer, _>(version)
                .bind::<Text, _>(format!("hook_{tenant_id}_{provider_id}_{version}"))
                .execute(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
        .expect("insert credential");
    })
    .await;
}

async fn visible_credential_count(pool: DbPool, tenant_id: &str, provider_id: &str) -> i64 {
    let tenant_id = tenant_id.to_string();
    let provider_id = provider_id.to_string();
    tenant::scope(tenant_id, async move {
        tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                let row = diesel::sql_query(
                    "SELECT count(*)::bigint AS value
                       FROM payment.payment_provider_credential
                      WHERE provider_id = $1
                        AND deleted_at IS NULL",
                )
                .bind::<Text, _>(&provider_id)
                .get_result::<CountRow>(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(row.value)
            })
        })
        .await
        .expect("count visible credentials")
    })
    .await
}

fn signed_stripe_event(secret: &str) -> (Vec<u8>, String) {
    let ts = Utc::now().timestamp();
    let raw = br#"{"id":"evt_isolation","type":"payment_intent.succeeded","livemode":false}"#;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
    mac.update(format!("{}.{}", ts, std::str::from_utf8(raw).unwrap()).as_bytes());
    (
        raw.to_vec(),
        format!("t={},v1={}", ts, hex::encode(mac.finalize().into_bytes())),
    )
}

fn verifies(provider: Arc<dyn PaymentProvider>, secret: &str) -> bool {
    let (raw, signature) = signed_stripe_event(secret);
    provider.verify_webhook(&raw, Some(&signature)).is_ok()
}

#[tokio::test]
async fn tenant_credentials_are_query_isolated_and_build_distinct_stripe_providers() {
    install_key();
    let db = stheno_db::testing::TestDb::migrated().await;
    stheno_db::testing::TestDb::execute(
        &db,
        "INSERT INTO platform.tenant (id, name) VALUES ('ten_a','A'), ('ten_b','B')
         ON CONFLICT (id) DO NOTHING",
    )
    .await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool.clone());
    service.register_factory(Arc::new(StripePaymentProviderFactory));

    let encrypted_a = encrypt_credential_map("ten_a", PROVIDER, &stripe_creds("whsec_a"))
        .expect("encrypt tenant A");
    insert_credential(pool.clone(), "ten_a", PROVIDER, 1, encrypted_a).await;

    assert_eq!(
        visible_credential_count(pool.clone(), "ten_a", PROVIDER).await,
        1
    );
    assert_eq!(
        visible_credential_count(pool.clone(), "ten_b", PROVIDER).await,
        0,
        "tenant B must not see tenant A's provider row by query"
    );
    assert!(
        service.get_provider("ten_b", PROVIDER).await.is_err(),
        "tenant B must not build a provider from tenant A credentials"
    );

    let encrypted_b = encrypt_credential_map("ten_b", PROVIDER, &stripe_creds("whsec_b"))
        .expect("encrypt tenant B");
    insert_credential(pool.clone(), "ten_b", PROVIDER, 1, encrypted_b).await;

    let tenant_a = service
        .get_provider("ten_a", PROVIDER)
        .await
        .expect("tenant A provider");
    let tenant_b = service
        .get_provider("ten_b", PROVIDER)
        .await
        .expect("tenant B provider");

    assert!(!Arc::ptr_eq(&tenant_a, &tenant_b));
    assert!(verifies(tenant_a.clone(), "whsec_a"));
    assert!(!verifies(tenant_a, "whsec_b"));
    assert!(verifies(tenant_b.clone(), "whsec_b"));
    assert!(!verifies(tenant_b, "whsec_a"));

    db.drop().await;
}
