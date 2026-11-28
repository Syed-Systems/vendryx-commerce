use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::build_create_product_quota_saga;
use stheno_product::ProductService;

#[tokio::test]
async fn test_quota_100_parallel_creates_against_limit_of_10() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let product_service = Arc::new(ProductService::new(pool.clone()));

    let limit = 10;
    let mut handles = Vec::new();

    for i in 0..100 {
        let ps = product_service.clone();
        let qc = counter.clone();
        handles.push(tokio::spawn(async move {
            let saga = build_create_product_quota_saga(ps, qc, Some(limit));
            saga.execute(
                &format!("tx_conc_{i}"),
                "create-product",
                json!({
                    "title": format!("Parallel Prod {i}"),
                    "handle": format!("parallel-prod-{i}"),
                }),
            )
            .await
        }));
    }

    let mut successes = 0;
    let mut failures = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    assert_eq!(successes, 10, "Exactly 10 parallel creates must succeed");
    assert_eq!(failures, 90, "Exactly 90 parallel creates must be rejected");

    let products = product_service.list_products().await.expect("list");
    assert_eq!(products.len(), 10);

    db.drop().await;
}
