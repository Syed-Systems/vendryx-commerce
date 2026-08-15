//! Boundary DTOs for `stheno-tax`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRegionDTO {
    pub id: Id<TaxRegionDTO>,
    pub country_code: String,
    pub province_code: Option<String>,
    pub parent_id: Option<String>,
    pub provider_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRateDTO {
    pub id: Id<TaxRateDTO>,
    pub tax_region_id: String,
    pub rate: Decimal,
    pub code: String,
    pub name: String,
    pub is_default: bool,
    pub is_combinable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRateRuleDTO {
    pub id: Id<TaxRateRuleDTO>,
    pub tax_rate_id: String,
    pub reference: String,
    pub reference_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateTaxRegionInput {
    pub country_code: String,
    pub province_code: Option<String>,
    pub parent_id: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaxRateInput {
    pub tax_region_id: String,
    pub rate: Decimal,
    pub code: String,
    pub name: String,
    pub is_default: Option<bool>,
    pub is_combinable: Option<bool>,
    pub rules: Option<Vec<CreateTaxRateRuleInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaxRateRuleInput {
    pub reference: String,
    pub reference_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTaxRegionInput {
    pub country_code: Option<String>,
    pub province_code: Option<String>,
    pub parent_id: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTaxRateInput {
    pub rate: Option<Decimal>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_default: Option<bool>,
    pub is_combinable: Option<bool>,
}

/// Resolved tax line for a given item, after rate resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLine {
    pub rate: Decimal,
    pub code: String,
    pub name: String,
    pub tax_rate_id: String,
    pub provider_id: Option<String>,
}

/// Context for tax resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxContext {
    pub country_code: String,
    pub province_code: Option<String>,
    pub product_id: Option<String>,
    pub product_type_id: Option<String>,
    pub shipping_option_id: Option<String>,
    pub is_tax_inclusive: bool,
}

/// Result of tax-inclusive reverse computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInclusiveBreakdown {
    pub subtotal: Decimal,
    pub tax_total: Decimal,
    pub total: Decimal, // = subtotal + tax_total
    pub rate: Decimal,
}
