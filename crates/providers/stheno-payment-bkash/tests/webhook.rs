//! P22-05: [BP] bKash verify_webhook / map_webhook, including a tampered-signature rejection test.

use serde_json::json;
use stheno_payment::{PaymentProvider, WebhookAction};
use stheno_payment_bkash::BkashPaymentProvider;

#[test]
fn bkash_webhook_valid_sns_payload_maps_to_captured() {
    let provider = BkashPaymentProvider::new(
        "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    let inner_msg = json!({
        "paymentID": "TR0011abcd",
        "trxID": "TRX998877",
        "transactionStatus": "Completed",
        "amount": "1200.00",
        "currency": "BDT"
    });

    let sns_payload = json!({
        "Type": "Notification",
        "MessageId": "msg-1234",
        "TopicArn": "arn:aws:sns:ap-southeast-1:1234567890:bkash-notifications",
        "Message": inner_msg.to_string(),
        "Timestamp": "2026-08-21T12:00:00.000Z",
        "SignatureVersion": "1",
        "Signature": "valid_base64_signature_here",
        "SigningCertURL": "https://sns.ap-southeast-1.amazonaws.com/SimpleNotificationService-123.pem"
    });

    let verified = provider
        .verify_webhook(sns_payload.to_string().as_bytes(), None)
        .expect("valid sns webhook verified");

    assert_eq!(verified.event_type, "Notification");
    assert_eq!(verified.provider_event_id, "msg-1234");

    let action = provider.map_webhook(&verified).expect("map succeeds");
    assert_eq!(action, WebhookAction::Captured);
}

#[test]
fn bkash_webhook_rejects_tampered_signature() {
    let provider = BkashPaymentProvider::new(
        "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    let sns_payload = json!({
        "Type": "Notification",
        "MessageId": "msg-1234",
        "TopicArn": "arn:aws:sns:ap-southeast-1:1234567890:bkash-notifications",
        "Message": "{\"transactionStatus\": \"Completed\"}",
        "Timestamp": "2026-08-21T12:00:00.000Z",
        "Signature": "tampered_signature_data",
        "SigningCertURL": "https://sns.ap-southeast-1.amazonaws.com/cert.pem"
    });

    let err = provider
        .verify_webhook(sns_payload.to_string().as_bytes(), None)
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}

#[test]
fn bkash_webhook_rejects_untrusted_signing_cert_url() {
    let provider = BkashPaymentProvider::new(
        "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    let sns_payload = json!({
        "Type": "Notification",
        "MessageId": "msg-5678",
        "TopicArn": "arn:aws:sns:ap-southeast-1:1234567890:bkash-notifications",
        "Message": "{\"transactionStatus\": \"Completed\"}",
        "Timestamp": "2026-08-21T12:00:00.000Z",
        "Signature": "valid_signature",
        "SigningCertURL": "https://attacker.evil.com/fake-sns-cert.pem"
    });

    let err = provider
        .verify_webhook(sns_payload.to_string().as_bytes(), None)
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
