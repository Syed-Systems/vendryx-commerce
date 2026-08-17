use std::sync::Arc;
use stheno_cart::{CartService, CreateCartInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::platform::RetentionJobWorkflow;

#[tokio::test]
async fn test_retention_job_cycle_and_staleness_tracking() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let cart_service = Arc::new(CartService::new(pool));
    let workflow = RetentionJobWorkflow::new(cart_service.clone(), 30);

    // 1. Create a guest cart
    let _guest_cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: None,
            sales_channel_id: Some("sc_web".to_string()),
            email: None,
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: None,
            metadata: None,
        })
        .await
        .unwrap();

    // 2. Run retention cycle
    let result = workflow.run_retention_cycle().await.unwrap();

    assert_eq!(result.stores_checked, 5);
    assert!(result.staleness_alerts.is_empty());
}
