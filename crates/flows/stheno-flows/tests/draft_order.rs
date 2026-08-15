//! Draft Order Creation and Management Test (P6-15).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_flows::DraftOrderWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateShippingMethodInput, OrderService,
};

#[tokio::test]
async fn test_draft_order_create_and_manage() {
    let order_service = Arc::new(OrderService::new());
    let engine = DraftOrderWorkflowEngine::new(order_service.clone());

    // 1. Create a draft order
    let draft = engine
        .create_draft_order(
            Some("cust_draft_01"),
            Some("draft@example.com"),
            Some("reg_us"),
            "usd",
            vec![
                CreateLineItemInput {
                    title: "Draft Item 1".to_string(),
                    quantity: 2,
                    unit_price: dec!(40.00),
                    variant_id: Some("var_draft_1".to_string()),
                    is_tax_inclusive: Some(false),
                    requires_shipping: Some(true),
                    is_discountable: Some(true),
                    ..Default::default()
                },
                CreateLineItemInput {
                    title: "Draft Item 2".to_string(),
                    quantity: 1,
                    unit_price: dec!(20.00),
                    variant_id: Some("var_draft_2".to_string()),
                    is_tax_inclusive: Some(false),
                    requires_shipping: Some(true),
                    is_discountable: Some(true),
                    ..Default::default()
                },
            ],
            vec![CreateShippingMethodInput {
                name: "Standard Delivery".to_string(),
                amount: dec!(15.00),
                shipping_option_id: Some("so_draft_std".to_string()),
                is_tax_inclusive: Some(false),
                data: None,
                metadata: None,
                ..Default::default()
            }],
            Some(CreateAddressInput {
                first_name: Some("Draft".to_string()),
                last_name: Some("Recipient".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(draft.status, "draft");
    assert!(draft.is_draft_order);
    assert_eq!(draft.items.len(), 2);
    assert_eq!(draft.shipping_methods.len(), 1);
    assert_eq!(draft.summary.unwrap().current_order_total, dec!(115.00));
}
