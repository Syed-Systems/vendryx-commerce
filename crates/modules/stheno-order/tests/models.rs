use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_order::*;

#[tokio::test]
async fn test_order_all_23_entities_lifecycle() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = OrderService::new(pool);

    // 1. Create order with shipping address, billing address, line items, and shipping methods (version 1)
    let order = service
        .create_order(CreateOrderInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_123".to_string()),
            sales_channel_id: Some("sc_main".to_string()),
            email: Some("buyer@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Alice".to_string()),
                last_name: Some("Smith".to_string()),
                address_1: Some("123 Market St".to_string()),
                city: Some("San Francisco".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("94105".to_string()),
                ..Default::default()
            }),
            billing_address: Some(CreateAddressInput {
                first_name: Some("Alice".to_string()),
                last_name: Some("Smith".to_string()),
                address_1: Some("123 Market St".to_string()),
                city: Some("San Francisco".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("94105".to_string()),
                ..Default::default()
            }),
            items: Some(vec![CreateLineItemInput {
                title: "Leather Handbag".to_string(),
                subtitle: Some("Black".to_string()),
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(220.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: Some("var_bag_blk".to_string()),
                product_id: Some("prod_bag".to_string()),
                product_title: Some("Leather Handbag".to_string()),
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: Some("leather-handbag".to_string()),
                variant_sku: Some("SKU-BAG-BLK".to_string()),
                variant_barcode: None,
                variant_title: Some("Black".to_string()),
                variant_option_values: None,
                metadata: None,
            }]),
            shipping_methods: Some(vec![CreateShippingMethodInput {
                name: "Standard Ground".to_string(),
                description: Some("3-5 business days".to_string()),
                amount: dec!(15.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: Some("so_ground".to_string()),
                data: None,
                metadata: None,
            }]),
            is_draft_order: Some(false),
            metadata: None,
        })
        .await
        .expect("order created");

    assert_eq!(order.version, 1);
    assert_eq!(order.items.len(), 1);
    assert_eq!(order.shipping_methods.len(), 1);
    assert_eq!(
        order.summary.as_ref().unwrap().current_order_total,
        dec!(235.00)
    );
}
