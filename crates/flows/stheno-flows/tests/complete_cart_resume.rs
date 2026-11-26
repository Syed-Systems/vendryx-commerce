use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{
    CartService, CreateAddressInput, CreateCartInput, CreateLineItemInput,
    CreateShippingMethodInput,
};
use stheno_flows::{CompleteCartInput, CompleteCartWorkflowEngine, StepFailureMode};
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;

async fn setup_cart(cart_service: &CartService, id_suffix: usize) -> String {
    let cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some(format!("cus_resume_{id_suffix}")),
            sales_channel_id: None,
            email: Some(format!("resume{id_suffix}@example.com")),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Resume".to_string()),
                last_name: Some("Tester".to_string()),
                address_1: Some("123 Checkpoint Ave".to_string()),
                city: Some("Austin".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("78701".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Mechanical Keyboard".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(150.00),
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
                name: "Standard Ground".to_string(),
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
async fn test_crash_resumption_at_all_11_steps() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
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
        Arc::new(stheno_workflow::WorkflowStore::from_pool(pool.clone())),
    );

    // Test crash and resume at each step 1..=10
    for crash_step in 1..=10 {
        let cart_id = setup_cart(&cart_service, crash_step).await;

        // 1. Simulate crash at step
        let crash_res = engine
            .execute(
                CompleteCartInput {
                    cart_id: cart_id.clone(),
                    idempotency_key: None,
                },
                StepFailureMode::CrashAtStep(crash_step),
            )
            .await;

        assert!(crash_res.is_err(), "Step {crash_step} should have crashed");

        // 2. Resume execution from checkpoint
        let resume_res = engine
            .execute(
                CompleteCartInput {
                    cart_id: cart_id.clone(),
                    idempotency_key: None,
                },
                StepFailureMode::None,
            )
            .await;

        assert!(
            resume_res.is_ok(),
            "Resumption from step {} failed: {:?}",
            crash_step,
            resume_res.err()
        );
        let output = resume_res.unwrap();
        assert_eq!(
            output.order.summary.as_ref().unwrap().current_order_total,
            dec!(160.00)
        );
    }
}
