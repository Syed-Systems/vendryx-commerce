//! P21-08: [BP] Provider conformance suite: session lifecycle, idempotent capture,
//! webhook replay, and refund paths (PHASE3_TASKS.md P21-08).

use rust_decimal_macros::dec;
use serde_json::json;
use stheno_payment::{
    AuthorizeInput, CancelInput, CaptureInput, DeleteInput, InitiateInput, PaymentProvider,
    PaymentSessionStatus, RefundInput, UpdateInput,
};
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn conformance_session_lifecycle() {
    let provider = OfflinePaymentProvider::new(None);

    // 1. Initiate
    let init_out = provider
        .initiate(InitiateInput {
            amount: dec!(150.00),
            currency_code: "USD".to_string(),
            context: Some(json!({"order_id": "ord_conf_1"})),
            metadata: None,
        })
        .await
        .expect("initiate succeeds");

    assert!(!init_out.id.is_empty(), "session id must not be empty");

    // 2. Update
    let update_out = provider
        .update(UpdateInput {
            session_id: init_out.id.clone(),
            amount: dec!(200.00),
            currency_code: "USD".to_string(),
            data: None,
        })
        .await
        .expect("update succeeds");

    assert_eq!(
        update_out.status,
        PaymentSessionStatus::PendingAuthorization
    );

    // 3. Delete
    provider
        .delete(DeleteInput {
            session_id: init_out.id,
            data: None,
        })
        .await
        .expect("delete succeeds");
}

#[tokio::test]
async fn conformance_idempotent_capture_and_refund_paths() {
    let provider = OfflinePaymentProvider::new(None);

    // Authorize
    let auth_out = provider
        .authorize(AuthorizeInput {
            session_id: "sess_conf_2".to_string(),
            amount: dec!(100.00),
            currency_code: "USD".to_string(),
            context: None,
            data: None,
        })
        .await
        .expect("authorize succeeds");

    assert_eq!(auth_out.status, PaymentSessionStatus::Authorized);

    // First Capture
    let cap_out_1 = provider
        .capture(CaptureInput {
            payment_id: "sess_conf_2".to_string(),
            amount: dec!(100.00),
            currency_code: "USD".to_string(),
            data: None,
        })
        .await
        .expect("first capture succeeds");

    assert_eq!(cap_out_1.data["captured_amount"], json!(dec!(100.00)));

    // Idempotent second capture
    let cap_out_2 = provider
        .capture(CaptureInput {
            payment_id: "sess_conf_2".to_string(),
            amount: dec!(100.00),
            currency_code: "USD".to_string(),
            data: None,
        })
        .await
        .expect("second capture succeeds without error");

    assert_eq!(cap_out_2.data["captured_amount"], json!(dec!(100.00)));

    // Full Refund
    let ref_out = provider
        .refund(RefundInput {
            payment_id: "sess_conf_2".to_string(),
            amount: dec!(100.00),
            currency_code: "USD".to_string(),
            reason: Some("customer_request".to_string()),
            note: None,
            data: None,
        })
        .await
        .expect("full refund succeeds");

    assert_eq!(ref_out.data["refunded_amount"], json!(dec!(100.00)));

    // Cancel flow
    let cancel_out = provider
        .cancel(CancelInput {
            payment_id: "sess_conf_3".to_string(),
            data: None,
        })
        .await
        .expect("cancel succeeds");

    assert!(cancel_out.data["canceled_at"].is_string());
}

#[tokio::test]
async fn conformance_webhook_verification_and_replay_contract() {
    let provider = OfflinePaymentProvider::new(None);

    let raw = b"{\"event\":\"manual_settlement\",\"id\":\"evt_offline_1\"}";
    let verified1 = provider
        .verify_webhook(raw, None)
        .expect("webhook verification succeeds");

    assert!(!verified1.provider_event_id.is_empty());

    // Re-verify identical payload
    let verified2 = provider
        .verify_webhook(raw, None)
        .expect("webhook re-verification succeeds");

    assert!(!verified2.provider_event_id.is_empty());
}

