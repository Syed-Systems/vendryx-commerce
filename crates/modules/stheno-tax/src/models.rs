//! Tax entity models.
//! Strictly `pub(crate)` (AGENTS.md §2.9).
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

/// A tax region defines a geographic area for tax collection.
/// Regions form a hierarchy: a province-level region has a `parent_id`
/// pointing to its country-level region.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for TaxRegion {
    const TABLE: &'static str = "tax.tax_region";
    const ENTITY: &'static str = "tax_region";

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

/// A tax rate within a region.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for TaxRate {
    const TABLE: &'static str = "tax.tax_rate";
    const ENTITY: &'static str = "tax_rate";

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

/// A rule that further constrains when a tax rate applies.
/// E.g. only for product_type_id = "ptid_digital_goods".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaxRateRule {
    pub id: Id<TaxRateRule>,
    pub tax_rate_id: String,
    pub reference: String, // "product_id", "product_type_id", "shipping_option_id"
    pub reference_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TaxRateRule {
    const TABLE: &'static str = "tax.tax_rate_rule";
    const ENTITY: &'static str = "tax_rate_rule";

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

/// A tax provider for external tax calculation (e.g. TaxJar, Avalara).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TaxProvider {
    pub id: Id<TaxProvider>,
    pub is_installed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TaxProvider {
    const TABLE: &'static str = "tax.tax_provider";
    const ENTITY: &'static str = "tax_provider";

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

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    TaxProvider => "tp",
    TaxRate => "txr",
    TaxRateRule => "txrule",
    TaxRegion => "txreg",
}
