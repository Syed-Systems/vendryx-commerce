use rust_decimal_macros::dec;
use serde_json::json;
use stheno_order::*;

#[tokio::test]
async fn test_order_change_integration_preview_and_confirmation() {
    let service = OrderService::new();

    // 1. Create base order at version 1 (total: 200.00)
    let order = service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_1".to_string()),
            sales_channel_id: None,
            email: Some("test@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Oxford Shirt".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 2,
                unit_price: dec!(100.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: Some("var_shirt".to_string()),
                product_id: None,
                product_title: None,
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: None,
                variant_sku: None,
                variant_barcode: None,
                variant_title: None,
                variant_option_values: None,
                metadata: None,
            }]),
            shipping_methods: None,
            is_draft_order: None,
            metadata: None,
        })
        .await
        .expect("order created");

    assert_eq!(order.version, 1);
    assert_eq!(order.items[0].quantity, 2);
    let line_item_id = order.items[0].id.clone();

    // 2. Open an OrderChange (edit)
    let change = service
        .create_order_change(CreateOrderChangeInput {
            order_id: order.id.clone(),
            change_type: "edit".to_string(),
            requested_by: Some("admin_user".to_string()),
            metadata: None,
        })
        .await
        .expect("order change created");

    assert_eq!(change.version, 1);
    assert_eq!(change.status, "pending");

    // 3. Add an action: Update quantity from 2 to 1 (amount delta: -100)
    service
        .add_order_change_action(
            &change.id,
            CreateOrderChangeActionInput {
                action: "ITEM_UPDATE".to_string(),
                reference: Some("line_item".to_string()),
                reference_id: Some(line_item_id.clone()),
                amount: Some(dec!(-100.00)),
                details: Some(json!({ "quantity": 1 })),
            },
        )
        .await
        .expect("action added");

    // 4. Preview in-memory diff without writing (P5-02 step 3)
    let preview = service
        .preview_order_change(&change.id)
        .await
        .expect("preview calculated");

    assert_eq!(preview.current_total.amount, dec!(200.00));
    assert_eq!(preview.difference_total.amount, dec!(-100.00));
    assert_eq!(preview.new_total.amount, dec!(100.00));

    // 5. Confirm OrderChange (bumps version to 2, writes new order_items at version 2)
    let updated_order = service
        .confirm_order_change(&change.id, Some("admin_user"))
        .await
        .expect("order change confirmed");

    assert_eq!(updated_order.version, 2);
    assert_eq!(updated_order.items.len(), 1);
    assert_eq!(updated_order.items[0].quantity, 1);
    assert_eq!(
        updated_order.summary.as_ref().unwrap().current_order_total,
        dec!(100.00)
    );
}
