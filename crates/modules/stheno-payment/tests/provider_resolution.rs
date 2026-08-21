use std::sync::Arc;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use stheno_db::diesel::sql_types::{Integer, Text};
use stheno_db::diesel::{self};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{tenant, DbPool};
use stheno_payment::{
    encrypt_credential_map, CredentialMap, EncryptedCredential, PaymentProvider, PaymentService,
};
use stheno_payment_stripe::StripePaymentProviderFactory;

type HmacSha256 = Hmac<Sha256>;

const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

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

async fn rotate_credential(
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
                    "UPDATE payment.payment_provider_credential
                        SET ciphertext = decode($1, 'hex'),
                            nonce = decode($2, 'hex'),
                            credential_version = $3,
                            updated_at = now()
                      WHERE provider_id = $4",
                )
                .bind::<Text, _>(hex::encode(encrypted.ciphertext))
                .bind::<Text, _>(hex::encode(encrypted.nonce))
                .bind::<Integer, _>(version)
                .bind::<Text, _>(&provider_id)
                .execute(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
        .expect("rotate credential");
    })
    .await;
}

fn signed_stripe_event(secret: &str) -> (Vec<u8>, String) {
    let ts = Utc::now().timestamp();
    let raw = br#"{"id":"evt_resolution","type":"payment_intent.succeeded","livemode":false}"#;
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
async fn resolves_provider_from_tenant_credentials_and_cache_version() {
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

    let encrypted_v1 =
        encrypt_credential_map("ten_a", "pp_stripe_stripe", &stripe_creds("whsec_v1"))
            .expect("encrypt v1");
    insert_credential(pool.clone(), "ten_a", "pp_stripe_stripe", 1, encrypted_v1).await;

    let provider_v1 = service
        .get_provider("ten_a", "pp_stripe_stripe")
        .await
        .expect("tenant A provider");
    assert!(verifies(provider_v1.clone(), "whsec_v1"));

    let cached_v1 = service
        .get_provider("ten_a", "pp_stripe_stripe")
        .await
        .expect("tenant A provider cached");
    assert!(Arc::ptr_eq(&provider_v1, &cached_v1));

    let tenant_b = service.get_provider("ten_b", "pp_stripe_stripe").await;
    assert!(
        tenant_b.is_err(),
        "tenant B must not see tenant A credential"
    );

    let encrypted_v2 =
        encrypt_credential_map("ten_a", "pp_stripe_stripe", &stripe_creds("whsec_v2"))
            .expect("encrypt v2");
    rotate_credential(pool.clone(), "ten_a", "pp_stripe_stripe", 2, encrypted_v2).await;

    let provider_v2 = service
        .get_provider("ten_a", "pp_stripe_stripe")
        .await
        .expect("tenant A provider after rotation");
    assert!(!Arc::ptr_eq(&provider_v1, &provider_v2));
    assert!(verifies(provider_v2.clone(), "whsec_v2"));
    assert!(!verifies(provider_v2, "whsec_v1"));

    db.drop().await;
}
