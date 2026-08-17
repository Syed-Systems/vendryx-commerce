//! Order Edit Preview Zero-Writes Test (P6-03).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderEditWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_order_edit_preview_computed_with_zero_writes() {
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
                first_name: Some("Alice".to_string()),
                last_name: Some("Wonder".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("alice@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Book".to_string(),
                quantity: 1,
                unit_price: dec!(50.00),
                variant_id: Some("var_book".to_string()),
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

    let initial_version = order.version;
    let initial_summary = order.summary.clone().unwrap();

    // 2. Begin edit and add items
    let edit = engine
        .begin_order_edit(&order.id, Some("admin"))
        .await
        .unwrap();

    engine
        .add_item(&edit.id, "Second Book", 2, dec!(30.00), Some("var_book2"))
        .await
        .unwrap();

    // 3. Compute preview (zero writes)
    let preview = engine.preview_edit(&edit.id).await.unwrap();

    assert_eq!(preview.current_total.amount, dec!(50.00));
    assert_eq!(preview.difference_total.amount, dec!(60.00));
    assert_eq!(preview.new_total.amount, dec!(110.00));
    assert_eq!(preview.actions_count, 1);

    // 4. Assert that order state and version are UNCHANGED before confirmation
    let unchanged_order = order_service.get_order(&order.id).await.unwrap();
    assert_eq!(unchanged_order.version, initial_version);
    assert_eq!(
        unchanged_order.summary.unwrap().current_order_total,
        initial_summary.current_order_total
    );
    assert_eq!(unchanged_order.items.len(), 1);

    db.drop().await;
}
