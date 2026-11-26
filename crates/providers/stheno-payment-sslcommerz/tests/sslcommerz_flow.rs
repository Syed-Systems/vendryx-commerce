use rust_decimal_macros::dec;
use serde_json::json;
use std::collections::BTreeMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment::{
    CaptureInput, CredentialMap, InitiateInput, PaymentProvider, PaymentProviderFactory,
    PaymentSessionStatus, RefundInput, WebhookAction,
};
use stheno_payment_sslcommerz::{SslcommerzPaymentProvider, SslcommerzPaymentProviderFactory};

fn test_credentials(server_url: &str) -> CredentialMap {
    let mut map = BTreeMap::new();
    map.insert("store_id".to_string(), json!("test_store_id"));
    map.insert("store_passwd".to_string(), json!("test_store_passwd"));
    map.insert("is_sandbox".to_string(), json!(true));
    map.insert("base_url".to_string(), json!(server_url));
    map
}

#[tokio::test]
async fn sslcommerz_factory_builds_provider_with_schema_validation() {
    let factory = SslcommerzPaymentProviderFactory;
    assert_eq!(factory.provider_id(), "pp_sslcommerz_sslcommerz");
    assert!(!factory.credential_schema().is_empty());

    let creds = test_credentials("http://127.0.0.1:8080");
    let provider = factory.build(&creds).expect("build succeeds");
    assert_eq!(provider.id(), "pp_sslcommerz_sslcommerz");
    assert!(provider.capabilities().partial_capture);
    assert!(provider.capabilities().partial_refund);
}

#[tokio::test]
async fn sslcommerz_initiate_generates_redirect_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/gwprocess/v4/api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "SUCCESS",
            "failedreason": "",
            "sessionkey": "sess_123456",
            "gw": { "visa": "1", "master": "1", "bkash": "1" },
            "GatewayPageURL": format!("{}/hosted_payment_page?session=sess_123456", mock_server.uri()),
            "storeBanner": "https://example.com/banner.png",
            "storeLogo": "https://example.com/logo.png"
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
            amount: dec!(1500.00),
            currency_code: "BDT".to_string(),
            context: Some(json!({
                "session_id": "sess_cart_1",
                "customer_name": "Rahim Ahmed",
                "customer_email": "rahim@example.com",
                "customer_phone": "01711111111"
            })),
            metadata: None,
        })
        .await
        .expect("initiate succeeds");

    assert_eq!(output.id, "sess_123456");
    assert_eq!(output.status, PaymentSessionStatus::Pending);
    assert_eq!(
        output.data["redirect_url"],
        format!(
            "{}/hosted_payment_page?session=sess_123456",
            mock_server.uri()
        )
    );
}

#[tokio::test]
async fn sslcommerz_capture_validates_order_server_side() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/validator/api/validationserverAPI.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "VALID",
            "tran_id": "cart_123",
            "val_id": "val_999",
            "amount": "1500.00",
            "store_amount": "1462.50",
            "currency": "BDT",
            "bank_tran_id": "bank_tx_1234"
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
            payment_id: "cart_123".to_string(),
            amount: dec!(1500.00),
            currency_code: "BDT".to_string(),
            data: Some(json!({ "val_id": "val_999" })),
        })
        .await
        .expect("capture succeeds");

    assert_eq!(capture_out.data["status"], "VALID");
    assert_eq!(capture_out.data["bank_tran_id"], "bank_tx_1234");
}

#[tokio::test]
async fn sslcommerz_refund_processes_merchant_refund() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/validator/api/merchantTransIDvalidationAPI.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "refund_ref_id": "ref_98765",
            "trans_id": "cart_123",
            "amount": "500.00"
        })))
        .mount(&mock_server)
        .await;

    let provider = SslcommerzPaymentProvider::new(
        "test_store_id",
        "test_store_passwd",
        Some(&mock_server.uri()),
        true,
    );

    let refund_out = provider
        .refund(RefundInput {
            payment_id: "cart_123".to_string(),
            amount: dec!(500.00),
            currency_code: "BDT".to_string(),
            reason: None,
            note: Some("Customer return".to_string()),
            data: Some(json!({ "bank_tran_id": "bank_tx_1234" })),
        })
        .await
        .expect("refund succeeds");

    assert_eq!(refund_out.data["status"], "success");
    assert_eq!(refund_out.data["refund_ref_id"], "ref_98765");
}

#[tokio::test]
async fn sslcommerz_webhook_ipn_parsing_and_mapping() {
    let provider = SslcommerzPaymentProvider::new("test_store_id", "test_store_passwd", None, true);

    let ipn_payload =
        "tran_id=order_abc&val_id=val_123&status=VALID&amount=1200.00&currency_type=BDT";
    let verified = provider
        .verify_webhook(ipn_payload.as_bytes(), None)
        .expect("ipn verified");

    assert_eq!(verified.event_type, "VALID");
    assert_eq!(verified.payload["tran_id"], "order_abc");

    let action = provider.map_webhook(&verified).expect("mapped action");
    assert_eq!(action, WebhookAction::Captured);
}
