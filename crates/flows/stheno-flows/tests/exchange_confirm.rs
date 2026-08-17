//! Exchange Confirm with Computed Difference Test (P6-14).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::ExchangeWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_exchange_confirm_with_computed_difference() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = ExchangeWorkflowEngine::new(order_service.clone());

    // 1. Create base order with $50 item
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Sam".to_string()),
                last_name: Some("Swap".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("swap@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Regular Shoes".to_string(),
                quantity: 1,
                unit_price: dec!(50.00),
                variant_id: Some("var_shoes_reg".to_string()),
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

    // 2. Begin exchange for $70 luxury shoes (difference due = +$20.00)
    let exchange = engine
        .begin_exchange(&order.id, true, Some("admin"))
        .await
        .unwrap();

    engine
        .add_exchange_item(&exchange.id, &orig_item_id, 1)
        .await
        .unwrap();

    // 3. Confirm exchange with difference due
    let confirmed_exchange = engine
        .confirm_exchange(&exchange.id, Some(dec!(20.00)))
        .await
        .unwrap();

    assert_eq!(confirmed_exchange.status, "confirmed");
    assert_eq!(confirmed_exchange.difference_due, Some(dec!(20.00)));
    assert!(confirmed_exchange.allow_backorder);

    db.drop().await;
}
