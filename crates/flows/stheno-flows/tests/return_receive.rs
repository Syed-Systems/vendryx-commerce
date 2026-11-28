//! Return Receive & Confirmation Test (P6-08).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::ReturnWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateTransactionInput, OrderService,
};

#[tokio::test]
async fn test_return_receive_confirm_and_cancel_lifecycle() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = ReturnWorkflowEngine::new(order_service.clone(), None);

    // 1. Create order with $100 payment
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Receiver".to_string()),
                last_name: Some("Test".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("recv@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Electronic Gadget".to_string(),
                quantity: 2,
                unit_price: dec!(50.00),
                variant_id: Some("var_gadget".to_string()),
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
                reference_id: "pay_recv_01".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    // 2. Request return and add item
    let ret = engine
        .request_return(&order.id, Some("loc_wh1"), Some(dec!(50.00)), None)
        .await
        .unwrap();

    engine
        .add_return_item(&ret.id, &item_id, 1, None, None)
        .await
        .unwrap();

    // Reject the whole batch if it contains an unknown item; the valid line
    // must not be partially applied.
    let invalid_batch = engine
        .receive_return_items(
            &ret.id,
            vec![
                (item_id.clone(), 1, None),
                ("not_in_return".to_string(), 1, None),
            ],
        )
        .await;
    assert!(matches!(
        invalid_batch,
        Err(stheno_types::Error::Invalid {
            code: "return_item_not_found",
            ..
        })
    ));
    assert_eq!(
        order_service.get_return_dto(&ret.id).await.unwrap().items[0].received_quantity,
        0
    );

    // 3. Two warehouse requests with the same absolute receipt total serialize
    // on the return row and cannot double-count the received units.
    let (first_receipt, second_receipt) = tokio::join!(
        engine.receive_return_items(&ret.id, vec![(item_id.clone(), 1, None)]),
        engine.receive_return_items(&ret.id, vec![(item_id.clone(), 1, None)]),
    );
    let first_receipt = first_receipt.unwrap();
    let second_receipt = second_receipt.unwrap();

    assert_eq!(first_receipt.status, "received");
    assert_eq!(first_receipt.items[0].received_quantity, 1);
    assert_eq!(second_receipt.status, "received");
    assert_eq!(second_receipt.items[0].received_quantity, 1);
    assert_eq!(
        order_service.get_return_dto(&ret.id).await.unwrap().items[0].received_quantity,
        1
    );

    // Replaying the same absolute quantity is safe and does not double-count.
    let replayed = engine
        .receive_return_items(&ret.id, vec![(item_id.clone(), 1, None)])
        .await;
    assert_eq!(replayed.unwrap().items[0].received_quantity, 1);

    // A quantity above the request is rejected without changing the receipt.
    let excess = engine
        .receive_return_items(&ret.id, vec![(item_id.clone(), 2, None)])
        .await;
    assert!(matches!(
        excess,
        Err(stheno_types::Error::Invalid {
            code: "return_receive_exceeds_requested",
            ..
        })
    ));
    assert_eq!(
        order_service.get_return_dto(&ret.id).await.unwrap().items[0].received_quantity,
        1
    );
    for invalid in [
        vec![(item_id.clone(), -1, None)],
        vec![(item_id.clone(), 1, Some(2))],
    ] {
        assert!(matches!(
            engine.receive_return_items(&ret.id, invalid).await,
            Err(stheno_types::Error::Invalid {
                code: "invalid_return_receive_quantity",
                ..
            })
        ));
    }

    // 4. Confirm return and process refund
    let confirmation = engine.confirm_return(&ret.id, Some(dec!(50.00))).await;
    assert!(matches!(
        confirmation,
        Err(stheno_types::Error::Invalid {
            code: "refund_requires_payment_service",
            ..
        })
    ));

    // Confirming a return cannot manufacture a refund ledger entry. A payment
    // service refund must be completed before the return can be marked complete.
    let updated_order = order_service.get_order(&order.id).await.unwrap();
    let summary = updated_order.summary.unwrap();
    assert_eq!(summary.paid_total, dec!(100.00));
    assert_eq!(summary.refunded_total, dec!(0.00));
    assert_eq!(summary.accounting_total, dec!(100.00));
    assert_eq!(
        order_service.get_return_dto(&ret.id).await.unwrap().status,
        "received"
    );

    db.drop().await;
}
