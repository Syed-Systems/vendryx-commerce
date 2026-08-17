//! Cart entity models (DATA_MODEL §2.3).
//! 9 entities: `cart`, `line_item`, `line_item_adjustment`, `line_item_tax_line`,
//! `shipping_method`, `shipping_method_adjustment`, `shipping_method_tax_line`,
//! `credit_line`, `address`.
//! Strictly `pub(crate)` (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Cart {
    pub id: Id<Cart>,
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub shipping_address_id: Option<String>,
    pub billing_address_id: Option<String>,
    pub cart_secret: String,
    pub metadata: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LineItem {
    pub id: Id<LineItem>,
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
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LineItemAdjustment {
    pub id: Id<LineItemAdjustment>,
    pub item_id: String,
    pub amount: Decimal,
    pub promotion_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LineItemTaxLine {
    pub id: Id<LineItemTaxLine>,
    pub item_id: String,
    pub rate: Decimal,
    pub code: String,
    pub tax_rate_id: Option<String>,
    pub provider_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShippingMethod {
    pub id: Id<ShippingMethod>,
    pub cart_id: String,
    pub name: String,
    pub amount: Decimal,
    pub is_tax_inclusive: bool,
    pub shipping_option_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShippingMethodAdjustment {
    pub id: Id<ShippingMethodAdjustment>,
    pub shipping_method_id: String,
    pub amount: Decimal,
    pub promotion_id: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShippingMethodTaxLine {
    pub id: Id<ShippingMethodTaxLine>,
    pub shipping_method_id: String,
    pub rate: Decimal,
    pub code: String,
    pub tax_rate_id: Option<String>,
    pub provider_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CreditLine {
    pub id: Id<CreditLine>,
    pub cart_id: String,
    pub amount: Decimal,
    pub reference: String,
    pub reference_id: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Address {
    pub id: Id<Address>,
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
    pub deleted_at: Option<DateTime<Utc>>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    Address => "addr",
    Cart => "cart",
    CreditLine => "cl",
    LineItem => "item",
    LineItemAdjustment => "lia",
    LineItemTaxLine => "lit",
    ShippingMethod => "sm",
    ShippingMethodAdjustment => "sma",
    ShippingMethodTaxLine => "smt",
}

macro_rules! impl_crud_entity {
    ($t:ident, $table:literal, $entity:literal) => {
        impl CrudEntity for $t {
            const TABLE: &'static str = $table;
            const ENTITY: &'static str = $entity;

            fn id(&self) -> String {
                self.id.to_string()
            }
            fn is_deleted(&self) -> bool {
                self.deleted_at.is_some()
            }
            fn set_deleted(&mut self, deleted: bool) {
                self.deleted_at = if deleted { Some(Utc::now()) } else { None };
            }
        }
    };
}

impl_crud_entity!(Cart, "cart.cart", "cart");
impl_crud_entity!(LineItem, "cart.line_item", "line_item");
impl_crud_entity!(
    LineItemAdjustment,
    "cart.line_item_adjustment",
    "line_item_adjustment"
);
impl_crud_entity!(
    LineItemTaxLine,
    "cart.line_item_tax_line",
    "line_item_tax_line"
);
impl_crud_entity!(ShippingMethod, "cart.shipping_method", "shipping_method");
impl_crud_entity!(
    ShippingMethodAdjustment,
    "cart.shipping_method_adjustment",
    "shipping_method_adjustment"
);
impl_crud_entity!(
    ShippingMethodTaxLine,
    "cart.shipping_method_tax_line",
    "shipping_method_tax_line"
);
impl_crud_entity!(CreditLine, "cart.credit_line", "credit_line");
impl_crud_entity!(Address, "cart.address", "address");
