use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_db::QuotaCounter;
use stheno_flows::build_create_product_quota_saga;
use stheno_product::{CreateProductInput, ProductService, UpdateProductInput};

#[tokio::test]
async fn test_grandfathering_412_products_on_limit_100_readable_editable_413th_rejected() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 10).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let product_service = Arc::new(ProductService::new(pool.clone()));

    // 1. Create a product under existing quota
    let p1 = product_service
        .create_product(CreateProductInput {
            title: "Original Product".to_string(),
            handle: Some("orig-prod".to_string()),
            subtitle: None,
            description: None,
            discountable: Some(true),
        })
        .await
        .expect("create product");

    // Set usage to 412 in Postgres entitlement_usage table
    let mut conn = pool.get().await.expect("conn");
    counter
        .set_usage(&mut conn, "products", 412)
        .await
        .expect("set usage 412");
    drop(conn);

    // Plan downgraded to Free with limit of 100
    let limit = 100;
    let saga =
        build_create_product_quota_saga(product_service.clone(), counter.clone(), Some(limit));

    // 2. Existing product is readable
    let fetched = product_service
        .get_product(&p1.id.to_string())
        .await
        .expect("must be readable");
    assert_eq!(fetched.title, "Original Product");

    // 3. Existing product is editable
    let updated = product_service
        .update_product(
            &p1.id.to_string(),
            UpdateProductInput {
                title: Some("Edited Original Product".to_string()),
                handle: None,
                subtitle: None,
                description: None,
                discountable: None,
                status: None,
                thumbnail: None,
            },
        )
        .await
        .expect("must be editable");
    assert_eq!(updated.title, "Edited Original Product");

    // 4. 413th product create is rejected
    let res413 = saga
        .execute(
            "tx_413",
            "create-product",
            json!({ "title": "New Product 413", "handle": "new-prod-413" }),
        )
        .await;
    assert!(
        res413.is_err(),
        "413th create must be rejected over limit of 100"
    );

    db.drop().await;
}
