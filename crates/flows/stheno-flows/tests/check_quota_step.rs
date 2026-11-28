use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::CheckQuotaStep;
use stheno_workflow::SagaCoordinator;

#[tokio::test]
async fn test_check_quota_step_workflow_execution() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());

    let step = Arc::new(CheckQuotaStep::new(counter.clone(), "products", Some(5)));
    let mut saga = SagaCoordinator::new();
    saga.add_step(step);

    // 1. Consume 1 product
    let res1 = saga
        .execute("tx_1", "check-quota", json!({ "title": "Prod 1" }))
        .await;
    assert!(res1.is_ok());

    // 2. Consume 4 more products
    for i in 2..=5 {
        let res = saga
            .execute(
                &format!("tx_{i}"),
                "check-quota",
                json!({ "title": format!("Prod {i}") }),
            )
            .await;
        assert!(res.is_ok());
    }

    // 3. 6th product must fail with structured over-quota error
    let res6 = saga
        .execute("tx_6", "check-quota", json!({ "title": "Prod 6" }))
        .await;
    assert!(res6.is_err());
    let err_str = res6.unwrap_err().to_string();
    assert!(err_str.contains("Product limit reached (5)"));
    assert!(err_str.contains("Upgrade to Pro"));

    db.drop().await;
}
