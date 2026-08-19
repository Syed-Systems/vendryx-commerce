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
async fn test_complete_cart_all_11_steps_happy_path() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
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

    // 1. Seed ready cart
    let cart = cart_service
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_alex".to_string()),
            sales_channel_id: Some("sc_web".to_string()),
            email: Some("alex@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Alex".to_string()),
                last_name: Some("Johnson".to_string()),
                address_1: Some("456 Pine St".to_string()),
                city: Some("Seattle".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("98101".to_string()),
                ..Default::default()
            }),
            billing_address: Some(CreateAddressInput {
                first_name: Some("Alex".to_string()),
                last_name: Some("Johnson".to_string()),
                address_1: Some("456 Pine St".to_string()),
                city: Some("Seattle".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("98101".to_string()),
                ..Default::default()
            }),
            items: Some(vec![CreateLineItemInput {
                title: "Waterproof Hiking Boots".to_string(),
                subtitle: Some("Size 10".to_string()),
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(180.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: Some("var_boot_10".to_string()),
                product_id: Some("prod_boot".to_string()),
                product_title: Some("Waterproof Hiking Boots".to_string()),
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: Some("hiking-boots".to_string()),
                variant_sku: Some("SKU-BOOT-10".to_string()),
                variant_barcode: None,
                variant_title: Some("Size 10".to_string()),
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
                name: "Expedited Air".to_string(),
                amount: dec!(20.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: Some("so_air".to_string()),
                data: None,
                metadata: None,
            },
        )
        .await
        .expect("shipping method added");

    // 2. Execute Complete Cart
    let output = engine
        .execute(
            CompleteCartInput {
                cart_id: cart.id.clone(),
                idempotency_key: Some("idem_cart_checkout_001".to_string()),
            },
            StepFailureMode::None,
        )
        .await
        .expect("complete-cart succeeded");

    assert_eq!(output.order.email, Some("alex@example.com".to_string()));
    assert_eq!(output.order.items.len(), 1);
    assert_eq!(
        output.order.summary.as_ref().unwrap().current_order_total,
        dec!(200.00)
    );
    assert_eq!(output.order.payment_status, "captured");

    // Verify cart marked completed
    let completed_cart = cart_service.get_cart(&cart.id).await.expect("cart fetched");
    assert!(completed_cart.completed_at.is_some());

    // Verify order_cart link
    let linked_carts = link_store.get_by_left("order_cart", &output.order.id);
    assert_eq!(linked_carts, vec![cart.id]);
}
