//! P22-14: [BP] SSLCommerz IPN validation and transaction-query convergence.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{CaptureInput, PaymentProvider, WebhookAction};
use stheno_payment_sslcommerz::SslcommerzPaymentProvider;

#[tokio::test]
async fn sslcommerz_ipn_webhook_verification_and_mapping() {
    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        None,
        true,
    );

    let ipn_payload = json!({
        "status": "VALID",
        "tran_id": "tx_order_101",
        "val_id": "VAL_SSL_9988",
        "amount": "3500.00",
        "store_amount": "3412.50",
        "bank_tran_id": "BANK_TX_5544",
        "card_type": "VISA-CityBank"
    });

    let verified = provider
        .verify_webhook(ipn_payload.to_string().as_bytes(), None)
        .expect("verify succeeds");

    assert_eq!(verified.event_type, "VALID");
    assert_eq!(verified.provider_event_id, "VAL_SSL_9988");

    let action = provider.map_webhook(&verified).expect("map succeeds");
    assert_eq!(action, WebhookAction::Captured);
}

#[tokio::test]
async fn sslcommerz_ipn_validation_server_convergence() {
    let mock_server = MockServer::start().await;

    // validationserverAPI confirms valid status
    Mock::given(method("GET"))
        .and(path("/validator/api/validationserverAPI.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "VALID",
            "tran_id": "tx_order_101",
            "val_id": "VAL_SSL_9988",
            "amount": "3500.00",
            "bank_tran_id": "BANK_TX_5544"
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        Some(&mock_server.uri()),
        true,
    );

    let capture_out = provider
        .capture(CaptureInput {
            payment_id: "tx_order_101".to_string(),
            amount: dec!(3500.00),
            currency_code: "BDT".to_string(),
            data: Some(json!({ "val_id": "VAL_SSL_9988" })),
        })
        .await
        .expect("validation capture succeeds");

    assert_eq!(capture_out.data["status"], "VALID");
    assert_eq!(capture_out.data["bank_tran_id"], "BANK_TX_5544");
}

#[tokio::test]
async fn sslcommerz_transaction_query_convergence_when_val_id_absent() {
    let mock_server = MockServer::start().await;

    // If val_id is absent, query convergence falls back to merchantTransIDvalidationAPI
    Mock::given(method("GET"))
        .and(path("/validator/api/merchantTransIDvalidationAPI.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "VALID",
            "tran_id": "tx_order_converge",
            "amount": "1200.00",
            "currency": "BDT"
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        Some(&mock_server.uri()),
        true,
    );

    let capture_out = provider
        .capture(CaptureInput {
            payment_id: "tx_order_converge".to_string(),
            amount: dec!(1200.00),
            currency_code: "BDT".to_string(),
            data: None, // No val_id in data
        })
        .await
        .expect("capture converges via transaction query");

    assert_eq!(capture_out.data["status"], "VALID");
}
