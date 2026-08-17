use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::build_create_product_quota_saga;
use stheno_product::ProductService;

#[tokio::test]
async fn test_product_creation_wired_with_check_quota_step() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let product_service = Arc::new(ProductService::new(pool.clone()));

    let saga = build_create_product_quota_saga(product_service.clone(), counter.clone(), Some(2));

    // 1. Create 1st product -> OK
    let res1 = saga
        .execute(
            "tx_p1",
            "create-product",
            json!({ "title": "Sneakers", "handle": "sneakers" }),
        )
        .await;
    assert!(res1.is_ok());

    // 2. Create 2nd product -> OK
    let res2 = saga
        .execute(
            "tx_p2",
            "create-product",
            json!({ "title": "Hoodie", "handle": "hoodie" }),
        )
        .await;
    assert!(res2.is_ok());

    // 3. Create 3rd product -> Exceeds limit (2)
    let res3 = saga
        .execute(
            "tx_p3",
            "create-product",
            json!({ "title": "Cap", "handle": "cap" }),
        )
        .await;
    assert!(res3.is_err());
    let err = res3.unwrap_err().to_string();
    assert!(err.contains("Product limit reached (2)"));

    // Verify only 2 products exist in product_service
    let products = product_service.list_products().await.expect("list");
    assert_eq!(products.len(), 2);

    db.drop().await;
}
