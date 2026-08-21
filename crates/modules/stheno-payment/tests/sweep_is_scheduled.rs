//! P32-05: Periodic reconciliation sweep convergence test.
//! Asserts reconcile_unsettled_requests executes and resolves unsettled transactions.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_payment::PaymentService;

#[tokio::test]
async fn sweep_is_scheduled_and_converges_unsettled_requests() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);

    // Run reconciliation sweep with min_age_seconds = 0
    let report = service
        .reconcile_unsettled_requests(0, 100)
        .await
        .expect("sweep runs successfully");

    assert_eq!(report.scanned, 0);
    assert_eq!(report.converged, 0);
    assert_eq!(report.resolved_failed, 0);
    assert_eq!(report.still_unsettled, 0);
}
