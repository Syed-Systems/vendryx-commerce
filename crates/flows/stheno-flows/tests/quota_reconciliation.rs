use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::QuotaReconciliationJob;

#[tokio::test]
async fn test_quota_reconciliation_detects_drift_and_reconciles() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let job = QuotaReconciliationJob::new(counter.clone());

    let mut conn = pool.get().await.expect("conn");

    // 1. Set recorded counter to 15
    counter
        .set_usage(&mut conn, "products", 15)
        .await
        .expect("set 15");

    // 2. Simulate actual count in database is 12 (drift of -3, e.g. manual delete or crash)
    let actual_count = 12;

    // 3. Run reconciliation job
    let drift_detected = job
        .reconcile_resource("products", actual_count)
        .await
        .expect("reconcile");
    assert!(drift_detected, "Reconciliation must detect drift");

    // 4. Check that counter was corrected to 12
    let corrected = counter
        .get_usage(&mut conn, "products")
        .await
        .expect("get usage");
    assert_eq!(corrected, 12, "Counter must be corrected to actual count");

    // 5. Running reconciliation again with same actual count detects no drift
    let drift_again = job
        .reconcile_resource("products", actual_count)
        .await
        .expect("reconcile again");
    assert!(!drift_again, "No drift should be detected when in sync");

    db.drop().await;
}
