//! Fulfillment entity models (DATA_MODEL §2.6, TASKS P5-07).
//! 12 entities in schema `fulfillment`.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

// 1. Fulfillment
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Fulfillment {
    pub id: Id<Fulfillment>,
    pub location_id: Option<String>,
    pub provider_id: String,
    pub shipping_option_id: Option<String>,
    pub delivery_address_id: Option<String>,
    pub packed_at: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 2. FulfillmentItem
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FulfillmentItem {
    pub id: Id<FulfillmentItem>,
    pub fulfillment_id: String,
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub line_item_id: String,
    pub inventory_item_id: Option<String>,
    pub quantity: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 3. FulfillmentLabel
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FulfillmentLabel {
    pub id: Id<FulfillmentLabel>,
    pub fulfillment_id: String,
    pub tracking_number: String,
    pub tracking_url: Option<String>,
    pub label_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 4. FulfillmentProvider
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FulfillmentProvider {
    pub id: Id<FulfillmentProvider>,
    pub name: String,
    pub is_enabled: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 5. FulfillmentSet
#[derive(Debug, Clone, Serialize)]
pub(crate) struct FulfillmentSet {
    pub id: Id<FulfillmentSet>,
    pub name: String,
    pub r#type: String, // "shipping", "pickup"
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 6. ServiceZone
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ServiceZone {
    pub id: Id<ServiceZone>,
    pub fulfillment_set_id: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 7. GeoZone
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GeoZone {
    pub id: Id<GeoZone>,
    pub service_zone_id: String,
    pub r#type: String, // "country", "province", "city", "zip"
    pub country_code: String,
    pub province_code: Option<String>,
    pub city: Option<String>,
    pub postal_expression: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 8. ShippingOption
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ShippingOption {
    pub id: Id<ShippingOption>,
    pub name: String,
    pub price_type: String, // "flat", "calculated"
    pub service_zone_id: String,
    pub shipping_profile_id: String,
    pub provider_id: String,
    pub shipping_option_type_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 9. ShippingOptionRule
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ShippingOptionRule {
    pub id: Id<ShippingOptionRule>,
    pub shipping_option_id: String,
    pub attribute: String, // "is_return", "customer_group_id", "weight", etc.
    pub operator: String,  // "in", "eq", "ne", "gt", "gte", "lt", "lte"
    pub value: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 10. ShippingOptionType
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ShippingOptionType {
    pub id: Id<ShippingOptionType>,
    pub label: String,
    pub description: Option<String>,
    pub code: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 11. ShippingProfile
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ShippingProfile {
    pub id: Id<ShippingProfile>,
    pub name: String,
    pub r#type: String, // "default", "custom", "gift_card"
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// 12. Address
#[derive(Debug, Clone, Serialize)]
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
