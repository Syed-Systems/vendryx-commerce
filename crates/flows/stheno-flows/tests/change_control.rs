//! Order Change Control Test (P6-17).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderChangeControlWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderChangeInput, CreateOrderInput, OrderService,
};

#[tokio::test]
async fn test_order_change_cancel_and_decline_controls() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderChangeControlWorkflowEngine::new(order_service.clone());

    // 1. Create base order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Control".to_string()),
                last_name: Some("Tester".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("control@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Widget".to_string(),
                quantity: 1,
                unit_price: dec!(20.00),
                variant_id: Some("var_widget".to_string()),
                is_tax_inclusive: Some(false),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                ..Default::default()
            }]),
            shipping_methods: Some(vec![]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // 2. Test cancel on change 1
    let change1 = order_service
        .create_order_change(CreateOrderChangeInput {
            order_id: order.id.clone(),
            change_type: "edit".to_string(),
            requested_by: Some("customer".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    let canceled = engine.cancel_change(&change1.id).await.unwrap();
    assert_eq!(canceled.status, "canceled");
    assert!(canceled.canceled_at.is_some());

    // 3. Test decline on change 2
    let change2 = order_service
        .create_order_change(CreateOrderChangeInput {
            order_id: order.id.clone(),
            change_type: "return".to_string(),
            requested_by: Some("customer".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    let declined = engine
        .decline_change(&change2.id, Some("admin"), Some("Policy exceeded"))
        .await
        .unwrap();
    assert_eq!(declined.status, "declined");
    assert_eq!(declined.declined_by, Some("admin".to_string()));
    assert!(declined.declined_at.is_some());

    db.drop().await;
}
