//! Timeline reconstruction test (P6-18).
use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_order::{
    CreateAddressInput, CreateOrderChangeActionInput, CreateOrderChangeInput, CreateOrderInput,
    CreateReturnInput, CreateTransactionInput, OrderService,
};

#[tokio::test]
async fn test_order_timeline_reconstruction_from_change_actions_and_transactions() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    // 1. Create order
    let order = service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_timeline_01".to_string()),
            sales_channel_id: Some("sc_default".to_string()),
            region_id: Some("reg_us".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                customer_id: Some("cust_timeline_01".to_string()),
                first_name: Some("Timeline".to_string()),
                last_name: Some("Tester".to_string()),
                address_1: Some("100 Time Way".to_string()),
                city: Some("Chronos".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("90210".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("timeline@example.com".to_string()),
            items: Some(vec![]),
            shipping_methods: Some(vec![]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // 2. Add an order change with actions
    let change = service
        .create_order_change(CreateOrderChangeInput {
            order_id: order.id.clone(),
            change_type: "edit".to_string(),
            requested_by: Some("admin_1".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    service
        .add_order_change_action(
            &change.id,
            CreateOrderChangeActionInput {
                action: "ITEM_ADD".to_string(),
                reference: Some("order_item".to_string()),
                reference_id: None,
                amount: Some(dec!(50.00)),
                details: Some(serde_json::json!({
                    "title": "Added Item",
                    "quantity": 1,
                    "unit_price": "50.00"
                })),
            },
        )
        .await
        .unwrap();

    service
        .confirm_order_change(&change.id, Some("admin_1"))
        .await
        .unwrap();

    // 3. Add payment and refund transactions
    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(50.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "pay_001".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(10.00),
                currency_code: "usd".to_string(),
                reference: "refund".to_string(),
                reference_id: "ref_001".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    // 4. Create a return
    service
        .create_return(CreateReturnInput {
            order_id: order.id.clone(),
            location_id: Some("loc_wh1".to_string()),
            refund_amount: Some(dec!(10.00)),
            metadata: None,
        })
        .await
        .unwrap();

    // 5. Reconstruct timeline
    let timeline = service.get_order_timeline(&order.id).await.unwrap();

    assert!(timeline.len() >= 5);
    assert_eq!(timeline[0].event_type, "order_created");
    assert!(timeline
        .iter()
        .any(|e| e.event_type == "change_action_item_add"));
    assert!(timeline
        .iter()
        .any(|e| e.event_type == "order_change_confirmed"));
    assert!(timeline.iter().any(|e| e.event_type == "payment_captured"));
    assert!(timeline.iter().any(|e| e.event_type == "refund_issued"));
    assert!(timeline.iter().any(|e| e.event_type == "return_requested"));
}
