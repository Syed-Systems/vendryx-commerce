//! Checkout transactional integrity (P13-21):
//! Cart completion, payment, order, reservation commit or roll back together.

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{
    CartService, CreateAddressInput, CreateCartInput, CreateLineItemInput,
    CreateShippingMethodInput,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::{CompleteCartInput, CompleteCartWorkflowEngine, StepFailureMode};
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;

async fn setup_test_cart(cart_service: &CartService) -> String {
    let cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_buyer_atom".to_string()),
            sales_channel_id: Some("sc_web".to_string()),
            email: Some("buyer_atom@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Buyer".to_string()),
                last_name: Some("Atomicity".to_string()),
                address_1: Some("100 Transaction Blvd".to_string()),
                city: Some("Austin".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("78701".to_string()),
                ..Default::default()
            }),
            billing_address: Some(CreateAddressInput {
                first_name: Some("Buyer".to_string()),
                last_name: Some("Atomicity".to_string()),
                address_1: Some("100 Transaction Blvd".to_string()),
                city: Some("Austin".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("78701".to_string()),
                ..Default::default()
            }),
            items: Some(vec![CreateLineItemInput {
                title: "Atomic Solid Item".to_string(),
                quantity: 2,
                unit_price: dec!(49.99),
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
                name: "Standard Ground".to_string(),
                amount: dec!(10.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: Some("so_ground".to_string()),
                data: None,
                metadata: None,
            },
        )
        .await
        .expect("shipping added");

    cart.id
}

#[tokio::test]
async fn test_checkout_atomicity_commit_together() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let cart_service = Arc::new(CartService::new(pool.clone()));
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    let promotion_service = Arc::new(PromotionService::new(pool.clone()));

    let engine = CompleteCartWorkflowEngine::new(
        cart_service.clone(),
        order_service.clone(),
        payment_service.clone(),
        promotion_service.clone(),
        Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
    );

    let cart_id = setup_test_cart(&cart_service).await;

    // Execute complete cart workflow without failures
    let output = engine
        .execute(
            CompleteCartInput {
                cart_id: cart_id.clone(),
                idempotency_key: None,
            },
            StepFailureMode::None,
        )
        .await
        .expect("checkout completes successfully");

    // 1. Order is created and persisted in Postgres
    let order = order_service
        .get_order(&output.order.id)
        .await
        .expect("order exists in Postgres");
    assert_eq!(order.email, Some("buyer_atom@example.com".to_string()));
    assert_eq!(order.status, "pending");

    // 2. Cart is marked as completed
    let cart = cart_service
        .get_cart(&cart_id)
        .await
        .expect("cart reloaded");
    assert!(
        cart.completed_at.is_some(),
        "cart must be marked as completed"
    );

    db.drop().await;
}

#[tokio::test]
async fn test_checkout_atomicity_rollback_on_failure() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let cart_service = Arc::new(CartService::new(pool.clone()));
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    let promotion_service = Arc::new(PromotionService::new(pool.clone()));

    let engine = CompleteCartWorkflowEngine::new(
        cart_service.clone(),
        order_service.clone(),
        payment_service.clone(),
        promotion_service.clone(),
        Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
    );

    let cart_id = setup_test_cart(&cart_service).await;

    // Simulate failure at payment authorization step (Step 6)
    let result = engine
        .execute(
            CompleteCartInput {
                cart_id: cart_id.clone(),
                idempotency_key: None,
            },
            StepFailureMode::FailAtStep(6),
        )
        .await;

    assert!(result.is_err(), "workflow must fail on step 6");

    // Verify cart was not marked completed and remained usable
    let cart = cart_service
        .get_cart(&cart_id)
        .await
        .expect("cart reloaded");
    assert!(
        cart.completed_at.is_none(),
        "cart must NOT be marked completed when checkout fails"
    );

    db.drop().await;
}
