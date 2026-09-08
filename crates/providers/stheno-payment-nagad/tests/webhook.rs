//! P22-10: [BP] Nagad verify_webhook / map_webhook with tampered-signature rejection.

use serde_json::json;
use stheno_payment::{PaymentProvider, WebhookAction};
use stheno_payment_nagad::NagadPaymentProvider;

#[test]
fn nagad_webhook_valid_payload_maps_to_captured() {
    let provider = NagadPaymentProvider::new(
        "https://sandbox.nagad.com.bd/remote-payment-gateway-1.0/api/dfs",
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let payload = json!({
        "payment_ref_id": "NAGAD_REF_998877",
        "order_id": "ord_nagad_101",
        "status": "Success",
        "signature": "valid_rsa_sha1_signature_hash",
        "amount": "1850.00"
    });

    let verified = provider
        .verify_webhook(payload.to_string().as_bytes(), None)
        .expect("verify succeeds");

    assert_eq!(verified.event_type, "Success");
    assert_eq!(verified.provider_event_id, "NAGAD_REF_998877");

    let action = provider.map_webhook(&verified).expect("map succeeds");
    assert_eq!(action, WebhookAction::Captured);
}

#[test]
fn nagad_webhook_rejects_tampered_signature() {
    let provider = NagadPaymentProvider::new(
        "https://sandbox.nagad.com.bd/remote-payment-gateway-1.0/api/dfs",
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let payload = json!({
        "payment_ref_id": "NAGAD_REF_998877",
        "order_id": "ord_nagad_101",
        "status": "Success",
        "signature": "tampered_forged_sig",
        "amount": "1850.00"
    });

    let err = provider
        .verify_webhook(payload.to_string().as_bytes(), None)
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}

#[test]
fn nagad_webhook_rejects_missing_event_id() {
    let provider = NagadPaymentProvider::new(
        "https://sandbox.nagad.com.bd/remote-payment-gateway-1.0/api/dfs",
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let payload = json!({
        "status": "Success",
        "signature": "valid_sig"
    });

    let err = provider
        .verify_webhook(payload.to_string().as_bytes(), None)
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
