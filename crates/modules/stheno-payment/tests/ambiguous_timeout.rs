use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::PaymentService;
use stheno_payment_offline::OfflinePaymentProvider;
use stheno_types::Result;

#[tokio::test]
async fn test_ambiguous_timeout_recovery_with_retrieve_reconciliation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_offline_test")));
    service.register_provider(offline);

    let key = "idem_key_timeout_test";
    let payload = json!({
        "amount": 2500,
        "currency": "usd"
    });

    // Simulate first attempt: network timeout occurs, error returned
    let res1: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_offline_test",
            "authorize",
            "session",
            "sess_ambiguous_1",
            &payload,
            |_k| async move {
                // Simulate network error/timeout
                Err(stheno_types::Error::Internal(
                    "Connection timeout while calling PSP".to_string().into(),
                ))
            },
        )
        .await;

    assert!(res1.is_err());

    // Second attempt with the SAME key and payload:
    // Idempotency engine invokes provider.retrieve() to reconcile the ambiguous state.
    // Provider reports session was Authorized -> engine marks request as succeeded and returns response.
    let res2: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_offline_test",
            "authorize",
            "session",
            "sess_ambiguous_1",
            &payload,
            |_k| async move {
                panic!("should not call provider directly on ambiguous retry reconciliation");
            },
        )
        .await;

    assert!(
        res2.is_ok(),
        "ambiguous timeout must reconcile successfully via retrieve()"
    );
}

#[tokio::test]
async fn ambiguous_refund_is_blocked_without_a_provider_specific_reconciliation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let key = "idem_key_ambiguous_refund";
    let payload = json!({
        "payment_id": "pay_ambiguous_refund",
        "amount": "25.00",
        "currency_code": "USD"
    });

    let first: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_test",
            "refund",
            "payment",
            "pay_ambiguous_refund",
            &payload,
            |_provider_key| async move {
                Err(stheno_types::Error::Internal(
                    "provider response timed out".to_string().into(),
                ))
            },
        )
        .await;
    assert!(first.is_err());

    let retry: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_test",
            "refund",
            "payment",
            "pay_ambiguous_refund",
            &payload,
            |_provider_key| async move {
                panic!("unresolved refund must not be sent to the provider again")
            },
        )
        .await;
    assert!(matches!(
        retry,
        Err(stheno_types::Error::Conflict {
            code: "provider_request_unknown",
            ..
        })
    ));

    db.drop().await;
}

#[tokio::test]
async fn provider_specific_refund_reconciliation_settles_and_replays_the_ledger_result() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let key = "idem_key_reconciled_refund";
    let payload = json!({
        "payment_id": "pay_reconciled_refund",
        "amount": "25.00",
        "currency_code": "USD"
    });

    let first: Result<serde_json::Value> = service
        .idempotent_call_with_reconciliation(
            key,
            "pp_test",
            "refund",
            "payment",
            "pay_reconciled_refund",
            &payload,
            |_provider_key| async move {
                Err(stheno_types::Error::Internal(
                    "provider response timed out".to_string().into(),
                ))
            },
            |_provider_key| async move { Ok(None) },
        )
        .await;
    assert!(first.is_err());

    let recovered: Result<serde_json::Value> = service
        .idempotent_call_with_reconciliation(
            key,
            "pp_test",
            "refund",
            "payment",
            "pay_reconciled_refund",
            &payload,
            |_provider_key| async move {
                panic!("a proven refund must not be sent to the provider again")
            },
            |_provider_key| async move {
                Ok(Some(json!({
                    "data": {
                        "id": "re_provider_refund",
                        "status": "succeeded"
                    }
                })))
            },
        )
        .await;
    let expected = json!({
        "data": {
            "id": "re_provider_refund",
            "status": "succeeded"
        }
    });
    assert_eq!(recovered.expect("reconciled response"), expected);

    let replay: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_test",
            "refund",
            "payment",
            "pay_reconciled_refund",
            &payload,
            |_provider_key| async move {
                panic!("a settled refund must replay from the durable ledger")
            },
        )
        .await;
    assert_eq!(replay.expect("cached response"), expected);

    db.drop().await;
}
