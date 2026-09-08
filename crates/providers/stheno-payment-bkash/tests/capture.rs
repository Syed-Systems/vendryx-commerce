//! P22-04: bKash authorize/capture via execute + query-payment convergence.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{CaptureInput, PaymentProvider};
use stheno_payment_bkash::BkashPaymentProvider;

#[tokio::test]
async fn bkash_capture_normal_execute_succeeds() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_exec_1",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/execute/PAY_BKASH_100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "paymentID": "PAY_BKASH_100",
            "trxID": "TRX_NORMAL_1",
            "amount": "1500.00",
            "currency": "BDT",
            "transactionStatus": "Completed"
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
        .capture(CaptureInput {
            payment_id: "PAY_BKASH_100".to_string(),
            amount: dec!(1500.00),
            currency_code: "BDT".to_string(),
            data: None,
        })
        .await
        .expect("capture succeeds");

    assert_eq!(output.data["trxID"], "TRX_NORMAL_1");
    assert_eq!(output.data["transactionStatus"], "Completed");
}

#[tokio::test]
async fn bkash_capture_converges_via_query_when_execute_drops() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_exec_2",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // Simulate execute failure (e.g. gateway error, 500 internal server error or network issue)
    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/execute/PAY_BKASH_DROP"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    // Convergence query proves the payment completed server-side despite execute failure
    Mock::given(method("GET"))
        .and(path("/tokenized/checkout/payment/query/PAY_BKASH_DROP"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "paymentID": "PAY_BKASH_DROP",
            "trxID": "TRX_CONVERGED_99",
            "amount": "1500.00",
            "currency": "BDT",
            "transactionStatus": "Completed"
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
        .capture(CaptureInput {
            payment_id: "PAY_BKASH_DROP".to_string(),
            amount: dec!(1500.00),
            currency_code: "BDT".to_string(),
            data: None,
        })
        .await
        .expect("capture converges via query and succeeds");

    assert_eq!(output.data["trxID"], "TRX_CONVERGED_99");
    assert_eq!(output.data["transactionStatus"], "Completed");
}

#[tokio::test]
async fn bkash_capture_refuses_when_query_confirms_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_exec_3",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/execute/PAY_BKASH_FAIL"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/tokenized/checkout/payment/query/PAY_BKASH_FAIL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "paymentID": "PAY_BKASH_FAIL",
            "transactionStatus": "Failed"
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
        .capture(CaptureInput {
            payment_id: "PAY_BKASH_FAIL".to_string(),
            amount: dec!(1500.00),
            currency_code: "BDT".to_string(),
            data: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
