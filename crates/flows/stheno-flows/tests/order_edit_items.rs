//! Order Edit Items Integration Test (P6-01).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderEditWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_order_edit_begin_add_update_remove_items() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderEditWorkflowEngine::new(order_service.clone(), None);

    // 1. Create base order with 1 item
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("test@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Original Item".to_string(),
                quantity: 2,
                unit_price: dec!(20.00),
                variant_id: Some("var_orig".to_string()),
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

    let orig_item_id = order.items[0].id.clone();
    assert_eq!(order.version, 1);
    assert_eq!(order.items.len(), 1);

    // 2. Begin order edit
    let edit = engine
        .begin_order_edit(&order.id, Some("admin_user"))
        .await
        .unwrap();
    assert_eq!(edit.status, "pending");

    // 3. Add a new item
    engine
        .add_item(&edit.id, "New Added Item", 1, dec!(35.00), Some("var_new"))
        .await
        .unwrap();

    // 4. Update the original item quantity
    engine
        .update_item(&edit.id, &orig_item_id, 3, Some(dec!(20.00)))
        .await
        .unwrap();

    // 5. Confirm edit
    let updated_order = engine
        .confirm_edit(&edit.id, Some("admin_user"))
        .await
        .unwrap();

    // 6. Verify version bump and updated items
    assert_eq!(updated_order.version, 2);
    assert_eq!(updated_order.items.len(), 2);
    let updated_orig = updated_order
        .items
        .iter()
        .find(|i| i.id == orig_item_id)
        .unwrap();
    assert_eq!(updated_orig.quantity, 3);

    // 7. Begin another edit to remove an item
    let edit2 = engine
        .begin_order_edit(&order.id, Some("admin_user"))
        .await
        .unwrap();
    engine.remove_item(&edit2.id, &orig_item_id).await.unwrap();
    let final_order = engine
        .confirm_edit(&edit2.id, Some("admin_user"))
        .await
        .unwrap();

    assert_eq!(final_order.version, 3);
    assert_eq!(final_order.items.len(), 1);
    assert_eq!(final_order.items[0].title, "New Added Item");

    db.drop().await;
}
