//! Return Dismiss / Write-Off Test (P6-10).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::ReturnWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_return_dismiss_and_write_off() {
    let order_service = Arc::new(OrderService::new());
    let engine = ReturnWorkflowEngine::new(order_service.clone(), None);

    // 1. Create order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Dismiss".to_string()),
                last_name: Some("User".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("dismiss@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Perishable Item".to_string(),
                quantity: 1,
                unit_price: dec!(15.00),
                variant_id: Some("var_perish".to_string()),
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

    let item_id = order.items[0].id.clone();

    // 2. Request return
    let ret = engine
        .request_return(&order.id, Some("loc_wh1"), None, None)
        .await
        .unwrap();

    engine
        .add_return_item(&ret.id, &item_id, 1, Some("spoiled"), None)
        .await
        .unwrap();

    // 3. Dismiss / write-off the return (e.g. item cost is too low to ship back)
    let dismissed_ret = engine
        .dismiss_return(
            &ret.id,
            Some("Write-off: customer keep item, zero return shipping"),
        )
        .await
        .unwrap();

    assert_eq!(dismissed_ret.status, "dismissed");
    let meta = dismissed_ret.metadata.unwrap();
    assert!(meta.get("dismiss_reason").is_some());
}
