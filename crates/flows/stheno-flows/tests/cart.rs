use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{CartService, CreateCartInput, CreateLineItemInput, CreateShippingMethodInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::CartWorkflows;

#[tokio::test]
async fn test_cart_workflows_lifecycle_and_line_items() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let cart_service = Arc::new(CartService::new(pool));
    let workflows = CartWorkflows::new(cart_service);

    // 1. Create Cart Workflow
    let cart = workflows
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: None,
            sales_channel_id: Some("sc_default".to_string()),
            currency_code: "usd".to_string(),
            email: Some("shopper@example.com".to_string()),
            shipping_address: None,
            billing_address: None,
            items: None,
            metadata: None,
        })
        .await
        .expect("cart created");

    assert_eq!(cart.currency_code, "usd");
    assert!(!cart.cart_secret.is_empty());

    // 2. Add Line Item Workflow (denormalized product fields)
    let item = workflows
        .add_line_item(
            &cart.id,
            CreateLineItemInput {
                title: "Silk Scarf".to_string(),
                subtitle: Some("Blue Pattern".to_string()),
                thumbnail: None,
                quantity: 2,
                unit_price: dec!(45.00),
                variant_id: Some("var_scarf_1".to_string()),
                product_id: Some("prod_scarf".to_string()),
                product_title: Some("Silk Scarf".to_string()),
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: Some("silk-scarf".to_string()),
                variant_sku: Some("SKU-SCARF-BLU".to_string()),
                variant_barcode: None,
                variant_title: Some("Blue".to_string()),
                variant_option_values: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_tax_inclusive: Some(false),
                is_custom_price: Some(false),
                compare_at_unit_price: None,
                metadata: None,
            },
        )
        .await
        .expect("item added");

    assert_eq!(item.quantity, 2);
    assert_eq!(item.unit_price, dec!(45.00));

    // 3. Add Shipping Method Workflow
    let sm = workflows
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
        .expect("shipping method added");

    assert_eq!(sm.amount, dec!(10.00));

    // 4. Get and verify pure totals recomputed over snapshot graph
    let refreshed = workflows.get_cart(&cart.id).await.expect("cart refreshed");
    assert_eq!(refreshed.items.len(), 1);
    assert_eq!(refreshed.shipping_methods.len(), 1);
    assert_eq!(refreshed.totals.item_subtotal, dec!(90.00)); // 2 * 45
    assert_eq!(refreshed.totals.shipping_subtotal, dec!(10.00));
    assert_eq!(refreshed.totals.total, dec!(100.00));
}
