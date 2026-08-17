//! Order Edit Cancel Test (P6-06).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderEditWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_order_edit_cancel_leaves_version_and_items_untouched() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderEditWorkflowEngine::new(order_service.clone(), None);

    // 1. Create order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Cancel".to_string()),
                last_name: Some("Test".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("cancel@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Keep Item".to_string(),
                quantity: 1,
                unit_price: dec!(40.00),
                variant_id: Some("var_keep".to_string()),
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

    // 2. Start order edit and make mutations (remove item, add new item)
    let edit = engine
        .begin_order_edit(&order.id, Some("admin"))
        .await
        .unwrap();

    engine.remove_item(&edit.id, &orig_item_id).await.unwrap();
    engine
        .add_item(&edit.id, "Temporary Item", 5, dec!(100.00), None)
        .await
        .unwrap();

    // 3. Cancel / discard the edit
    let canceled_change = engine.cancel_edit(&edit.id).await.unwrap();
    assert_eq!(canceled_change.status, "canceled");

    // 4. Verify order version, items, and summary remain unchanged
    let current_order = order_service.get_order(&order.id).await.unwrap();
    assert_eq!(current_order.version, 1);
    assert_eq!(current_order.items.len(), 1);
    assert_eq!(current_order.items[0].title, "Keep Item");
    assert_eq!(
        current_order.summary.unwrap().current_order_total,
        dec!(40.00)
    );

    db.drop().await;
}
