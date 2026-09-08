//! P22-06: bKash refund; declare partial_refund truthfully and refuse what is not supported.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{PaymentProvider, RefundInput};
use stheno_payment_bkash::BkashPaymentProvider;

#[tokio::test]
async fn bkash_refund_exercises_partial_refund_honestly() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_ref_1",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/payment/refund"))
        .and(header("Authorization", "token_ref_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "refundTrxID": "REF_TRX_8877",
            "transactionStatus": "Completed",
            "amount": "400.00",
            "currency": "BDT"
        })))
        .mount(&mock_server)
        .await;

    let provider = BkashPaymentProvider::new(
        &mock_server.uri(),
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    // Assert honesty: bKash declares partial_refund = true
    assert!(provider.capabilities().partial_refund);

    // Execute partial refund of 400.00 from an initial 1000.00
    let refund_out = provider
        .refund(RefundInput {
            payment_id: "PAY_BKASH_ORIG".to_string(),
            amount: dec!(400.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: Some("Partial item cancellation".to_string()),
            data: Some(json!({ "trxID": "ORIG_TRX_123" })),
        })
        .await
        .expect("partial refund succeeds");

    assert_eq!(refund_out.data["refundTrxID"], "REF_TRX_8877");
    assert_eq!(refund_out.data["transactionStatus"], "Completed");
}

#[tokio::test]
async fn bkash_refund_refuses_missing_trx_id() {
    let provider = BkashPaymentProvider::new(
        "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    let err = provider
        .refund(RefundInput {
            payment_id: "PAY_123".to_string(),
            amount: dec!(100.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: None,
            data: None, // Missing trxID
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}

#[tokio::test]
async fn bkash_refund_refuses_invalid_amount() {
    let provider = BkashPaymentProvider::new(
        "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    let err = provider
        .refund(RefundInput {
            payment_id: "PAY_123".to_string(),
            amount: dec!(0.00), // Zero amount is invalid
            currency_code: "BDT".to_string(),
            reason: None,
            note: None,
            data: Some(json!({ "trxID": "TRX_123" })),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
