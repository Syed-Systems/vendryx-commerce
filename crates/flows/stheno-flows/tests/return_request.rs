//! Return Request Test (P6-07).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::ReturnWorkflowEngine;
use stheno_order::{CreateAddressInput, CreateLineItemInput, CreateOrderInput, OrderService};

#[tokio::test]
async fn test_return_request_and_add_items() {
    let order_service = Arc::new(OrderService::new());
    let engine = ReturnWorkflowEngine::new(order_service.clone(), None);

    // 1. Create base order with 2 items
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Returner".to_string()),
                last_name: Some("One".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("returner@example.com".to_string()),
            items: Some(vec![
                CreateLineItemInput {
                    title: "Shoes".to_string(),
                    quantity: 2,
                    unit_price: dec!(80.00),
                    variant_id: Some("var_shoes".to_string()),
                    is_tax_inclusive: Some(false),
                    requires_shipping: Some(true),
                    is_discountable: Some(true),
                    ..Default::default()
                },
                CreateLineItemInput {
                    title: "Socks".to_string(),
                    quantity: 3,
                    unit_price: dec!(10.00),
                    variant_id: Some("var_socks".to_string()),
                    is_tax_inclusive: Some(false),
                    requires_shipping: Some(true),
                    is_discountable: Some(true),
                    ..Default::default()
                },
            ]),
            shipping_methods: Some(vec![]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    let shoes_item_id = order.items[0].id.clone();
    let socks_item_id = order.items[1].id.clone();

    // 2. Request a return for the order
    let ret = engine
        .request_return(&order.id, Some("loc_wh1"), Some(dec!(90.00)), None)
        .await
        .unwrap();

    assert_eq!(ret.status, "requested");
    assert_eq!(ret.refund_amount, Some(dec!(90.00)));

    // 3. Add return items with reason and note
    let ret_item1 = engine
        .add_return_item(
            &ret.id,
            &shoes_item_id,
            1,
            Some("reason_size_too_large"),
            Some("Does not fit"),
        )
        .await
        .unwrap();
    assert_eq!(ret_item1.quantity, 1);
    assert_eq!(ret_item1.received_quantity, 0);

    let ret_item2 = engine
        .add_return_item(
            &ret.id,
            &socks_item_id,
            1,
            Some("reason_wrong_color"),
            Some("Wanted black socks"),
        )
        .await
        .unwrap();
    assert_eq!(ret_item2.quantity, 1);

    // 4. Retrieve return and check items
    let loaded_ret = order_service.get_return_dto(&ret.id).await.unwrap();
    assert_eq!(loaded_ret.items.len(), 2);
}
