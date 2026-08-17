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
