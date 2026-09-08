//! P22-11: Nagad refund and capability declaration.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{PaymentProvider, RefundInput};
use stheno_payment_nagad::NagadPaymentProvider;

#[tokio::test]
async fn nagad_refund_declares_and_enforces_no_partial_refund() {
    let provider = NagadPaymentProvider::new(
        "https://sandbox.nagad.com.bd/remote-payment-gateway-1.0/api/dfs",
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    // Assert honesty: Nagad declares partial_refund = false
    assert!(!provider.capabilities().partial_refund);

    // Attempting a partial refund MUST be refused
    let err = provider
        .refund(RefundInput {
            payment_id: "NAGAD_REF_998877".to_string(),
            amount: dec!(500.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: None,
            data: Some(json!({ "original_amount": "1850.00" })),
        })
        .await
        .unwrap_err();

    match err {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "partial_refund_not_supported");
        }
        other => panic!("expected partial_refund_not_supported, got: {other:?}"),
    }
}

#[tokio::test]
async fn nagad_refund_full_refund_succeeds() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/refund/payment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "Success",
            "refundRefId": "NAGAD_REFUND_5544",
            "message": "Full refund successful"
        })))
        .mount(&mock_server)
        .await;

    let provider = NagadPaymentProvider::new(
        &mock_server.uri(),
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let refund_out = provider
        .refund(RefundInput {
            payment_id: "NAGAD_REF_998877".to_string(),
            amount: dec!(1850.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: Some("Full order cancellation".to_string()),
            data: Some(json!({ "original_amount": "1850.00" })),
        })
        .await
        .expect("full refund succeeds");

    assert_eq!(refund_out.data["status"], "Success");
    assert_eq!(refund_out.data["refundRefId"], "NAGAD_REFUND_5544");
}

#[tokio::test]
async fn nagad_refund_refuses_invalid_amount() {
    let provider = NagadPaymentProvider::new(
        "https://sandbox.nagad.com.bd/remote-payment-gateway-1.0/api/dfs",
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let err = provider
        .refund(RefundInput {
            payment_id: "NAGAD_REF_1".to_string(),
            amount: dec!(0.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: None,
            data: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
