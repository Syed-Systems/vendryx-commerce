use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{
    CartService, CreateAddressInput, CreateCartInput, CreateLineItemInput,
    CreateShippingMethodInput,
};
use stheno_flows::{CompleteCartInput, CompleteCartWorkflowEngine, StepFailureMode};
use stheno_links::LinkStore;
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;

async fn setup_test_cart(cart_service: &CartService) -> String {
    let cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_comp".to_string()),
            sales_channel_id: None,
            email: Some("comp@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Chris".to_string()),
                last_name: Some("Evans".to_string()),
                address_1: Some("100 Main St".to_string()),
                city: Some("Boston".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("02108".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Canvas Backpack".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(85.00),
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

    cart_service
        .add_shipping_method(
            &cart.id,
            CreateShippingMethodInput {
                name: "Standard".to_string(),
                amount: dec!(10.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: None,
                data: None,
                metadata: None,
            },
        )
        .await
        .expect("shipping added");

    cart.id
}

#[tokio::test]
async fn test_complete_cart_compensation_and_step_11_no_compensation() {
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

    // 1. Failure at Step 6 (authorize_payment) -> unwinds Step 5 (release_reservations)
    let cart_id_1 = setup_test_cart(&cart_service).await;
    let res_step_6 = engine
        .execute(
            CompleteCartInput {
                cart_id: cart_id_1.clone(),
                idempotency_key: None,
            },
            StepFailureMode::FailAtStep(6),
        )
        .await;

    assert!(res_step_6.is_err());
    let cart_after_fail = cart_service.get_cart(&cart_id_1).await.unwrap();
    assert!(cart_after_fail.completed_at.is_none());

    // 2. Failure at Step 10 (mark_cart_completed) -> unwinds steps 9, 8, 7, 6, 5
    let cart_id_2 = setup_test_cart(&cart_service).await;
    let res_step_10 = engine
        .execute(
            CompleteCartInput {
                cart_id: cart_id_2.clone(),
                idempotency_key: None,
            },
            StepFailureMode::FailAtStep(10),
        )
        .await;

    assert!(res_step_10.is_err());

    // 3. Failure at Step 11 (notifications/search-sync) -> NO COMPENSATION!
    // The order MUST remain intact and succeed because side-effects must not roll back placed orders (WORKFLOWS §2.3)
    let cart_id_3 = setup_test_cart(&cart_service).await;
    let res_step_11 = engine
        .execute(
            CompleteCartInput {
                cart_id: cart_id_3.clone(),
                idempotency_key: None,
            },
            StepFailureMode::FailAtStep(11),
        )
        .await;

    assert!(
        res_step_11.is_ok(),
        "Step 11 must be marked no_compensation and succeed"
    );
    let output = res_step_11.unwrap();
    assert_eq!(
        output.order.summary.as_ref().unwrap().current_order_total,
        dec!(95.00)
    );
}
