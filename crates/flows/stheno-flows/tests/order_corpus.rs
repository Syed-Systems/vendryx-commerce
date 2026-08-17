//! Medusa Fixture Corpus Test for All 4 Mutation Families (P6-19).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::{
    ClaimWorkflowEngine, ExchangeWorkflowEngine, OrderEditWorkflowEngine, ReturnWorkflowEngine,
};
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateTransactionInput, OrderService,
};

#[tokio::test]
async fn test_medusa_fixture_corpus_for_all_four_mutation_families() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let edit_engine = OrderEditWorkflowEngine::new(order_service.clone(), None);
    let return_engine = ReturnWorkflowEngine::new(order_service.clone(), None);
    let claim_engine = ClaimWorkflowEngine::new(order_service.clone());
    let exchange_engine = ExchangeWorkflowEngine::new(order_service.clone());

    // Create standard Medusa fixture base order
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_medusa_corpus".to_string()),
            sales_channel_id: Some("sc_default".to_string()),
            region_id: Some("reg_us".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Medusa".to_string()),
                last_name: Some("Tester".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("medusa_corpus@example.com".to_string()),
            items: Some(vec![
                CreateLineItemInput {
                    title: "Medusa Hoodie".to_string(),
                    quantity: 2,
                    unit_price: dec!(65.00),
                    variant_id: Some("var_hoodie".to_string()),
                    is_tax_inclusive: Some(false),
                    requires_shipping: Some(true),
                    is_discountable: Some(true),
                    ..Default::default()
                },
                CreateLineItemInput {
                    title: "Medusa Cap".to_string(),
                    quantity: 1,
                    unit_price: dec!(25.00),
                    variant_id: Some("var_cap".to_string()),
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

    let hoodie_item_id = order.items[0].id.clone();
    let cap_item_id = order.items[1].id.clone();

    // Initial $155 payment
    order_service
        .add_transaction(
            &order.id,
            CreateTransactionInput {
                amount: dec!(155.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "pay_corpus_01".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    // ── Family 1: Order Edit Mutation ──
    let edit = edit_engine
        .begin_order_edit(&order.id, Some("admin"))
        .await
        .unwrap();
    edit_engine
        .add_item(&edit.id, "Medusa Socks", 2, dec!(12.00), Some("var_socks"))
        .await
        .unwrap();
    let preview = edit_engine.preview_edit(&edit.id).await.unwrap();
    assert_eq!(preview.difference_total.amount, dec!(24.00));
    let order_v2 = edit_engine
        .confirm_edit(&edit.id, Some("admin"))
        .await
        .unwrap();
    assert_eq!(order_v2.version, 2);
    assert_eq!(order_v2.items.len(), 3);

    // ── Family 2: Return Mutation ──
    let reason = order_service
        .create_return_reason(stheno_order::CreateReturnReasonInput {
            value: "wrong_size".to_string(),
            label: "Wrong Size".to_string(),
            description: Some("Item size does not fit".to_string()),
        })
        .await
        .unwrap();

    let ret = return_engine
        .request_return(&order.id, Some("loc_wh_main"), Some(dec!(25.00)), None)
        .await
        .unwrap();
    return_engine
        .add_return_item(&ret.id, &cap_item_id, 1, Some(&reason.id), None)
        .await
        .unwrap();
    return_engine
        .receive_return_items(&ret.id, vec![(cap_item_id.clone(), 1, None)])
        .await
        .unwrap();
    let completed_ret = return_engine
        .confirm_return(&ret.id, Some(dec!(25.00)))
        .await
        .unwrap();
    assert_eq!(completed_ret.status, "completed");

    // ── Family 3: Claim Mutation ──
    let claim = claim_engine
        .begin_claim(&order.id, "refund", Some(dec!(65.00)), Some("admin"))
        .await
        .unwrap();
    claim_engine
        .add_claim_item(
            &claim.id,
            &hoodie_item_id,
            1,
            "seam_defect",
            Some(vec!["https://cdn.example.com/claim.png".to_string()]),
        )
        .await
        .unwrap();
    let confirmed_claim = claim_engine.confirm_claim(&claim.id).await.unwrap();
    assert_eq!(confirmed_claim.status, "confirmed");

    // ── Family 4: Exchange Mutation ──
    let exchange = exchange_engine
        .begin_exchange(&order.id, true, Some("admin"))
        .await
        .unwrap();
    exchange_engine
        .add_exchange_item(&exchange.id, &hoodie_item_id, 1)
        .await
        .unwrap();
    let confirmed_exchange = exchange_engine
        .confirm_exchange(&exchange.id, Some(dec!(10.00)))
        .await
        .unwrap();
    assert_eq!(confirmed_exchange.status, "confirmed");
    assert_eq!(confirmed_exchange.difference_due, Some(dec!(10.00)));

    // Reconstruct full timeline across all four mutation families
    let timeline = order_service.get_order_timeline(&order.id).await.unwrap();
    assert!(timeline.len() >= 6);

    db.drop().await;
}
