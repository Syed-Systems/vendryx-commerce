use rust_decimal_macros::dec;
use stheno_cart::{
    CartService, CreateAddressInput, CreateCartInput, CreateCreditLineInput,
    CreateLineItemAdjustmentInput, CreateLineItemInput, CreateLineItemTaxLineInput,
    CreateShippingMethodAdjustmentInput, CreateShippingMethodInput,
    CreateShippingMethodTaxLineInput,
};

#[tokio::test]
async fn test_cart_all_9_entities_lifecycle() {
    let service = CartService::new();

    // 1. Address & Cart
    let cart = service
        .create_cart(CreateCartInput {
            currency_code: "usd".to_string(),
            email: Some("test@example.com".to_string()),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Ada".to_string()),
                last_name: Some("Lovelace".to_string()),
                address_1: Some("123 Binary Way".to_string()),
                city: Some("London".to_string()),
                country_code: Some("gb".to_string()),
                postal_code: Some("SW1A 1AA".to_string()),
                ..Default::default()
            }),
            billing_address: Some(CreateAddressInput {
                first_name: Some("Ada".to_string()),
                last_name: Some("Lovelace".to_string()),
                address_1: Some("123 Binary Way".to_string()),
                city: Some("London".to_string()),
                country_code: Some("gb".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .expect("cart creation should succeed");

    assert!(cart.shipping_address.is_some());
    assert!(cart.billing_address.is_some());
    assert_eq!(cart.currency_code, "usd");
    assert!(!cart.cart_secret.is_empty());

    // 2. Line Item
    let item = service
        .add_line_item(
            &cart.id,
            CreateLineItemInput {
                title: "Hydration Flask".to_string(),
                quantity: 2,
                unit_price: dec!(25.00),
                product_title: Some("Hydration Flask".to_string()),
                variant_sku: Some("FLASK-001".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("line item creation should succeed");

    assert_eq!(item.quantity, 2);
    assert_eq!(item.unit_price, dec!(25.00));
    assert_eq!(item.variant_sku.as_deref(), Some("FLASK-001"));

    // 3. Line Item Adjustment
    let adj = service
        .add_line_item_adjustment(
            &item.id,
            CreateLineItemAdjustmentInput {
                amount: dec!(5.00),
                promotion_id: Some("promo_summer".to_string()),
                code: Some("SUMMER5".to_string()),
                description: Some("$5 discount".to_string()),
            },
        )
        .await
        .expect("line item adjustment should succeed");

    assert_eq!(adj.amount, dec!(5.00));

    // 4. Line Item Tax Line
    let tax_line = service
        .add_line_item_tax_line(
            &item.id,
            CreateLineItemTaxLineInput {
                rate: dec!(0.10),
                code: "VAT10".to_string(),
                tax_rate_id: Some("txr_10".to_string()),
                provider_id: Some("system".to_string()),
                description: Some("10% standard VAT".to_string()),
            },
        )
        .await
        .expect("tax line creation should succeed");

    assert_eq!(tax_line.rate, dec!(0.10));

    // 5. Shipping Method
    let sm = service
        .add_shipping_method(
            &cart.id,
            CreateShippingMethodInput {
                name: "Express Courier".to_string(),
                amount: dec!(15.00),
                is_tax_inclusive: Some(false),
                shipping_option_id: Some("so_express".to_string()),
                data: None,
                metadata: None,
            },
        )
        .await
        .expect("shipping method creation should succeed");

    assert_eq!(sm.amount, dec!(15.00));

    // 6. Shipping Method Adjustment
    let sm_adj = service
        .add_shipping_method_adjustment(
            &sm.id,
            CreateShippingMethodAdjustmentInput {
                amount: dec!(3.00),
                promotion_id: Some("promo_freeship".to_string()),
                code: Some("SHIP3".to_string()),
                description: Some("$3 off shipping".to_string()),
            },
        )
        .await
        .expect("shipping method adjustment should succeed");

    assert_eq!(sm_adj.amount, dec!(3.00));

    // 7. Shipping Method Tax Line
    let sm_tax = service
        .add_shipping_method_tax_line(
            &sm.id,
            CreateShippingMethodTaxLineInput {
                rate: dec!(0.10),
                code: "VAT10".to_string(),
                tax_rate_id: Some("txr_10".to_string()),
                provider_id: Some("system".to_string()),
                description: Some("10% VAT on shipping".to_string()),
            },
        )
        .await
        .expect("shipping tax line creation should succeed");

    assert_eq!(sm_tax.rate, dec!(0.10));

    // 8. Credit Line
    let credit = service
        .add_credit_line(
            &cart.id,
            CreateCreditLineInput {
                amount: dec!(10.00),
                reference: "gift_card".to_string(),
                reference_id: "gc_12345".to_string(),
                metadata: None,
            },
        )
        .await
        .expect("credit line creation should succeed");

    assert_eq!(credit.amount, dec!(10.00));

    // Fetch and verify hydrated cart with all 9 entities
    let hydrated = service
        .get_cart(&cart.id)
        .await
        .expect("cart retrieval should succeed");

    assert_eq!(hydrated.items.len(), 1);
    assert_eq!(hydrated.items[0].adjustments.len(), 1);
    assert_eq!(hydrated.items[0].tax_lines.len(), 1);
    assert_eq!(hydrated.shipping_methods.len(), 1);
    assert_eq!(hydrated.shipping_methods[0].adjustments.len(), 1);
    assert_eq!(hydrated.shipping_methods[0].tax_lines.len(), 1);
    assert_eq!(hydrated.credit_lines.len(), 1);
}
