use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_wiremock_stripe_payment_mock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/payment_intents/pi_test123/capture"))
        .and(header("Idempotency-Key", "idem_key_777"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "pi_test123",
            "status": "succeeded",
            "amount_received": 5000
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/v1/payment_intents/pi_test123/capture",
        mock_server.uri()
    );

    let res = client
        .post(&url)
        .header("Idempotency-Key", "idem_key_777")
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["id"], "pi_test123");
    assert_eq!(body["status"], "succeeded");
}

#[tokio::test]
async fn test_wiremock_sendgrid_notification_mock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v3/mail/send"))
        .respond_with(ResponseTemplate::new(202))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/v3/mail/send", mock_server.uri());

    let res = client
        .post(&url)
        .json(&json!({
            "personalizations": [{ "to": [{ "email": "customer@example.com" }] }],
            "subject": "Order Confirmation",
            "content": [{ "type": "text/plain", "value": "Your order is confirmed." }]
        }))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(res.status(), 202);
}

#[tokio::test]
async fn test_wiremock_s3_upload_mock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/bucket/product-images/shirt.png"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/bucket/product-images/shirt.png", mock_server.uri());

    let res = client
        .put(&url)
        .body(vec![0x89, 0x50, 0x4E, 0x47]) // PNG header bytes
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(res.status(), 200);
}
