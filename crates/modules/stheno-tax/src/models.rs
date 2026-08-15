//! Tax entity models.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use stheno_types::Id;

/// A tax region defines a geographic area for tax collection.
/// Regions form a hierarchy: a province-level region has a `parent_id`
/// pointing to its country-level region.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct TaxRegion {
    pub id: Id<TaxRegion>,
    pub country_code: String,
    pub province_code: Option<String>,
    pub parent_id: Option<String>, // Self-referencing for hierarchy
    pub provider_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A tax rate within a region.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct TaxRate {
    pub id: Id<TaxRate>,
    pub tax_region_id: String,
    pub rate: Decimal, // e.g. 0.07 = 7%
    pub code: String,  // e.g. "VAT", "GST", "STATE_TAX"
    pub name: String,
    pub is_default: bool,
    pub is_combinable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A rule that further constrains when a tax rate applies.
/// E.g. only for product_type_id = "ptid_digital_goods".
#[derive(Debug, Clone, Serialize)]
pub(crate) struct TaxRateRule {
    pub id: Id<TaxRateRule>,
    pub tax_rate_id: String,
    pub reference: String, // "product_id", "product_type_id", "shipping_option_id"
    pub reference_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A tax provider for external tax calculation (e.g. TaxJar, Avalara).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct TaxProvider {
    pub id: Id<TaxProvider>,
    pub is_installed: bool,
}
