//! DTOs and pure totals engine for `stheno-cart` (TRD §4.1, DATA_MODEL §2.3).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

// ── Address ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressDTO {
    pub id: String,
    pub customer_id: Option<String>,
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateAddressInput {
    pub customer_id: Option<String>,
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAddressInput {
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ── Adjustments & Tax Lines ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemAdjustmentDTO {
    pub id: String,
    pub item_id: String,
    pub amount: Decimal,
    pub promotion_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLineItemAdjustmentInput {
    pub amount: Decimal,
    pub promotion_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemTaxLineDTO {
    pub id: String,
    pub item_id: String,
    pub rate: Decimal,
    pub code: String,
    pub tax_rate_id: Option<String>,
    pub provider_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLineItemTaxLineInput {
    pub rate: Decimal,
    pub code: String,
    pub tax_rate_id: Option<String>,
    pub provider_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethodAdjustmentDTO {
    pub id: String,
    pub shipping_method_id: String,
    pub amount: Decimal,
    pub promotion_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingMethodAdjustmentInput {
    pub amount: Decimal,
    pub promotion_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethodTaxLineDTO {
    pub id: String,
    pub shipping_method_id: String,
    pub rate: Decimal,
    pub code: String,
    pub tax_rate_id: Option<String>,
    pub provider_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingMethodTaxLineInput {
    pub rate: Decimal,
    pub code: String,
    pub tax_rate_id: Option<String>,
    pub provider_id: Option<String>,
    pub description: Option<String>,
}

// ── Line Items & Shipping Methods ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemDTO {
    pub id: String,
    pub cart_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub thumbnail: Option<String>,
    pub quantity: i32,
    pub variant_id: Option<String>,
    pub product_id: Option<String>,
    pub product_title: Option<String>,
    pub product_description: Option<String>,
    pub product_subtitle: Option<String>,
    pub product_type: Option<String>,
    pub product_collection: Option<String>,
    pub product_handle: Option<String>,
    pub variant_sku: Option<String>,
    pub variant_barcode: Option<String>,
    pub variant_title: Option<String>,
    pub variant_option_values: Option<serde_json::Value>,
    pub requires_shipping: bool,
    pub is_discountable: bool,
    pub is_tax_inclusive: bool,
    pub is_custom_price: bool,
    pub unit_price: Decimal,
    pub compare_at_unit_price: Option<Decimal>,
    pub adjustments: Vec<LineItemAdjustmentDTO>,
    pub tax_lines: Vec<LineItemTaxLineDTO>,
    pub totals: Option<LineItemTotalsDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateLineItemInput {
    pub title: String,
    pub subtitle: Option<String>,
    pub thumbnail: Option<String>,
    pub quantity: i32,
    pub variant_id: Option<String>,
    pub product_id: Option<String>,
    pub product_title: Option<String>,
    pub product_description: Option<String>,
    pub product_subtitle: Option<String>,
    pub product_type: Option<String>,
    pub product_collection: Option<String>,
    pub product_handle: Option<String>,
    pub variant_sku: Option<String>,
    pub variant_barcode: Option<String>,
    pub variant_title: Option<String>,
    pub variant_option_values: Option<serde_json::Value>,
    pub requires_shipping: Option<bool>,
    pub is_discountable: Option<bool>,
    pub is_tax_inclusive: Option<bool>,
    pub is_custom_price: Option<bool>,
    pub unit_price: Decimal,
    pub compare_at_unit_price: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateLineItemInput {
    pub title: Option<String>,
    pub quantity: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub compare_at_unit_price: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingMethodDTO {
    pub id: String,
    pub cart_id: String,
    pub name: String,
    pub amount: Decimal,
    pub is_tax_inclusive: bool,
    pub shipping_option_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub adjustments: Vec<ShippingMethodAdjustmentDTO>,
    pub tax_lines: Vec<ShippingMethodTaxLineDTO>,
    pub totals: Option<ShippingMethodTotalsDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingMethodInput {
    pub name: String,
    pub amount: Decimal,
    pub is_tax_inclusive: Option<bool>,
    pub shipping_option_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

// ── Credit Lines ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditLineDTO {
    pub id: String,
    pub cart_id: String,
    pub amount: Decimal,
    pub reference: String,
    pub reference_id: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreditLineInput {
    pub amount: Decimal,
    pub reference: String,
    pub reference_id: String,
    pub metadata: Option<serde_json::Value>,
}

// ── Cart ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartDTO {
    pub id: String,
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub shipping_address: Option<AddressDTO>,
    pub billing_address: Option<AddressDTO>,
    pub items: Vec<LineItemDTO>,
    pub shipping_methods: Vec<ShippingMethodDTO>,
    pub credit_lines: Vec<CreditLineDTO>,
    pub totals: CartTotalsDTO,
    pub cart_secret: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCartInput {
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub shipping_address: Option<CreateAddressInput>,
    pub billing_address: Option<CreateAddressInput>,
    pub items: Option<Vec<CreateLineItemInput>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCartInput {
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: Option<String>,
    pub shipping_address_id: Option<String>,
    pub billing_address_id: Option<String>,
    pub shipping_address: Option<CreateAddressInput>,
    pub billing_address: Option<CreateAddressInput>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

// ── Computed Totals (Pure Function, P4-02) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LineItemTotalsDTO {
    pub subtotal: Decimal,
    pub total: Decimal,
    pub original_total: Decimal,
    pub discount_total: Decimal,
    pub tax_total: Decimal,
    pub original_tax_total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ShippingMethodTotalsDTO {
    pub subtotal: Decimal,
    pub total: Decimal,
    pub original_total: Decimal,
    pub discount_total: Decimal,
    pub tax_total: Decimal,
    pub original_tax_total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CartTotalsDTO {
    pub item_subtotal: Decimal,
    pub item_total: Decimal,
    pub item_tax_total: Decimal,
    pub shipping_subtotal: Decimal,
    pub shipping_total: Decimal,
    pub shipping_tax_total: Decimal,
    pub discount_total: Decimal,
    pub tax_total: Decimal,
    pub subtotal: Decimal,
    pub total: Decimal,
    pub credit_total: Decimal,
    pub balance_due: Decimal,
}

/// Pure totals computation over a cart snapshot (P4-02, FR-PLUS-7).
/// Supports exact decimal precision (0, 2, 3 decimal places) and tax-inclusive/exclusive handling.
pub fn calculate_line_item_totals(item: &LineItemDTO, precision: u32) -> LineItemTotalsDTO {
    let qty = Decimal::from(item.quantity);
    let original_subtotal = (item.unit_price * qty).round_dp(precision);

    // Sum adjustments (discounts)
    let discount_total = item
        .adjustments
        .iter()
        .map(|a| a.amount)
        .sum::<Decimal>()
        .min(original_subtotal)
        .round_dp(precision);

    let subtotal_after_discount = (original_subtotal - discount_total).max(dec!(0));

    // Calculate taxes
    let mut tax_total = dec!(0);
    let mut original_tax_total = dec!(0);

    for tax_line in &item.tax_lines {
        if item.is_tax_inclusive {
            // Reverse calculate inclusive tax: subtotal - (subtotal / (1 + rate))
            let divisor = dec!(1) + tax_line.rate;
            let net = (subtotal_after_discount / divisor).round_dp(precision);
            let tax = subtotal_after_discount - net;
            tax_total += tax;

            let orig_net = (original_subtotal / divisor).round_dp(precision);
            let orig_tax = original_subtotal - orig_net;
            original_tax_total += orig_tax;
        } else {
            let tax = (subtotal_after_discount * tax_line.rate).round_dp(precision);
            tax_total += tax;

            let orig_tax = (original_subtotal * tax_line.rate).round_dp(precision);
            original_tax_total += orig_tax;
        }
    }

    let total = if item.is_tax_inclusive {
        subtotal_after_discount
    } else {
        subtotal_after_discount + tax_total
    };

    let original_total = if item.is_tax_inclusive {
        original_subtotal
    } else {
        original_subtotal + original_tax_total
    };

    LineItemTotalsDTO {
        subtotal: original_subtotal,
        total,
        original_total,
        discount_total,
        tax_total,
        original_tax_total,
    }
}

pub fn calculate_shipping_method_totals(
    method: &ShippingMethodDTO,
    precision: u32,
) -> ShippingMethodTotalsDTO {
    let original_subtotal = method.amount.round_dp(precision);

    let discount_total = method
        .adjustments
        .iter()
        .map(|a| a.amount)
        .sum::<Decimal>()
        .min(original_subtotal)
        .round_dp(precision);

    let subtotal_after_discount = (original_subtotal - discount_total).max(dec!(0));

    let mut tax_total = dec!(0);
    let mut original_tax_total = dec!(0);

    for tax_line in &method.tax_lines {
        if method.is_tax_inclusive {
            let divisor = dec!(1) + tax_line.rate;
            let net = (subtotal_after_discount / divisor).round_dp(precision);
            let tax = subtotal_after_discount - net;
            tax_total += tax;

            let orig_net = (original_subtotal / divisor).round_dp(precision);
            let orig_tax = original_subtotal - orig_net;
            original_tax_total += orig_tax;
        } else {
            let tax = (subtotal_after_discount * tax_line.rate).round_dp(precision);
            tax_total += tax;

            let orig_tax = (original_subtotal * tax_line.rate).round_dp(precision);
            original_tax_total += orig_tax;
        }
    }

    let total = if method.is_tax_inclusive {
        subtotal_after_discount
    } else {
        subtotal_after_discount + tax_total
    };

    let original_total = if method.is_tax_inclusive {
        original_subtotal
    } else {
        original_subtotal + original_tax_total
    };

    ShippingMethodTotalsDTO {
        subtotal: original_subtotal,
        total,
        original_total,
        discount_total,
        tax_total,
        original_tax_total,
    }
}

pub fn calculate_cart_totals(
    items: &[LineItemDTO],
    shipping_methods: &[ShippingMethodDTO],
    credit_lines: &[CreditLineDTO],
    precision: u32,
) -> CartTotalsDTO {
    let mut item_subtotal = dec!(0);
    let mut item_total = dec!(0);
    let mut item_tax_total = dec!(0);
    let mut item_discount_total = dec!(0);

    for item in items {
        let t = calculate_line_item_totals(item, precision);
        item_subtotal += t.subtotal;
        item_total += t.total;
        item_tax_total += t.tax_total;
        item_discount_total += t.discount_total;
    }

    let mut shipping_subtotal = dec!(0);
    let mut shipping_total = dec!(0);
    let mut shipping_tax_total = dec!(0);
    let mut shipping_discount_total = dec!(0);

    for sm in shipping_methods {
        let t = calculate_shipping_method_totals(sm, precision);
        shipping_subtotal += t.subtotal;
        shipping_total += t.total;
        shipping_tax_total += t.tax_total;
        shipping_discount_total += t.discount_total;
    }

    let discount_total = item_discount_total + shipping_discount_total;
    let tax_total = item_tax_total + shipping_tax_total;
    let subtotal = item_subtotal + shipping_subtotal;
    let total = item_total + shipping_total;

    let credit_total = credit_lines
        .iter()
        .map(|c| c.amount)
        .sum::<Decimal>()
        .round_dp(precision);

    let balance_due = (total - credit_total).max(dec!(0));

    CartTotalsDTO {
        item_subtotal,
        item_total,
        item_tax_total,
        shipping_subtotal,
        shipping_total,
        shipping_tax_total,
        discount_total,
        tax_total,
        subtotal,
        total,
        credit_total,
        balance_due,
    }
}
