//! Order Edit Shipping Integration Test (P6-02).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::OrderEditWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateShippingMethodInput,
    OrderService,
};

#[tokio::test]
async fn test_order_edit_add_update_remove_shipping() {
    let order_service = Arc::new(OrderService::new());
    let engine = OrderEditWorkflowEngine::new(order_service.clone(), None);

    // 1. Create base order with 1 shipping method
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Jane".to_string()),
                last_name: Some("Smith".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("jane@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Product 1".to_string(),
                quantity: 1,
                unit_price: dec!(100.00),
                variant_id: Some("var_1".to_string()),
                is_tax_inclusive: Some(false),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                ..Default::default()
            }]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Standard Ground".to_string(),
                amount: dec!(10.00),
                shipping_option_id: Some("so_standard".to_string()),
                is_tax_inclusive: Some(false),
                data: None,
                metadata: None,
                ..Default::default()
            }]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    let orig_sm_id = order.shipping_methods[0].id.clone();
    assert_eq!(order.shipping_methods.len(), 1);

    // 2. Begin edit & add express shipping
    let edit1 = engine
        .begin_order_edit(&order.id, Some("admin_user"))
        .await
        .unwrap();
    engine
        .add_shipping_method(
            &edit1.id,
            "Express Priority",
            dec!(25.00),
            Some("so_express"),
        )
        .await
        .unwrap();

    let updated_order1 = engine
        .confirm_edit(&edit1.id, Some("admin_user"))
        .await
        .unwrap();
    assert_eq!(updated_order1.version, 2);
    assert_eq!(updated_order1.shipping_methods.len(), 2);

    // 3. Begin edit & update standard shipping amount
    let edit2 = engine
        .begin_order_edit(&order.id, Some("admin_user"))
        .await
        .unwrap();
    engine
        .update_shipping_method(&edit2.id, &orig_sm_id, dec!(12.00))
        .await
        .unwrap();

    let updated_order2 = engine
        .confirm_edit(&edit2.id, Some("admin_user"))
        .await
        .unwrap();
    assert_eq!(updated_order2.version, 3);
    let updated_sm = updated_order2
        .shipping_methods
        .iter()
        .find(|s| s.id == orig_sm_id)
        .unwrap();
    assert_eq!(updated_sm.amount, dec!(12.00));

    // 4. Begin edit & remove standard shipping
    let edit3 = engine
        .begin_order_edit(&order.id, Some("admin_user"))
        .await
        .unwrap();
    engine
        .remove_shipping_method(&edit3.id, &orig_sm_id)
        .await
        .unwrap();

    let updated_order3 = engine
        .confirm_edit(&edit3.id, Some("admin_user"))
        .await
        .unwrap();
    assert_eq!(updated_order3.version, 4);
    assert_eq!(updated_order3.shipping_methods.len(), 1);
    assert_eq!(updated_order3.shipping_methods[0].name, "Express Priority");
}
