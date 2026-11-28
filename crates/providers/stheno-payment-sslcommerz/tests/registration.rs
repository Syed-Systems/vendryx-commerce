//! P22-12: stheno-payment-sslcommerz crate skeleton, registered, capabilities honest.

use serde_json::json;
use std::collections::BTreeMap;
use stheno_payment::PaymentProviderFactory;
use stheno_payment_sslcommerz::SslcommerzPaymentProviderFactory;

fn valid_credentials() -> stheno_payment::CredentialMap {
    let mut map = BTreeMap::new();
    map.insert("store_id".to_string(), json!("test_store_id"));
    map.insert("store_passwd".to_string(), json!("test_store_passwd"));
    map.insert("is_sandbox".to_string(), json!(true));
    map
}

#[test]
fn sslcommerz_factory_registration_and_schema() {
    let factory = SslcommerzPaymentProviderFactory;
    assert_eq!(factory.provider_id(), "pp_sslcommerz_sslcommerz");

    let schema = factory.credential_schema();
    assert!(schema.iter().any(|f| f.key == "store_id" && f.required));
    assert!(schema.iter().any(|f| f.key == "store_passwd" && f.required));
    assert!(schema.iter().any(|f| f.key == "is_sandbox" && !f.required));
}

#[test]
fn sslcommerz_factory_rejects_missing_credentials() {
    let factory = SslcommerzPaymentProviderFactory;
    let mut incomplete = valid_credentials();
    incomplete.remove("store_id");

    let err = factory
        .build(&incomplete)
        .err()
        .expect("must fail on missing store_id");
    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}

#[test]
fn sslcommerz_factory_builds_and_verifies_honest_capabilities() {
    let factory = SslcommerzPaymentProviderFactory;
    let creds = valid_credentials();
    let provider = factory.build(&creds).expect("build succeeds");

    assert_eq!(provider.id(), "pp_sslcommerz_sslcommerz");

    let caps = provider.capabilities();
    assert!(caps.redirect, "SSLCommerz requires gateway redirect");
    assert!(caps.partial_refund, "SSLCommerz supports partial refund");
    assert!(caps.partial_capture, "SSLCommerz supports partial capture");
    assert!(!caps.multi_capture);
    assert!(
        caps.async_settlement,
        "SSLCommerz IPN settles asynchronously"
    );
}
