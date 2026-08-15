use rust_decimal_macros::dec;
use stheno_order::*;

#[tokio::test]
async fn test_order_summary_computation_from_transactions_and_items() {
    let service = OrderService::new();

    // 1. Create order with 1 item of $100 and $10 shipping -> total $110
    let order = service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_1".to_string()),
            sales_channel_id: None,
            email: Some("summary@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: None,
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Wool Blanket".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(100.00),
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
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Standard".to_string(),
                description: None,
                amount: dec!(10.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: None,
                data: None,
                metadata: None,
            }]),
            is_draft_order: None,
            metadata: None,
        })
        .await
        .expect("order created");

    let sum1 = order.summary.unwrap();
    assert_eq!(sum1.current_order_total, dec!(110.00));
    assert_eq!(sum1.paid_total, dec!(0.00));
    assert_eq!(sum1.pending_difference, dec!(110.00));

    // 2. Add payment transaction of $110
    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(110.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "pay_1".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("transaction added");

    let updated_order = service.get_order(&order.id).await.expect("order fetched");
    let sum2 = updated_order.summary.unwrap();
    assert_eq!(sum2.paid_total, dec!(110.00));
    assert_eq!(sum2.refunded_total, dec!(0.00));
    assert_eq!(sum2.accounting_total, dec!(110.00));
    assert_eq!(sum2.pending_difference, dec!(0.00));
    assert_eq!(updated_order.payment_status, "captured");

    // 3. Add partial refund transaction of $30
    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(30.00),
                currency_code: "usd".to_string(),
                reference: "refund".to_string(),
                reference_id: "ref_1".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("refund added");

    let refunded_order = service.get_order(&order.id).await.expect("order fetched");
    let sum3 = refunded_order.summary.unwrap();
    assert_eq!(sum3.paid_total, dec!(110.00));
    assert_eq!(sum3.refunded_total, dec!(30.00));
    assert_eq!(sum3.accounting_total, dec!(80.00));
    assert_eq!(sum3.pending_difference, dec!(30.00));
}
