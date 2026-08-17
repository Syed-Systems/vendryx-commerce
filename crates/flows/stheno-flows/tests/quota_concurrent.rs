use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::CheckQuotaStep;
use stheno_workflow::SagaCoordinator;

#[tokio::test]
async fn test_quota_concurrency_100_creates_vs_limit_10() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let counter = QuotaCounter::new(pool.clone());

    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for i in 0..100 {
        let qc = counter.clone();
        let sc = success_count.clone();
        let fc = failure_count.clone();

        handles.push(tokio::spawn(async move {
            let mut saga = SagaCoordinator::new();
            saga.add_step(Arc::new(CheckQuotaStep::new(qc, "products", Some(10))));

            let res = saga
                .execute(
                    &format!("tx_{}", i),
                    "quota-flow",
                    json!({ "product": "test" }),
                )
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

    // Exactly 10 succeed, 90 fail
    assert_eq!(success_count.load(Ordering::SeqCst), 10);
    assert_eq!(failure_count.load(Ordering::SeqCst), 90);

    let mut conn = pool.get().await.expect("conn");
    assert_eq!(
        counter
            .get_usage(&mut conn, "products")
            .await
            .expect("usage"),
        10
    );

    db.drop().await;
}
