use rust_decimal_macros::dec;
use stheno_cart::{
    calculate_cart_totals, calculate_line_item_totals, CreditLineDTO, LineItemAdjustmentDTO,
    LineItemDTO, LineItemTaxLineDTO, ShippingMethodDTO, ShippingMethodTaxLineDTO,
};

#[test]
fn test_pure_line_item_totals_tax_exclusive() {
    let item = LineItemDTO {
        id: "item_1".to_string(),
        cart_id: "cart_1".to_string(),
        title: "Test Item".to_string(),
        subtitle: None,
        thumbnail: None,
        quantity: 2,
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
        requires_shipping: true,
        is_discountable: true,
        is_tax_inclusive: false,
        is_custom_price: false,
        unit_price: dec!(50.00),
        compare_at_unit_price: None,
        adjustments: vec![LineItemAdjustmentDTO {
            id: "adj_1".to_string(),
            item_id: "item_1".to_string(),
            amount: dec!(10.00),
            promotion_id: None,
            code: None,
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
        tax_lines: vec![LineItemTaxLineDTO {
            id: "tax_1".to_string(),
            item_id: "item_1".to_string(),
            rate: dec!(0.20), // 20%
            code: "VAT20".to_string(),
            tax_rate_id: None,
            provider_id: None,
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
        totals: None,
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let totals = calculate_line_item_totals(&item, 2);
    // Subtotal = 50 * 2 = 100
    assert_eq!(totals.subtotal, dec!(100.00));
    // Discount = 10
    assert_eq!(totals.discount_total, dec!(10.00));
    // Taxable = 100 - 10 = 90; Tax = 90 * 0.20 = 18.00
    assert_eq!(totals.tax_total, dec!(18.00));
    // Total = 90 + 18 = 108.00
    assert_eq!(totals.total, dec!(108.00));
}

#[test]
fn test_pure_line_item_totals_tax_inclusive() {
    let item = LineItemDTO {
        id: "item_inc".to_string(),
        cart_id: "cart_1".to_string(),
        title: "Inclusive Item".to_string(),
        subtitle: None,
        thumbnail: None,
        quantity: 1,
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
        requires_shipping: true,
        is_discountable: true,
        is_tax_inclusive: true,
        is_custom_price: false,
        unit_price: dec!(119.00),
        compare_at_unit_price: None,
        adjustments: vec![],
        tax_lines: vec![LineItemTaxLineDTO {
            id: "tax_inc".to_string(),
            item_id: "item_inc".to_string(),
            rate: dec!(0.19), // 19% VAT inclusive
            code: "VAT19".to_string(),
            tax_rate_id: None,
            provider_id: None,
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
        totals: None,
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let totals = calculate_line_item_totals(&item, 2);
    assert_eq!(totals.subtotal, dec!(119.00));
    // 119 / 1.19 = 100 net; tax = 19
    assert_eq!(totals.tax_total, dec!(19.00));
    assert_eq!(totals.total, dec!(119.00));
}

#[test]
fn test_full_cart_totals_calculation() {
    let items = vec![LineItemDTO {
        id: "i1".to_string(),
        cart_id: "c1".to_string(),
        title: "Item 1".to_string(),
        subtitle: None,
        thumbnail: None,
        quantity: 2,
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
        requires_shipping: true,
        is_discountable: true,
        is_tax_inclusive: false,
        is_custom_price: false,
        unit_price: dec!(20.00), // subtotal 40
        compare_at_unit_price: None,
        adjustments: vec![LineItemAdjustmentDTO {
            id: "a1".to_string(),
            item_id: "i1".to_string(),
            amount: dec!(5.00),
            promotion_id: None,
            code: None,
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
        tax_lines: vec![LineItemTaxLineDTO {
            id: "t1".to_string(),
            item_id: "i1".to_string(),
            rate: dec!(0.10), // 10% on (40-5) = 3.50
            code: "TAX10".to_string(),
            tax_rate_id: None,
            provider_id: None,
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
        totals: None,
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

    let shipping_methods = vec![ShippingMethodDTO {
        id: "sm1".to_string(),
        cart_id: "c1".to_string(),
        name: "Standard".to_string(),
        amount: dec!(10.00),
        is_tax_inclusive: false,
        shipping_option_id: None,
        data: None,
        adjustments: vec![],
        tax_lines: vec![ShippingMethodTaxLineDTO {
            id: "smt1".to_string(),
            shipping_method_id: "sm1".to_string(),
            rate: dec!(0.10), // 1.00
            code: "TAX10".to_string(),
            tax_rate_id: None,
            provider_id: None,
            description: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }],
        totals: None,
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

    let credit_lines = vec![CreditLineDTO {
        id: "cl1".to_string(),
        cart_id: "c1".to_string(),
        amount: dec!(15.00),
        reference: "store_credit".to_string(),
        reference_id: "sc_1".to_string(),
        metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

    let cart_totals = calculate_cart_totals(&items, &shipping_methods, &credit_lines, 2);

    // Item subtotal = 40, Shipping subtotal = 10 -> Subtotal = 50
    assert_eq!(cart_totals.item_subtotal, dec!(40.00));
    assert_eq!(cart_totals.shipping_subtotal, dec!(10.00));
    assert_eq!(cart_totals.subtotal, dec!(50.00));

    // Discount = 5.00
    assert_eq!(cart_totals.discount_total, dec!(5.00));

    // Item tax = 3.50, Shipping tax = 1.00 -> Total Tax = 4.50
    assert_eq!(cart_totals.tax_total, dec!(4.50));

    // Total = (40 - 5 + 3.50) + (10 + 1.00) = 38.50 + 11.00 = 49.50
    assert_eq!(cart_totals.total, dec!(49.50));

    // Credit = 15.00 -> Balance Due = 49.50 - 15.00 = 34.50
    assert_eq!(cart_totals.credit_total, dec!(15.00));
    assert_eq!(cart_totals.balance_due, dec!(34.50));
}
