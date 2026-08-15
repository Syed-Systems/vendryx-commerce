use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use stheno_cart::{
    calculate_cart_totals, calculate_line_item_totals, calculate_shipping_method_totals,
    LineItemAdjustmentDTO, LineItemDTO, LineItemTaxLineDTO, ShippingMethodDTO,
};

proptest! {
    #[test]
    fn test_totals_proptest_line_sums_equal_cart_totals_exactly(
        unit_price_cents in 100i64..100000i64,
        quantity in 1i32..20i32,
        discount_cents in 0i64..5000i64,
        tax_rate_bps in 0i64..3000i64, // 0% to 30%
        precision in 0u32..=3u32,
    ) {
        let unit_price = Decimal::new(unit_price_cents, 2);
        let discount = Decimal::new(discount_cents, 2);
        let tax_rate = Decimal::new(tax_rate_bps, 4);

        let item = LineItemDTO {
            id: "prop_item".to_string(),
            cart_id: "cart_1".to_string(),
            title: "Proptest Item".to_string(),
            subtitle: None,
            thumbnail: None,
            quantity,
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
            unit_price,
            compare_at_unit_price: None,
            adjustments: vec![LineItemAdjustmentDTO {
                id: "adj_prop".to_string(),
                item_id: "prop_item".to_string(),
                amount: discount,
                promotion_id: None,
                code: None,
                description: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }],
            tax_lines: vec![LineItemTaxLineDTO {
                id: "tax_prop".to_string(),
                item_id: "prop_item".to_string(),
                rate: tax_rate,
                code: "PROP_TAX".to_string(),
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

        let sm = ShippingMethodDTO {
            id: "sm_prop".to_string(),
            cart_id: "cart_1".to_string(),
            name: "Ship".to_string(),
            amount: dec!(10.00),
            is_tax_inclusive: false,
            shipping_option_id: None,
            data: None,
            adjustments: vec![],
            tax_lines: vec![],
            totals: None,
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let item_totals = calculate_line_item_totals(&item, precision);
        let sm_totals = calculate_shipping_method_totals(&sm, precision);
        let cart_totals = calculate_cart_totals(&[item], &[sm], &[], precision);

        // Invariant 1: Sum of item totals + sum of shipping totals == cart total
        prop_assert_eq!(item_totals.total + sm_totals.total, cart_totals.total);

        // Invariant 2: Sum of item subtotals + sum of shipping subtotals == cart subtotal
        prop_assert_eq!(item_totals.subtotal + sm_totals.subtotal, cart_totals.subtotal);

        // Invariant 3: Sum of item taxes + sum of shipping taxes == cart tax_total
        prop_assert_eq!(item_totals.tax_total + sm_totals.tax_total, cart_totals.tax_total);

        // Invariant 4: Sum of discounts == cart discount_total
        prop_assert_eq!(item_totals.discount_total + sm_totals.discount_total, cart_totals.discount_total);
    }
}
