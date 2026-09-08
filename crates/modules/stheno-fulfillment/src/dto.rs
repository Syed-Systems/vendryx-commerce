//! DTOs and input types for `stheno-fulfillment` (DATA_MODEL §2.6, TASKS P5-07..P5-13).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use crate::status::CourierStatus;

// ── 1. Fulfillment DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentDTO {
    pub id: String,
    pub location_id: Option<String>,
    pub provider_id: String,
    pub shipping_option_id: Option<String>,
    pub delivery_address: Option<AddressDTO>,
    pub items: Vec<FulfillmentItemDTO>,
    pub labels: Vec<FulfillmentLabelDTO>,
    pub packed_at: Option<DateTime<Utc>>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFulfillmentInput {
    pub location_id: Option<String>,
    pub provider_id: String,
    pub shipping_option_id: Option<String>,
    pub delivery_address: Option<CreateAddressInput>,
    pub items: Vec<CreateFulfillmentItemInput>,
    pub labels: Option<Vec<CreateFulfillmentLabelInput>>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentItemDTO {
    pub id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFulfillmentItemInput {
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub line_item_id: String,
    pub inventory_item_id: Option<String>,
    pub quantity: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentLabelDTO {
    pub id: String,
    pub fulfillment_id: String,
    pub tracking_number: String,
    pub tracking_url: Option<String>,
    pub label_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFulfillmentLabelInput {
    pub tracking_number: String,
    pub tracking_url: Option<String>,
    pub label_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ── 2. Fulfillment Set & Service Zone DTOs (P5-08) ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentSetDTO {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub service_zones: Vec<ServiceZoneDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFulfillmentSetInput {
    pub name: String,
    pub r#type: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceZoneDTO {
    pub id: String,
    pub fulfillment_set_id: String,
    pub name: String,
    pub geo_zones: Vec<GeoZoneDTO>,
    pub shipping_options: Vec<ShippingOptionDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceZoneInput {
    pub fulfillment_set_id: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoZoneDTO {
    pub id: String,
    pub service_zone_id: String,
    pub r#type: String,
    pub country_code: String,
    pub province_code: Option<String>,
    pub city: Option<String>,
    pub postal_expression: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGeoZoneInput {
    pub service_zone_id: String,
    pub r#type: String,
    pub country_code: String,
    pub province_code: Option<String>,
    pub city: Option<String>,
    pub postal_expression: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

// ── 3. Shipping Options, Rules, Types & Profiles (P5-09) ────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingOptionDTO {
    pub id: String,
    pub name: String,
    pub price_type: String, // "flat", "calculated"
    pub service_zone_id: String,
    pub shipping_profile_id: String,
    pub provider_id: String,
    pub shipping_option_type_id: Option<String>,
    pub rules: Vec<ShippingOptionRuleDTO>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingOptionInput {
    pub name: String,
    pub price_type: String,
    pub service_zone_id: String,
    pub shipping_profile_id: String,
    pub provider_id: String,
    pub shipping_option_type_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingOptionRuleDTO {
    pub id: String,
    pub shipping_option_id: String,
    pub attribute: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingOptionRuleInput {
    pub shipping_option_id: String,
    pub attribute: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShippingOptionFilter {
    pub service_zone_id: Option<String>,
    pub shipping_profile_id: Option<String>,
    pub is_return: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingOptionTypeDTO {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub code: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingOptionTypeInput {
    pub label: String,
    pub description: Option<String>,
    pub code: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingProfileDTO {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShippingProfileInput {
    pub name: String,
    pub r#type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Partial update for a shipping profile. Absent means unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShippingProfileInput {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ── 4. Eligibility Context & Calculation (P5-10, P5-12) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EligibilityContext {
    pub country_code: Option<String>,
    pub province_code: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub currency_code: Option<String>,
    pub is_return: Option<bool>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatePriceContext {
    pub items_count: usize,
    pub total_weight: Option<Decimal>,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentOptionDTO {
    pub id: String,
    pub is_return: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFulfillmentResponse {
    pub data: Option<serde_json::Value>,
    pub labels: Vec<CreateFulfillmentLabelInput>,
}

// ── 5. Address DTO ──────────────────────────────────────────────────────────

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentProviderRequestDTO {
    pub id: String,
    pub idempotency_key: String,
    pub provider_id: String,
    pub operation: String,
    pub resource_type: String,
    pub resource_id: String,
    pub state: String,
    pub response: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub attempts: i32,
    pub first_sent_at: DateTime<Utc>,
    pub settled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentWebhookEventDTO {
    pub id: String,
    pub provider_id: String,
    pub provider_event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub signature_verified: bool,
    pub provider_created_at: DateTime<Utc>,
    pub state: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}


