use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{CartService, CreateAddressInput, CreateCartInput, CreateLineItemInput};
use stheno_flows::{CompleteCartInput, CompleteCartWorkflowEngine, StepFailureMode};
use stheno_links::LinkStore;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;

#[tokio::test]
async fn test_no_double_charge_duplicate_complete_returns_same_order() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let cart_service = Arc::new(CartService::new(pool.clone()));
    let order_service = Arc::new(OrderService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()));
    let promotion_service = Arc::new(PromotionService::new(pool.clone()));
    let link_store = Arc::new(LinkStore::new());

    let engine = CompleteCartWorkflowEngine::new(
        cart_service.clone(),
        order_service.clone(),
        payment_service.clone(),
        promotion_service.clone(),
        link_store.clone(),
        Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
    );

    // 1. Seed cart
    let cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_idem".to_string()),
            sales_channel_id: None,
            email: Some("idem@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Idem".to_string()),
                last_name: Some("User".to_string()),
                address_1: Some("100 Safe Way".to_string()),
                city: Some("New York".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("10001".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Noise Cancelling Headphones".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(350.00),
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
            metadata: None,
        })
        .await
        .expect("cart created");

    let idem_key = "idemp_checkout_9999".to_string();

    // 2. First complete execution
    let first_res = engine
        .execute(
            CompleteCartInput {
                cart_id: cart.id.clone(),
                idempotency_key: Some(idem_key.clone()),
            },
            StepFailureMode::None,
        )
        .await
        .expect("first complete succeeded");

    // 3. Second duplicate complete execution with identical idempotency key
    let second_res = engine
        .execute(
            CompleteCartInput {
                cart_id: cart.id.clone(),
                idempotency_key: Some(idem_key.clone()),
            },
            StepFailureMode::None,
        )
        .await
        .expect("second complete succeeded");

    // Both must return the EXACT same order ID and details (zero double-charging)
    assert_eq!(first_res.order.id, second_res.order.id);
    assert_eq!(first_res.order.display_id, second_res.order.display_id);
    assert_eq!(first_res.order.summary.unwrap().paid_total, dec!(350.00));
    assert_eq!(second_res.order.summary.unwrap().paid_total, dec!(350.00));
}
