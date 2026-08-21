use rust_decimal_macros::dec;
use serde_json::json;
use std::collections::BTreeMap;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{
    CaptureInput, CredentialMap, InitiateInput, PaymentProvider, PaymentProviderFactory,
    PaymentSessionStatus, RefundInput, WebhookAction,
};
use stheno_payment_bkash::{BkashPaymentProvider, BkashPaymentProviderFactory};

fn test_credentials(server_url: &str) -> CredentialMap {
    let mut map = BTreeMap::new();
    map.insert("base_url".to_string(), json!(server_url));
    map.insert("app_key".to_string(), json!("test_app_key"));
    map.insert("app_secret".to_string(), json!("test_app_secret"));
    map.insert("username".to_string(), json!("test_username"));
    map.insert("password".to_string(), json!("test_password"));
    map
}

#[tokio::test]
async fn bkash_factory_builds_provider_with_schema_validation() {
    let factory = BkashPaymentProviderFactory;
    assert_eq!(factory.provider_id(), "pp_bkash_bkash");
    assert!(!factory.credential_schema().is_empty());

    let creds = test_credentials("http://127.0.0.1:8080");
    let provider = factory.build(&creds).expect("build succeeds");
    assert_eq!(provider.id(), "pp_bkash_bkash");
    assert!(provider.capabilities().partial_refund);
    assert!(provider.capabilities().cancel_after_authorize);
}

#[tokio::test]
async fn bkash_initiate_grants_token_and_generates_bkash_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .and(header("username", "test_username"))
        .and(header("password", "test_password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "id_token": "token_bkash_123456",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "refresh_123"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/payment/create"))
        .and(header("Authorization", "token_bkash_123456"))
        .and(header("X-App-Key", "test_app_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "paymentID": "TR0011abcd",
            "bkashURL": format!("{}/checkout?paymentID=TR0011abcd", mock_server.uri()),
            "callbackURL": "https://example.com/checkout/callback",
            "amount": "1200.00",
            "currency": "BDT",
            "intent": "authorization",
            "merchantInvoiceNumber": "cart_order_1"
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
            amount: dec!(1200.00),
            currency_code: "BDT".to_string(),
            context: Some(json!({
                "session_id": "cart_order_1",
                "payer_reference": "01711223344"
            })),
            metadata: None,
        })
        .await
        .expect("initiate succeeds");

    assert_eq!(output.id, "TR0011abcd");
    assert_eq!(output.status, PaymentSessionStatus::Pending);
    assert_eq!(
        output.data["redirect_url"],
        format!("{}/checkout?paymentID=TR0011abcd", mock_server.uri())
    );
}

#[tokio::test]
async fn bkash_capture_executes_payment() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_bkash_exec",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/execute/TR0011abcd"))
        .and(header("Authorization", "token_bkash_exec"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "paymentID": "TR0011abcd",
            "trxID": "TRX998877",
            "amount": "1200.00",
            "currency": "BDT",
            "transactionStatus": "Completed",
            "customerMsisdn": "01711223344"
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

    let capture_out = provider
        .capture(CaptureInput {
            payment_id: "TR0011abcd".to_string(),
            amount: dec!(1200.00),
            currency_code: "BDT".to_string(),
            data: None,
        })
        .await
        .expect("capture succeeds");

    assert_eq!(capture_out.data["trxID"], "TRX998877");
    assert_eq!(capture_out.data["transactionStatus"], "Completed");
}

#[tokio::test]
async fn bkash_refund_processes_payment_refund() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_bkash_refund",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/payment/refund"))
        .and(header("Authorization", "token_bkash_refund"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "refundTrxID": "RTRX12345",
            "transactionStatus": "Completed",
            "amount": "600.00",
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

    let refund_out = provider
        .refund(RefundInput {
            payment_id: "TR0011abcd".to_string(),
            amount: dec!(600.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: Some("Item returned".to_string()),
            data: Some(json!({ "trxID": "TRX998877" })),
        })
        .await
        .expect("refund succeeds");

    assert_eq!(refund_out.data["refundTrxID"], "RTRX12345");
}

#[tokio::test]
async fn bkash_webhook_sns_payload_mapping() {
    let provider = BkashPaymentProvider::new(
        "https://tokenized.sandbox.bka.sh/v1.2.0-beta",
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    let inner_msg = json!({
        "paymentID": "TR0011abcd",
        "trxID": "TRX998877",
        "transactionStatus": "Completed",
        "amount": "1200.00",
        "currency": "BDT"
    });

    let sns_payload = json!({
        "Type": "Notification",
        "MessageId": "msg-1234",
        "TopicArn": "arn:aws:sns:ap-southeast-1:1234567890:bkash-notifications",
        "Message": inner_msg.to_string(),
        "Timestamp": "2026-08-21T12:00:00.000Z"
    });

    let verified = provider
        .verify_webhook(sns_payload.to_string().as_bytes(), None)
        .expect("sns webhook verified");

    assert_eq!(verified.event_type, "Notification");

    let action = provider.map_webhook(&verified).expect("mapped action");
    assert_eq!(action, WebhookAction::Captured);
}
