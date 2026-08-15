use serde_json::json;
use std::sync::Arc;
use stheno_flows::{CheckQuotaStep, QuotaStore};
use stheno_workflow::SagaCoordinator;

#[tokio::test]
async fn test_quota_step_atomic_enforcement() {
    let quota_store = QuotaStore::new();
    quota_store.set_quota("tenant_abc", "max_products", 5, 0);

    let step = Arc::new(CheckQuotaStep::new(
        quota_store.clone(),
        "tenant_abc",
        "max_products",
    ));
    let mut saga = SagaCoordinator::new();
    saga.add_step(step);

    // 1. Consume 3 units -> succeeds
    let res1 = saga
        .execute("tx_1", "check-quota", json!({ "count": 3 }))
        .await;
    assert!(res1.is_ok());
    assert_eq!(quota_store.get_count("tenant_abc", "max_products"), 3);

    // 2. Consume 2 units -> succeeds (total 5 = limit)
    let res2 = saga
        .execute("tx_2", "check-quota", json!({ "count": 2 }))
        .await;
    assert!(res2.is_ok());
    assert_eq!(quota_store.get_count("tenant_abc", "max_products"), 5);

    // 3. Consume 1 unit -> fails with quota_exceeded
    let res3 = saga
        .execute("tx_3", "check-quota", json!({ "count": 1 }))
        .await;
    assert!(res3.is_err());
    assert!(res3.unwrap_err().to_string().contains("quota_exceeded"));
}
