//! P22-03: bKash initiate -> create payment, returns redirect_url.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{InitiateInput, PaymentProvider, PaymentSessionStatus};
use stheno_payment_bkash::BkashPaymentProvider;

#[tokio::test]
async fn bkash_initiate_creates_payment_and_returns_redirect_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "id_token": "token_init_123",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/payment/create"))
        .and(header("Authorization", "token_init_123"))
        .and(header("X-App-Key", "test_app_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "paymentID": "BKASH_PAY_9988",
            "bkashURL": format!("{}/checkout?paymentID=BKASH_PAY_9988", mock_server.uri()),
            "callbackURL": "https://example.com/checkout/callback",
            "amount": "2500.00",
            "currency": "BDT",
            "intent": "authorization",
            "merchantInvoiceNumber": "inv_test_01"
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

    let output = provider
        .initiate(InitiateInput {
            amount: dec!(2500.00),
            currency_code: "BDT".to_string(),
            context: Some(json!({
                "session_id": "inv_test_01",
                "payer_reference": "01811223344"
            })),
            metadata: None,
        })
        .await
        .expect("initiate succeeds");

    assert_eq!(output.id, "BKASH_PAY_9988");
    assert_eq!(output.status, PaymentSessionStatus::Pending);
    let redirect = output.redirect_url();
    assert_eq!(
        redirect,
        Some(format!("{}/checkout?paymentID=BKASH_PAY_9988", mock_server.uri()).as_str())
    );
}

#[tokio::test]
async fn bkash_initiate_refuses_rejected_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_init_fail",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/payment/create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "2006",
            "statusMessage": "Duplicate Merchant Invoice Number"
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

    let err = provider
        .initiate(InitiateInput {
            amount: dec!(100.00),
            currency_code: "BDT".to_string(),
            context: None,
            metadata: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
