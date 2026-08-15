use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_flows::{CheckQuotaStep, QuotaStore};
use stheno_workflow::SagaCoordinator;

#[tokio::test]
async fn test_quota_concurrency_100_creates_vs_limit_10() {
    let quota_store = QuotaStore::new();
    quota_store.set_quota("tenant_xyz", "max_products", 10, 0);

    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for i in 0..100 {
        let qs = quota_store.clone();
        let sc = success_count.clone();
        let fc = failure_count.clone();

        handles.push(tokio::spawn(async move {
            let mut saga = SagaCoordinator::new();
            saga.add_step(Arc::new(CheckQuotaStep::new(
                qs,
                "tenant_xyz",
                "max_products",
            )));

            let res = saga
                .execute(&format!("tx_{}", i), "quota-flow", json!({ "count": 1 }))
                .await;

            match res {
                Ok(_) => {
                    sc.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    fc.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    // Critical assertion for Task P2-21c: Exactly 10 succeed, 90 fail
    assert_eq!(success_count.load(Ordering::SeqCst), 10);
    assert_eq!(failure_count.load(Ordering::SeqCst), 90);
    assert_eq!(quota_store.get_count("tenant_xyz", "max_products"), 10);
}
