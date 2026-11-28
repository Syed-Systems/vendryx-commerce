//! P22-15: SSLCommerz refund and capability declaration.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{PaymentProvider, RefundInput};
use stheno_payment_sslcommerz::SslcommerzPaymentProvider;

#[tokio::test]
async fn sslcommerz_refund_exercises_partial_refund_honestly() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/validator/api/merchantTransIDvalidationAPI.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "APIConnect": "DONE",
            "status": "success",
            "refund_ref_id": "REF_SSL_7788",
            "trans_id": "BANK_TX_5544",
            "errorreason": ""
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        Some(&mock_server.uri()),
        true,
    );

    assert!(provider.capabilities().partial_refund);

    let refund_out = provider
        .refund(RefundInput {
            payment_id: "tx_order_101".to_string(),
            amount: dec!(500.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: Some("Customer returned item".to_string()),
            data: Some(json!({ "bank_tran_id": "BANK_TX_5544" })),
        })
        .await
        .expect("refund succeeds");

    assert_eq!(refund_out.data["status"], "success");
    assert_eq!(refund_out.data["refund_ref_id"], "REF_SSL_7788");
}

#[tokio::test]
async fn sslcommerz_refund_refuses_invalid_amount() {
    let provider = SslcommerzPaymentProvider::new("test_store_id", "test_store_passwd", None, true);

    let err = provider
        .refund(RefundInput {
            payment_id: "tx_123".to_string(),
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

#[tokio::test]
async fn sslcommerz_refund_handles_gateway_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/validator/api/merchantTransIDvalidationAPI.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "APIConnect": "DONE",
            "status": "failed",
            "errorreason": "Refund already initiated for this transaction"
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        Some(&mock_server.uri()),
        true,
    );

    let err = provider
        .refund(RefundInput {
            payment_id: "tx_order_101".to_string(),
            amount: dec!(500.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: None,
            data: Some(json!({ "bank_tran_id": "BANK_TX_5544" })),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
