//! Exchange Items Integration Test (P6-13).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::ExchangeWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_exchange_begin_and_add_items() {
    let order_service = Arc::new(OrderService::new());
    let engine = ExchangeWorkflowEngine::new(order_service.clone());

    // 1. Create base order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Exchanger".to_string()),
                last_name: Some("User".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("exchange@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Medium Shirt".to_string(),
                quantity: 1,
                unit_price: dec!(45.00),
                variant_id: Some("var_shirt_m".to_string()),
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

    let shirt_item_id = order.items[0].id.clone();

    // 2. Begin exchange
    let exchange = engine
        .begin_exchange(&order.id, false, Some("admin"))
        .await
        .unwrap();

    assert_eq!(exchange.status, "open");
    assert!(!exchange.allow_backorder);

    // 3. Add inbound (returned) item and outbound (new) item
    let exc_item1 = engine
        .add_exchange_item(&exchange.id, &shirt_item_id, 1)
        .await
        .unwrap();
    assert_eq!(exc_item1.quantity, 1);

    let loaded_exc = order_service.get_exchange_dto(&exchange.id).await.unwrap();
    assert_eq!(loaded_exc.items.len(), 1);
}
