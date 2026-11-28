//! P22-08: Nagad checkout initialise with its key-exchange/signing scheme.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{InitiateInput, PaymentProvider, PaymentSessionStatus};
use stheno_payment_nagad::NagadPaymentProvider;

#[tokio::test]
async fn nagad_initialise_executes_two_step_checkout_and_returns_redirect_url() {
    let mock_server = MockServer::start().await;

    // Step 1: Initialize
    Mock::given(method("POST"))
        .and(path("/check-out/initialize/TEST_MERCHANT_01/ord_nagad_101"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sensitiveData": "encrypted_init_data",
            "signature": "nagad_pub_sig",
            "paymentReferenceId": "NAGAD_REF_998877",
            "challenge": "challenge_token_abc"
        })))
        .mount(&mock_server)
        .await;

    // Step 2: Complete
    Mock::given(method("POST"))
        .and(path("/check-out/complete/NAGAD_REF_998877"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "Success",
            "callBackUrl": format!("{}/dfs/payment?paymentRefId=NAGAD_REF_998877", mock_server.uri()),
            "message": "Checkout URL generated"
        })))
        .mount(&mock_server)
        .await;

    let provider = NagadPaymentProvider::new(
        &mock_server.uri(),
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let output = provider
        .initiate(InitiateInput {
            amount: dec!(1850.00),
            currency_code: "BDT".to_string(),
            context: Some(json!({
                "session_id": "ord_nagad_101",
                "callback_url": "https://example.com/checkout/callback"
            })),
            metadata: None,
        })
        .await
        .expect("nagad initiate succeeds");

    assert_eq!(output.id, "NAGAD_REF_998877");
    assert_eq!(output.status, PaymentSessionStatus::Pending);
    let redirect = output.redirect_url();
    assert_eq!(
        redirect,
        Some(
            format!(
                "{}/dfs/payment?paymentRefId=NAGAD_REF_998877",
                mock_server.uri()
            )
            .as_str()
        )
    );
}

#[tokio::test]
async fn nagad_initialise_refuses_when_complete_fails() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/check-out/initialize/TEST_MERCHANT_01/ord_fail"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "paymentReferenceId": "NAGAD_REF_FAIL",
            "challenge": "chal_1"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/check-out/complete/NAGAD_REF_FAIL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "Failed",
            "message": "Invalid merchant credentials or signature"
        })))
        .mount(&mock_server)
        .await;

    let provider = NagadPaymentProvider::new(
        &mock_server.uri(),
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let err = provider
        .initiate(InitiateInput {
            amount: dec!(100.00),
            currency_code: "BDT".to_string(),
            context: Some(json!({ "session_id": "ord_fail" })),
            metadata: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
