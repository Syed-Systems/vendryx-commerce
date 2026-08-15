use serde_json::json;
use stheno_payment::PaymentService;
use stheno_types::Result;

#[tokio::test]
async fn test_outbound_idempotency_caching_and_payload_mismatch() {
    let service = PaymentService::new();

    let key = "idem_key_unique_123";
    let payload = json!({
        "amount": 5000,
        "currency": "usd"
    });

    // 1. Initial call -> executes closure
    let mut execution_count = 0;
    let res1: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_stripe_stripe",
            "capture",
            "payment",
            "pay_123",
            &payload,
            |_k| {
                execution_count += 1;
                async move {
                    Ok(json!({
                        "status": "succeeded",
                        "charge_id": "ch_mock_1"
                    }))
                }
            },
        )
        .await;

    assert!(res1.is_ok());
    assert_eq!(execution_count, 1);
    assert_eq!(res1.unwrap()["status"], "succeeded");

    // 2. Second call with SAME key and SAME payload -> returns cached response WITHOUT executing closure
    let res2: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_stripe_stripe",
            "capture",
            "payment",
            "pay_123",
            &payload,
            |_k| {
                execution_count += 1;
                async move {
                    Ok(json!({
                        "status": "should_not_run",
                    }))
                }
            },
        )
        .await;

    assert!(res2.is_ok());
    assert_eq!(
        execution_count, 1,
        "closure must not execute on cached replay"
    );
    assert_eq!(res2.unwrap()["status"], "succeeded");

    // 3. Third call with SAME key but DIFFERENT payload -> rejected with BadRequest payload mismatch (E4)
    let different_payload = json!({
        "amount": 9999,
        "currency": "usd"
    });

    let res3: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_stripe_stripe",
            "capture",
            "payment",
            "pay_123",
            &different_payload,
            |_k| async move { Ok(json!({})) },
        )
        .await;

    assert!(res3.is_err());
    match res3.unwrap_err() {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "idempotency_key_payload_mismatch");
        }
        _ => panic!("expected Invalid idempotency_key_payload_mismatch error"),
    }
}
