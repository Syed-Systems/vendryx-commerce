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

/// Resolved branding for the current tenant (P26-03).
///
/// An absent row is not an error — all fields default to `None` and the
/// storefront substitutes the platform defaults.  The JSON shape is frozen
/// as a Store API contract.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrandingDTO {
    pub id: Option<String>,
    pub store_name: Option<String>,
    /// Public URL for the logo asset (resolved via the file module).
    pub logo_url: Option<String>,
    /// Public URL for the favicon asset.
    pub favicon_url: Option<String>,
    /// Key from the curated font set (P26-04), e.g. "inter", "noto_sans_bengali".
    pub font_key: Option<String>,
    /// CSS hex/rgba values that map to --brand-* custom properties (P26-05).
    pub color_primary: Option<String>,
    pub color_secondary: Option<String>,
    pub color_accent: Option<String>,
    pub color_background: Option<String>,
    pub color_surface: Option<String>,
    pub color_text: Option<String>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Admin write side — explicit allowlist (AGENTS.md §2.9).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertBrandingInput {
    pub store_name: Option<String>,
    /// A file module ID returned by the upload endpoint; resolved to a URL on read.
    pub logo_file_id: Option<String>,
    pub favicon_file_id: Option<String>,
    pub font_key: Option<String>,
    pub color_primary: Option<String>,
    pub color_secondary: Option<String>,
    pub color_accent: Option<String>,
    pub color_background: Option<String>,
    pub color_surface: Option<String>,
    pub color_text: Option<String>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    StoreDTO => "store",
}
