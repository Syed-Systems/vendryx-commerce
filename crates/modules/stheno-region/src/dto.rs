//! Public DTOs for `stheno-region`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDTO {
    pub id: Id<RegionDTO>,
    pub name: String,
    pub currency_code: String,
    pub automatic_taxes: bool,
    pub countries: Vec<CountryDTO>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryDTO {
    pub id: Id<CountryDTO>,
    pub iso_2: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRegionInput {
    pub name: String,
    pub currency_code: String,
    pub automatic_taxes: Option<bool>,
    pub countries: Vec<String>, // ISO-2 country codes
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRegionInput {
    pub name: Option<String>,
    pub currency_code: Option<String>,
    pub automatic_taxes: Option<bool>,
    pub countries: Option<Vec<String>>,
}
