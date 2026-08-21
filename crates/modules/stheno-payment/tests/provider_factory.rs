use std::sync::Arc;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use stheno_db::testing::TestDb;
use stheno_payment::{
    CredentialKind, CredentialMap, InitiateInput, PaymentProviderFactory, PaymentService,
};
use stheno_payment_offline::OfflinePaymentProviderFactory;
use stheno_payment_stripe::StripePaymentProviderFactory;

type HmacSha256 = Hmac<Sha256>;

#[tokio::test]
async fn service_registers_payment_provider_factories() {
    let db = TestDb::migrated().await;
    let service = PaymentService::new(stheno_db::create_pool(&db.url, 2).expect("pool"));

    service.register_factory(Arc::new(OfflinePaymentProviderFactory));
    let factory = service
        .get_factory("pp_system_default")
        .expect("offline factory");

    assert_eq!(factory.provider_id(), "pp_system_default");
    assert!(factory.credential_schema().is_empty());

    db.drop().await;
}

#[tokio::test]
async fn offline_factory_builds_provider_without_credentials() {
    let factory = OfflinePaymentProviderFactory;
    let provider = factory
        .build(&CredentialMap::new())
        .expect("offline provider from empty credentials");

    assert_eq!(provider.id(), "pp_system_default");
    let initiated = provider
        .initiate(InitiateInput {
            amount: rust_decimal_macros::dec!(42),
            currency_code: "bdt".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .expect("offline initiate");
    assert_eq!(initiated.data["is_offline"], json!(true));
}

#[test]
fn stripe_factory_schema_describes_required_credentials() {
    let factory = StripePaymentProviderFactory;
    let schema = factory.credential_schema();

    assert!(schema.iter().any(|field| {
        field.key == "api_key" && field.kind == CredentialKind::Secret && field.required
    }));
    assert!(schema.iter().any(|field| {
        field.key == "webhook_secret" && field.kind == CredentialKind::Secret && field.required
    }));
    assert!(schema.iter().any(|field| {
        field.key == "base_url" && field.kind == CredentialKind::Url && !field.required
    }));
    assert!(schema.iter().any(|field| {
        field.key == "livemode" && field.kind == CredentialKind::Bool && !field.required
    }));
}

#[test]
fn stripe_factory_builds_provider_from_credentials() {
    let factory = StripePaymentProviderFactory;
    let secret = "whsec_factory_test";
    let provider = factory
        .build(&CredentialMap::from([
            ("api_key".to_string(), json!("sk_test_factory")),
            ("webhook_secret".to_string(), json!(secret)),
            ("base_url".to_string(), json!("https://api.stripe.test")),
            ("livemode".to_string(), json!(false)),
        ]))
        .expect("stripe provider from credentials");

    assert_eq!(provider.id(), "pp_stripe_stripe");

    let ts = Utc::now().timestamp();
    let raw = br#"{"id":"evt_factory","type":"payment_intent.succeeded","livemode":false}"#;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
    mac.update(format!("{}.{}", ts, std::str::from_utf8(raw).unwrap()).as_bytes());
    let signature = format!("t={},v1={}", ts, hex::encode(mac.finalize().into_bytes()));

    let event = provider
        .verify_webhook(raw, Some(&signature))
        .expect("webhook verifies with credential secret");
    assert_eq!(event.provider_event_id, "evt_factory");
}

#[test]
fn stripe_factory_rejects_invalid_credentials_at_build_time() {
    let factory = StripePaymentProviderFactory;
    let result = factory.build(&CredentialMap::from([
        ("api_key".to_string(), json!("sk_test_factory")),
        ("webhook_secret".to_string(), json!("whsec_factory")),
        ("base_url".to_string(), json!("not a url")),
    ]));

    let Err(err) = result else {
        panic!("invalid base_url must fail");
    };

    match err {
        stheno_types::Error::Invalid { code, field, .. } => {
            assert_eq!(code, "credential_url_invalid");
            assert_eq!(field.as_deref(), Some("base_url"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