#[tokio::test]
async fn conformance_bkash_provider_contract() {
    use stheno_payment_bkash::BkashPaymentProvider;
    let provider = BkashPaymentProvider::default();

    assert_eq!(provider.id(), "pp_bkash_bkash");
    let caps = provider.capabilities();
    assert!(caps.redirect);
    assert!(caps.cancel_after_authorize);

    // Cancel contract
    let cancel = provider
        .cancel(CancelInput {
            payment_id: "PAY_TEST_1".to_string(),
            data: None,
        })
        .await
        .expect("cancel succeeds");
    assert_eq!(cancel.data["status"], "cancelled");

    // Webhook contract
    let sns = serde_json::json!({
        "Type": "Notification",
        "MessageId": "msg-bkash-conf",
        "Message": serde_json::json!({
            "paymentID": "TR0011abcd",
            "transactionStatus": "Completed",
            "amount": "500.00"
        }).to_string(),
        "SigningCertURL": "https://sns.ap-southeast-1.amazonaws.com/cert.pem",
        "Signature": "valid_sig"
    });
    let verified = provider
        .verify_webhook(sns.to_string().as_bytes(), None)
        .expect("verify succeeds");
    assert_eq!(verified.provider_event_id, "msg-bkash-conf");
    let action = provider.map_webhook(&verified).expect("map succeeds");
    assert_eq!(action, stheno_payment::WebhookAction::Captured);
}

#[tokio::test]
async fn conformance_sslcommerz_provider_contract() {
    use stheno_payment_sslcommerz::SslcommerzPaymentProvider;
    let provider = SslcommerzPaymentProvider::default();

    assert_eq!(provider.id(), "pp_sslcommerz_sslcommerz");
    let caps = provider.capabilities();
    assert!(caps.redirect);
    assert!(caps.cancel_after_authorize);

    // Cancel contract
    let cancel = provider
        .cancel(CancelInput {
            payment_id: "tx_test_1".to_string(),
            data: None,
        })
        .await
        .expect("cancel succeeds");
    assert_eq!(cancel.data["status"], "cancelled");

    // Webhook contract
    let ipn = serde_json::json!({
        "status": "VALID",
        "val_id": "VAL_SSL_CONF_1",
        "tran_id": "tx_test_1",
        "amount": "1500.00"
    });
    let verified = provider
        .verify_webhook(ipn.to_string().as_bytes(), None)
        .expect("verify succeeds");
    assert_eq!(verified.provider_event_id, "VAL_SSL_CONF_1");
    let action = provider.map_webhook(&verified).expect("map succeeds");
    assert_eq!(action, stheno_payment::WebhookAction::Captured);
}

#[tokio::test]
async fn conformance_nagad_provider_contract() {
    use stheno_payment_nagad::NagadPaymentProvider;
    let provider = NagadPaymentProvider::default();

    assert_eq!(provider.id(), "pp_nagad_nagad");
    let caps = provider.capabilities();
    assert!(caps.redirect);
    assert!(caps.cancel_after_authorize);

    // Cancel contract
    let cancel = provider
        .cancel(CancelInput {
            payment_id: "NAGAD_REF_1".to_string(),
            data: None,
        })
        .await
        .expect("cancel succeeds");
    assert_eq!(cancel.data["status"], "cancelled");

    // Webhook contract
    let callback = serde_json::json!({
        "payment_ref_id": "NAGAD_CONF_1",
        "order_id": "ord_101",
        "status": "Success",
        "signature": "valid_signature",
        "amount": "1200.00"
    });
    let verified = provider
        .verify_webhook(callback.to_string().as_bytes(), None)
        .expect("verify succeeds");
    assert_eq!(verified.provider_event_id, "NAGAD_CONF_1");
    let action = provider.map_webhook(&verified).expect("map succeeds");
    assert_eq!(action, stheno_payment::WebhookAction::Captured);
}
