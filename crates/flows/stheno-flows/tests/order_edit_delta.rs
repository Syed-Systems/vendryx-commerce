//! Order Edit Delta Collection & Refund Test (P6-05).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::OrderEditWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateTransactionInput, OrderService,
};

#[tokio::test]
async fn test_order_edit_delta_collects_on_increase_and_refunds_on_decrease() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = OrderEditWorkflowEngine::new(order_service.clone(), None);

    // 1. Create base order with $100 item and capture $100 initial payment
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Delta".to_string()),
                last_name: Some("Customer".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("delta@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Premium Jacket".to_string(),
                quantity: 1,
                unit_price: dec!(100.00),
                variant_id: Some("var_jacket".to_string()),
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

    order_service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(100.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "init_pay_100".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    // 2. Increase order total: add a $50 scarf -> delta is +$50 (collect)
    let edit_up = engine
        .begin_order_edit(&order.id, Some("admin"))
        .await
        .unwrap();
    engine
        .add_item(&edit_up.id, "Silk Scarf", 1, dec!(50.00), Some("var_scarf"))
        .await
        .unwrap();

    let order_after_up = engine
        .confirm_edit(&edit_up.id, Some("admin"))
        .await
        .unwrap();
    let summary_up = order_after_up.summary.unwrap();
    assert_eq!(summary_up.current_order_total, dec!(150.00));
    assert_eq!(summary_up.paid_total, dec!(150.00)); // Collected +$50
    assert_eq!(summary_up.refunded_total, dec!(0.00));

    // 3. Decrease order total: update jacket price or remove jacket -> delta is negative (refund)
    let edit_down = engine
        .begin_order_edit(&order.id, Some("admin"))
        .await
        .unwrap();
    engine
        .remove_item(&edit_down.id, &orig_item_id)
        .await
        .unwrap();

    let order_after_down = engine
        .confirm_edit(&edit_down.id, Some("admin"))
        .await
        .unwrap();
    let summary_down = order_after_down.summary.unwrap();
    assert_eq!(summary_down.current_order_total, dec!(50.00));
    assert_eq!(summary_down.paid_total, dec!(150.00));
    assert_eq!(summary_down.refunded_total, dec!(100.00)); // Refunded $100 for removed item
    assert_eq!(summary_down.accounting_total, dec!(50.00));

    db.drop().await;
}
