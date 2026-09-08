//! P22-01: stheno-payment-bkash crate skeleton, registered, id() and capabilities() honest.

use serde_json::json;
use std::collections::BTreeMap;
use stheno_payment::PaymentProviderFactory;
use stheno_payment_bkash::BkashPaymentProviderFactory;

fn valid_credentials() -> stheno_payment::CredentialMap {
    let mut map = BTreeMap::new();
    map.insert("base_url".to_string(), json!("https://tokenized.sandbox.bka.sh/v1.2.0-beta"));
    map.insert("app_key".to_string(), json!("test_app_key"));
    map.insert("app_secret".to_string(), json!("test_app_secret"));
    map.insert("username".to_string(), json!("test_username"));
    map.insert("password".to_string(), json!("test_password"));
    map
}

#[test]
fn bkash_factory_registration_and_schema() {
    let factory = BkashPaymentProviderFactory;
    assert_eq!(factory.provider_id(), "pp_bkash_bkash");

    let schema = factory.credential_schema();
    assert_eq!(schema.len(), 5);

    let required_keys: Vec<&str> = schema.iter().filter(|f| f.required).map(|f| f.key).collect();
    assert!(required_keys.contains(&"base_url"));
    assert!(required_keys.contains(&"app_key"));
    assert!(required_keys.contains(&"app_secret"));
    assert!(required_keys.contains(&"username"));
    assert!(required_keys.contains(&"password"));
}

#[test]
fn bkash_factory_rejects_missing_credentials() {
    let factory = BkashPaymentProviderFactory;
    let mut incomplete = valid_credentials();
    incomplete.remove("app_secret");

    let err = factory.build(&incomplete).err().expect("must fail on missing credentials");
    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}

#[test]
fn bkash_factory_builds_and_verifies_honest_capabilities() {
    let factory = BkashPaymentProviderFactory;
    let creds = valid_credentials();
    let provider = factory.build(&creds).expect("build succeeds");

    assert_eq!(provider.id(), "pp_bkash_bkash");

    let caps = provider.capabilities();
    assert!(caps.redirect, "bKash requires browser redirect");
    assert!(caps.partial_refund, "bKash supports partial refund");
    assert!(!caps.partial_capture, "bKash execute is full-amount only");
    assert!(!caps.multi_capture, "bKash execute is one-shot");
    assert!(caps.cancel_after_authorize);
    assert!(!caps.vault, "bKash tokenized checkout manages agreements gateway-side");
    assert_eq!(caps.authorization_ttl, Some(std::time::Duration::from_secs(24 * 3600)));
}
