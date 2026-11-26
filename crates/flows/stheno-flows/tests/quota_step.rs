use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::CheckQuotaStep;
use stheno_workflow::SagaCoordinator;

#[tokio::test]
async fn test_quota_step_atomic_enforcement() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());

    let step = Arc::new(CheckQuotaStep::new(counter.clone(), "products", Some(5)));
    let mut saga = SagaCoordinator::new();
    saga.add_step(step);

    // 1. Consume 3 units
    for i in 1..=3 {
        let res = saga
            .execute(&format!("tx_{i}"), "check-quota", json!({ "count": 1 }))
            .await;
        assert!(res.is_ok());
    }

    // Reads and writes go through the scoped helpers: a raw `pool.get()` has no
    // tenant, so under RLS it would read zero rather than what the saga just wrote.
    assert_eq!(counter.usage_scoped("products").await.expect("usage"), 3);

    // 2. Consume 2 more units (total 5 = limit)
    for i in 4..=5 {
        let res = saga
            .execute(&format!("tx_{i}"), "check-quota", json!({ "count": 1 }))
            .await;
        assert!(res.is_ok());
    }
    assert_eq!(counter.usage_scoped("products").await.expect("usage"), 5);

    // 3. Consume 1 unit -> fails with not_allowed / limit reached
    let res6 = saga
        .execute("tx_6", "check-quota", json!({ "count": 1 }))
        .await;
    assert!(res6.is_err());
    assert!(res6
        .unwrap_err()
        .to_string()
        .contains("Product limit reached (5)"));

    db.drop().await;
}
