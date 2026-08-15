use rust_decimal_macros::dec;
use stheno_order::*;

#[tokio::test]
async fn test_transaction_ledger_and_status_independence() {
    let service = OrderService::new();

    // 1. Create order
    let order = service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_1".to_string()),
            sales_channel_id: None,
            email: Some("ledger@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Desk Lamp".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(75.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: None,
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

    assert_eq!(order.payment_status, "not_paid");
    assert_eq!(order.fulfillment_status, "not_fulfilled");

    // 2. Fulfill order independently before payment (e.g. COD flow or invoice terms)
    let fulfilled_order = service
        .update_order(
            &order.id,
            UpdateOrderInput {
                fulfillment_status: Some("fulfilled".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("fulfillment updated");

    assert_eq!(fulfilled_order.fulfillment_status, "fulfilled");
    assert_eq!(fulfilled_order.payment_status, "not_paid"); // Status independence preserved!

    // 3. Add payment transaction
    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(75.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "pay_cod_1".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("transaction added");

    let paid_order = service.get_order(&order.id).await.expect("order fetched");
    assert_eq!(paid_order.payment_status, "captured");
    assert_eq!(paid_order.fulfillment_status, "fulfilled");
    assert_eq!(paid_order.transactions.len(), 1);
}
