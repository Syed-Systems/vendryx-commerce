use rust_decimal_macros::dec;
use serde_json::json;
use stheno_order::*;

#[tokio::test]
async fn test_order_versioning_and_historical_snapshot_load() {
    let service = OrderService::new();

    // 1. Create Order at version 1 with 3 units of Item A (total: 150.00)
    let order = service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_1".to_string()),
            sales_channel_id: None,
            email: Some("version@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Espresso Beans".to_string(),
                subtitle: Some("1kg".to_string()),
                thumbnail: None,
                quantity: 3,
                unit_price: dec!(50.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: Some("var_beans".to_string()),
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

    let item_id = order.items[0].id.clone();

    // 2. Modify Order via OrderChange to quantity = 5 at version 2
    let change = service
        .create_order_change(CreateOrderChangeInput {
            order_id: order.id.clone(),
            change_type: "edit".to_string(),
            requested_by: Some("admin".to_string()),
            metadata: None,
        })
        .await
        .expect("change created");

    service
        .add_order_change_action(
            &change.id,
            CreateOrderChangeActionInput {
                action: "ITEM_UPDATE".to_string(),
                reference: Some("line_item".to_string()),
                reference_id: Some(item_id.clone()),
                amount: Some(dec!(100.00)),
                details: Some(json!({ "quantity": 5 })),
            },
        )
        .await
        .expect("action added");

    service
        .confirm_order_change(&change.id, Some("admin"))
        .await
        .expect("change confirmed");

    // 3. Load historical snapshot at version 1 (must show quantity = 3 and total = 150.00)
    let snapshot_v1 = service
        .get_order_at_version(&order.id, 1)
        .await
        .expect("v1 loaded");

    assert_eq!(snapshot_v1.version, 1);
    assert_eq!(snapshot_v1.items[0].quantity, 3);
    assert_eq!(
        snapshot_v1.summary.as_ref().unwrap().current_order_total,
        dec!(150.00)
    );

    // 4. Load latest snapshot at version 2 (must show quantity = 5 and total = 250.00)
    let snapshot_v2 = service
        .get_order_at_version(&order.id, 2)
        .await
        .expect("v2 loaded");

    assert_eq!(snapshot_v2.version, 2);
    assert_eq!(snapshot_v2.items[0].quantity, 5);
    assert_eq!(
        snapshot_v2.summary.as_ref().unwrap().current_order_total,
        dec!(250.00)
    );
}
