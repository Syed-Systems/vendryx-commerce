//! Boundary Data Transfer Objects (DTOs) for `stheno-store`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreDTO {
    pub id: Id<StoreDTO>,
    pub name: String,
    pub default_sales_channel_id: Option<String>,
    pub default_region_id: Option<String>,
    pub default_location_id: Option<String>,
    pub supported_currencies: Vec<String>,
    pub default_currency_code: Option<String>,
    pub supported_locales: Vec<String>,
    pub default_locale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateStoreInput {
    pub name: String,
    pub default_sales_channel_id: Option<String>,
    pub default_region_id: Option<String>,
    pub default_location_id: Option<String>,
    pub supported_currencies: Option<Vec<String>>,
    pub default_currency_code: Option<String>,
    pub supported_locales: Option<Vec<String>>,
    pub default_locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateStoreInput {
    pub name: Option<String>,
    pub default_sales_channel_id: Option<String>,
    pub default_region_id: Option<String>,
    pub default_location_id: Option<String>,
    pub supported_currencies: Option<Vec<String>>,
    pub default_currency_code: Option<String>,
    pub supported_locales: Option<Vec<String>>,
    pub default_locale: Option<String>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    StoreDTO => "store",
}
