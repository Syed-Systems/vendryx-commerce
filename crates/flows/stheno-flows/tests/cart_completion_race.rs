//! Concurrent completion of the same cart produces exactly one order (P13-22).

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{
    CartService, CreateAddressInput, CreateCartInput, CreateLineItemInput,
    CreateShippingMethodInput,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::{CompleteCartInput, CompleteCartWorkflowEngine, StepFailureMode};
use stheno_links::LinkStore;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;

#[tokio::test]
async fn test_concurrent_completion_of_same_cart_produces_exactly_one_order() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 10).expect("pool");
    let cart_service = Arc::new(CartService::new(pool.clone()));
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    let promotion_service = Arc::new(PromotionService::new(pool.clone()));
    let link_store = Arc::new(LinkStore::new());

    let engine = Arc::new(CompleteCartWorkflowEngine::new(
        cart_service.clone(),
        order_service.clone(),
        payment_service.clone(),
        promotion_service.clone(),
        link_store.clone(),
    ));

    // Seed a cart
    let cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_race".to_string()),
            sales_channel_id: Some("sc_web".to_string()),
            email: Some("race@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Race".to_string()),
                last_name: Some("Condition".to_string()),
                address_1: Some("123 Parallel St".to_string()),
                city: Some("Chicago".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("60601".to_string()),
                ..Default::default()
            }),
            billing_address: Some(CreateAddressInput {
                first_name: Some("Race".to_string()),
                last_name: Some("Condition".to_string()),
                address_1: Some("123 Parallel St".to_string()),
                city: Some("Chicago".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("60601".to_string()),
                ..Default::default()
            }),
            items: Some(vec![CreateLineItemInput {
                title: "Rare Limited Item".to_string(),
                quantity: 1,
                unit_price: dec!(250.00),
                is_tax_inclusive: Some(false),
                requires_shipping: Some(true),
                is_discountable: Some(true),
                ..Default::default()
            }]),
            metadata: None,
        })
        .await
        .expect("cart created");

    cart_service
        .add_shipping_method(
            &cart.id,
            CreateShippingMethodInput {
                name: "Overnight Delivery".to_string(),
                amount: dec!(25.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: Some("so_overnight".to_string()),
                data: None,
                metadata: None,
            },
        )
        .await
        .expect("shipping added");

    let cart_id = cart.id.clone();
    let idempotency_key = format!("idem_race_{}", cart_id);

    // Spawn 10 concurrent requests to complete the exact same cart
    let mut handles = Vec::new();
    for _ in 0..10 {
        let eng = engine.clone();
        let cid = cart_id.clone();
        let ikey = idempotency_key.clone();
        handles.push(tokio::spawn(async move {
            eng.execute(
                CompleteCartInput {
                    cart_id: cid,
                    idempotency_key: Some(ikey),
                },
                StepFailureMode::None,
            )
            .await
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        let res = h.await.expect("join handle").expect("completion ok");
        results.push(res);
    }

    // All 10 concurrent invocations must return the EXACT same order ID
    let first_order_id = results[0].order.id.clone();
    for r in &results {
        assert_eq!(
            r.order.id, first_order_id,
            "all concurrent completion requests must resolve to the identical order"
        );
    }

    // Verify exactly one order exists in Postgres with this email
    let (orders, total) = order_service
        .list_orders(stheno_order::OrderFilter {
            customer_id: Some("cus_race".to_string()),
            ..Default::default()
        })
        .await
        .expect("listed orders");

    assert_eq!(total, 1, "exactly one order must be persisted in database");
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].id, first_order_id);

    db.drop().await;
}
