//! P22-09: Nagad complete + verification callback -> capture.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{CaptureInput, PaymentProvider};
use stheno_payment_nagad::NagadPaymentProvider;

#[tokio::test]
async fn nagad_capture_verifies_payment_against_dfs_gateway() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/verify/payment/NAGAD_REF_998877"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "Success",
            "statusCode": "000_0000",
            "merchantId": "TEST_MERCHANT_01",
            "orderId": "ord_nagad_101",
            "paymentRefId": "NAGAD_REF_998877",
            "amount": "1850.00",
            "clientMobileNo": "01700112233"
        })))
        .mount(&mock_server)
        .await;

    let provider = NagadPaymentProvider::new(
        &mock_server.uri(),
        "TEST_MERCHANT_01",
        "mock_priv_key",
        "mock_pub_key",
    );

    let capture_out = provider
        .capture(CaptureInput {
            payment_id: "NAGAD_REF_998877".to_string(),
            amount: dec!(1850.00),
            currency_code: "BDT".to_string(),
            data: None,
        })
        .await
        .expect("capture succeeds");

    assert_eq!(capture_out.data["status"], "Success");
    assert_eq!(capture_out.data["clientMobileNo"], "01700112233");
}

#[tokio::test]
async fn nagad_capture_refuses_aborted_payment() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/verify/payment/NAGAD_REF_ABORT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "Aborted",
            "statusCode": "000_1001",
            "message": "User cancelled on Nagad PIN screen"
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
        .capture(CaptureInput {
            payment_id: "NAGAD_REF_ABORT".to_string(),
            amount: dec!(1850.00),
            currency_code: "BDT".to_string(),
            data: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
