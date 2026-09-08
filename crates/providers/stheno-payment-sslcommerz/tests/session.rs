//! P22-13: SSLCommerz hosted session creation returning redirect_url.

use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{InitiateInput, PaymentProvider, PaymentSessionStatus};
use stheno_payment_sslcommerz::SslcommerzPaymentProvider;

#[tokio::test]
async fn sslcommerz_session_creation_returns_redirect_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/gwprocess/v4/api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "SUCCESS",
            "failedreason": "",
            "sessionkey": "SSL_SESS_ABC_123",
            "GatewayPageURL": format!("{}/gw/checkout?session=SSL_SESS_ABC_123", mock_server.uri())
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        Some(&mock_server.uri()),
        true,
    );

    let output = provider
        .initiate(InitiateInput {
            amount: dec!(3500.00),
            currency_code: "BDT".to_string(),
            context: Some(json!({
                "session_id": "tx_order_101",
                "customer_name": "Rahim Uddin",
                "customer_email": "rahim@example.com",
                "customer_phone": "01711000000"
            })),
            metadata: None,
        })
        .await
        .expect("initiate succeeds");

    assert_eq!(output.id, "SSL_SESS_ABC_123");
    assert_eq!(output.status, PaymentSessionStatus::Pending);
    let redirect = output.redirect_url();
    assert_eq!(
        redirect,
        Some(format!("{}/gw/checkout?session=SSL_SESS_ABC_123", mock_server.uri()).as_str())
    );
}

#[tokio::test]
async fn sslcommerz_session_refuses_failure_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/gwprocess/v4/api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "FAILED",
            "failedreason": "Invalid store credentials"
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "bad_store",
        "bad_pass",
        Some(&mock_server.uri()),
        true,
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
