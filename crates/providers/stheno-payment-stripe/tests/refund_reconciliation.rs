use rust_decimal_macros::dec;
use serde_json::json;
use stheno_payment::{PaymentProvider, RefundInput};
use stheno_payment_stripe::StripePaymentProvider;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn refund_input() -> RefundInput {
    RefundInput {
        payment_id: "pi_refund_123".to_string(),
        amount: dec!(25.00),
        currency_code: "USD".to_string(),
        reason: None,
        note: None,
        data: None,
    }
}

#[tokio::test]
async fn refund_uses_durable_key_for_stripe_and_persists_lookup_metadata() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/refunds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "re_refund_123",
            "payment_intent": "pi_refund_123",
            "amount": 2500,
            "currency": "usd",
            "status": "succeeded"
        })))
        .mount(&server)
        .await;
    let provider = StripePaymentProvider::new(
        "sk_test_fixture",
        "whsec_fixture",
        Some(&server.uri()),
        false,
    );

    provider
        .refund_with_idempotency_key(refund_input(), "refund:payment:0:25.00")
        .await
        .expect("refund request");

    let requests = server.received_requests().await.expect("captured request");
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(
        request
            .headers
            .get("Idempotency-Key")
            .and_then(|value| value.to_str().ok()),
        Some("refund:payment:0:25.00")
    );
    let body = String::from_utf8_lossy(&request.body);
    assert!(body.contains("metadata%5Bstheno_idempotency_key%5D=refund%3Apayment%3A0%3A25.00"));
}

#[tokio::test]
async fn refund_reconciliation_requires_exact_key_amount_currency_and_payment() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/refunds"))
        .and(query_param("payment_intent", "pi_refund_123"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {
                    "id": "re_wrong_key",
                    "payment_intent": "pi_refund_123",
                    "amount": 2500,
                    "currency": "usd",
                    "status": "succeeded",
                    "metadata": { "stheno_idempotency_key": "other-key" }
                },
                {
                    "id": "re_refund_123",
                    "payment_intent": "pi_refund_123",
                    "amount": 2500,
                    "currency": "usd",
                    "status": "succeeded",
                    "metadata": { "stheno_idempotency_key": "refund:payment:0:25.00" }
                }
            ]
        })))
        .mount(&server)
        .await;
    let provider = StripePaymentProvider::new(
        "sk_test_fixture",
        "whsec_fixture",
        Some(&server.uri()),
        false,
    );

    let reconciled = provider
        .reconcile_refund(&refund_input(), "refund:payment:0:25.00")
        .await
        .expect("lookup succeeds")
        .expect("matching refund is found");
    assert_eq!(reconciled.data["id"], "re_refund_123");

    let mut wrong_amount = refund_input();
    wrong_amount.amount = dec!(24.00);
    assert!(provider
        .reconcile_refund(&wrong_amount, "refund:payment:0:25.00")
        .await
        .expect("lookup succeeds")
        .is_none());
}
