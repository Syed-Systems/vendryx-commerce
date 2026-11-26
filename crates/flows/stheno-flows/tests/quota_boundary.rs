use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::build_create_product_quota_saga;
use stheno_product::ProductService;

#[tokio::test]
async fn test_quota_boundary_exact_rejection_at_limit() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let product_service = Arc::new(ProductService::new(pool.clone()));

    let limit = 10;
    let saga =
        build_create_product_quota_saga(product_service.clone(), counter.clone(), Some(limit));

    // Exactly 10 products must succeed
    for i in 1..=limit {
        let res = saga
            .execute(
                &format!("tx_{i}"),
                "create-product",
                json!({ "title": format!("Product {i}"), "handle": format!("prod-{i}") }),
            )
            .await;
        assert!(res.is_ok(), "Product {i} within limit must succeed");
    }

    // 11th product (limit + 1) must be rejected
    let res11 = saga
        .execute(
            "tx_11",
            "create-product",
            json!({ "title": "Product 11", "handle": "prod-11" }),
        )
        .await;
    assert!(res11.is_err(), "11th product must be rejected");

    let count = product_service.list_products().await.expect("list").len();
    assert_eq!(
        count, 10,
        "Total created products must equal exactly the limit"
    );

    db.drop().await;
}
