//! P22-07: stheno-payment-nagad crate skeleton, registered, capabilities honest.

use serde_json::json;
use std::collections::BTreeMap;
use stheno_payment::PaymentProviderFactory;
use stheno_payment_nagad::NagadPaymentProviderFactory;

fn valid_credentials() -> stheno_payment::CredentialMap {
    let mut map = BTreeMap::new();
    map.insert("base_url".to_string(), json!("https://sandbox.nagad.com.bd/remote-payment-gateway-1.0/api/dfs"));
    map.insert("merchant_id".to_string(), json!("TEST_MERCHANT_01"));
    map.insert("merchant_private_key".to_string(), json!("-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----"));
    map.insert("nagad_public_key".to_string(), json!("-----BEGIN PUBLIC KEY-----\nMIIB...\n-----END PUBLIC KEY-----"));
    map
}

#[test]
fn nagad_factory_registration_and_schema() {
    let factory = NagadPaymentProviderFactory;
    assert_eq!(factory.provider_id(), "pp_nagad_nagad");

    let schema = factory.credential_schema();
    assert_eq!(schema.len(), 4);

    let required_keys: Vec<&str> = schema.iter().filter(|f| f.required).map(|f| f.key).collect();
    assert!(required_keys.contains(&"base_url"));
    assert!(required_keys.contains(&"merchant_id"));
    assert!(required_keys.contains(&"merchant_private_key"));
    assert!(required_keys.contains(&"nagad_public_key"));
}

#[test]
fn nagad_factory_rejects_missing_credentials() {
    let factory = NagadPaymentProviderFactory;
    let mut incomplete = valid_credentials();
    incomplete.remove("merchant_private_key");

    let err = factory.build(&incomplete).err().expect("must fail on missing merchant_private_key");
    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}

#[test]
fn nagad_factory_builds_and_verifies_honest_capabilities() {
    let factory = NagadPaymentProviderFactory;
    let creds = valid_credentials();
    let provider = factory.build(&creds).expect("build succeeds");

    assert_eq!(provider.id(), "pp_nagad_nagad");

    let caps = provider.capabilities();
    assert!(caps.redirect, "Nagad requires redirect to DFS gateway");
    assert!(!caps.partial_refund, "Nagad does not support partial refund; full refund required");
    assert!(!caps.partial_capture, "Nagad does not support partial capture");
    assert!(!caps.multi_capture);
    assert!(caps.cancel_after_authorize);
}
