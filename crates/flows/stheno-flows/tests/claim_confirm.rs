//! Claim Confirm Test for Replace and Refund Kinds (P6-12).
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::ClaimWorkflowEngine;
use stheno_order::{
    CreateAddressInput, CreateLineItemInput, CreateOrderInput, CreateTransactionInput, OrderService,
};

#[tokio::test]
async fn test_claim_confirm_replace_and_refund_kinds() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let engine = ClaimWorkflowEngine::new(order_service.clone());

    // 1. Create order with $200 payment
    let order = order_service
        .create_order(CreateOrderInput {
            customer_id: Some("cust_01".to_string()),
            sales_channel_id: Some("sc_01".to_string()),
            region_id: Some("reg_01".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Claim".to_string()),
                last_name: Some("Master".to_string()),
                country_code: Some("us".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            email: Some("claimmaster@example.com".to_string()),
            items: Some(vec![CreateLineItemInput {
                title: "Espresso Machine".to_string(),
                quantity: 1,
                unit_price: dec!(200.00),
                variant_id: Some("var_espresso".to_string()),
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
                amount: dec!(200.00),
                currency_code: "usd".to_string(),
                reference: "payment".to_string(),
                reference_id: "pay_espresso_200".to_string(),
                metadata: None,
            },
        )
        .await
        .unwrap();

    // 2. Test Claim Type: Refund
    let refund_claim = engine
        .begin_claim(&order.id, "refund", Some(dec!(200.00)), Some("admin"))
        .await
        .unwrap();
    engine
        .add_claim_item(&refund_claim.id, &item_id, 1, "broken_on_arrival", None)
        .await
        .unwrap();

    // Positive refunds must be executed through the payment service. Claim
    // confirmation cannot synthesize a ledger entry or claim that money moved.
    let confirmation = engine.confirm_claim(&refund_claim.id).await;
    assert!(matches!(
        confirmation,
        Err(stheno_types::Error::Invalid {
            code: "refund_requires_payment_service",
            ..
        })
    ));

    let updated_order = order_service.get_order(&order.id).await.unwrap();
    assert_eq!(updated_order.summary.unwrap().refunded_total, dec!(0.00));

    // 3. Test Claim Type: Replace
    let replace_claim = engine
        .begin_claim(&order.id, "replace", None, Some("admin"))
        .await
        .unwrap();
    engine
        .add_claim_item(&replace_claim.id, &item_id, 1, "missing_accessory", None)
        .await
        .unwrap();

    let confirmed_replace_claim = engine.confirm_claim(&replace_claim.id).await.unwrap();
    assert_eq!(confirmed_replace_claim.status, "confirmed");

    db.drop().await;
}
