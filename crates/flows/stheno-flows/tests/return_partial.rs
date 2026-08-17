//! Return Partial Receipt with Damaged Quantities Test (P6-09).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::ReturnWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateTransactionInput, OrderService,
};

#[tokio::test]
async fn test_return_partial_receipt_with_damaged_quantities() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = ReturnWorkflowEngine::new(order_service.clone(), None);

    // 1. Create order with 10 units @ $10 each = $100 total
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Partial".to_string()),
                last_name: Some("Receiver".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("partial@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Glass Vases".to_string(),
                quantity: 10,
                unit_price: dec!(10.00),
                variant_id: Some("var_vase".to_string()),
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

    order_service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(100.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "pay_partial_100".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    let reason = order_service
        .create_return_reason(stheno_order::CreateReturnReasonInput {
            value: "damaged_in_transit".to_string(),
            label: "Damaged in Transit".to_string(),
            description: Some("Item damaged in transit".to_string()),
        })
        .await
        .unwrap();

    // 2. Request return of 5 vases
    let ret = engine
        .request_return(&order.id, Some("loc_wh1"), None, None)
        .await
        .unwrap();

    engine
        .add_return_item(&ret.id, &item_id, 5, Some(&reason.id), None)
        .await
        .unwrap();

    // 3. Receive partial: 3 units received intact, 2 units received DAMAGED (not restockable)
    let received_ret = engine
        .receive_return_items(&ret.id, vec![(item_id.clone(), 5, Some(2))])
        .await
        .unwrap();

    assert_eq!(received_ret.items[0].received_quantity, 5);
    assert_eq!(received_ret.items[0].damaged_quantity, 2);

    // 4. Confirm return with calculated refund for all 5 returned units ($50.00)
    let confirmed_ret = engine
        .confirm_return(&ret.id, Some(dec!(50.00)))
        .await
        .unwrap();

    assert_eq!(confirmed_ret.status, "completed");
    assert_eq!(confirmed_ret.refund_amount, Some(dec!(50.00)));

    // 5. Verify order financial summary reflect the $50 refund
    let updated_order = order_service.get_order(&order.id).await.unwrap();
    let summary = updated_order.summary.unwrap();
    assert_eq!(summary.refunded_total, dec!(50.00));
    assert_eq!(summary.accounting_total, dec!(50.00));

    db.drop().await;
}
