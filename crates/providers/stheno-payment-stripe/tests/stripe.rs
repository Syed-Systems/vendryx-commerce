use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use stheno_payment::{PaymentProvider, WebhookAction};
use stheno_payment_stripe::StripePaymentProvider;

type HmacSha256 = Hmac<Sha256>;

#[test]
fn test_stripe_webhook_verification_signature_tolerance_and_livemode() {
    let secret = "whsec_test_secret_key_123";
    let provider = StripePaymentProvider::new("sk_test_123", secret, None, false);

    let raw_payload = br#"{"id":"evt_test_1","type":"payment_intent.succeeded","livemode":false}"#;
    let now = Utc::now().timestamp();

    // 1. Generate valid Stripe-Signature header
    let signed_payload = format!("{}.{}", now, std::str::from_utf8(raw_payload).unwrap());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signed_payload.as_bytes());
    let valid_sig = hex::encode(mac.finalize().into_bytes());

    let header = format!("t={now},v1={valid_sig}");

    // Verify valid webhook
    let verified = provider
        .verify_webhook(raw_payload, Some(&header))
        .expect("signature verification succeeds");

    assert_eq!(verified.provider_event_id, "evt_test_1");
    assert_eq!(verified.event_type, "payment_intent.succeeded");

    let action = provider.map_webhook(&verified).unwrap();
    assert_eq!(action, WebhookAction::Captured);

    // 2. Expired timestamp (> 300s old, E10) -> rejected
    let old_ts = now - 400;
    let old_signed = format!("{}.{}", old_ts, std::str::from_utf8(raw_payload).unwrap());
    let mut mac_old = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac_old.update(old_signed.as_bytes());
    let old_sig = hex::encode(mac_old.finalize().into_bytes());
    let old_header = format!("t={old_ts},v1={old_sig}");

    let err_old = provider
        .verify_webhook(raw_payload, Some(&old_header))
        .unwrap_err();
    match err_old {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "webhook_timestamp_out_of_tolerance");
        }
        _ => panic!("expected Invalid webhook_timestamp_out_of_tolerance"),
    }

    // 3. Forged signature (E14) -> rejected
    let bad_header = format!(
        "t={now},v1=0000000000000000000000000000000000000000000000000000000000000000"
    );
    let err_bad = provider
        .verify_webhook(raw_payload, Some(&bad_header))
        .unwrap_err();
    match err_bad {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "invalid_webhook_signature");
        }
        _ => panic!("expected Invalid invalid_webhook_signature"),
    }

    // 4. Livemode mismatch (test event delivered to live server, E13)
    let live_payload = br#"{"id":"evt_live_1","type":"payment_intent.succeeded","livemode":true}"#;
    let live_signed = format!("{}.{}", now, std::str::from_utf8(live_payload).unwrap());
    let mut mac_live = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac_live.update(live_signed.as_bytes());
    let live_sig = hex::encode(mac_live.finalize().into_bytes());
    let live_header = format!("t={now},v1={live_sig}");

    let err_live = provider
        .verify_webhook(live_payload, Some(&live_header))
        .unwrap_err();
    match err_live {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "webhook_livemode_mismatch");
        }
        _ => panic!("expected Invalid webhook_livemode_mismatch"),
    }
}
