//! Boundary Data Transfer Objects (DTOs) for `stheno-pricing`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSetDTO {
    pub id: Id<PriceSetDTO>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceDTO {
    pub id: Id<PriceDTO>,
    pub price_set_id: String,
    pub price_list_id: Option<String>,
    pub currency_code: String,
    pub amount: Decimal,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub rules_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePriceSetInput {
    pub prices: Vec<CreatePriceInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePriceInput {
    pub currency_code: String,
    pub amount: Decimal,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub price_list_id: Option<String>,
    pub rules: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePriceListInput {
    pub title: String,
    pub description: Option<String>,
    pub r#type: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub rules: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceListDTO {
    pub id: Id<PriceListDTO>,
    pub title: String,
    pub r#type: String,
    pub status: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePriceSetInput {
    pub prices: Option<Vec<CreatePriceInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePriceListInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub r#type: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub rules: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchPriceListPricesInput {
    pub create: Option<Vec<CreatePriceInput>>,
    pub delete: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePricePreferenceInput {
    pub attribute: String,
    pub value: String,
    pub is_tax_inclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePreferenceDTO {
    pub id: Id<PricePreferenceDTO>,
    pub attribute: String,
    pub value: String,
    pub is_tax_inclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingContext {
    pub currency_code: String,
    pub region_id: Option<String>,
    pub customer_group_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub quantity: Option<i32>,
    pub as_of: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedPriceDTO {
    pub price_set_id: String,
    pub calculated_amount: Decimal,
    pub original_amount: Decimal,
    pub currency_code: String,
    pub is_tax_inclusive: bool,
    pub price_list_type: Option<String>,
    pub matched_rules_count: i32,
    pub min_quantity: Option<i32>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    PriceDTO => "price",
    PriceListDTO => "plist",
    PricePreferenceDTO => "pricepref",
    PriceSetDTO => "pset",
}
