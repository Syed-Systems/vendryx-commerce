//! Internal entity models for `stheno-pricing` (TRD §4.1, DATA_MODEL.md).
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PriceSet {
    pub id: Id<PriceSet>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Price {
    pub id: Id<Price>,
    pub price_set_id: String,
    pub price_list_id: Option<String>,
    pub currency_code: String,
    pub amount: Decimal,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub rules_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PriceRule {
    pub id: Id<PriceRule>,
    pub price_id: String,
    pub attribute: String,
    pub value: String,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PriceList {
    pub id: Id<PriceList>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub r#type: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PriceListRule {
    pub id: Id<PriceListRule>,
    pub price_list_id: String,
    pub attribute: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PricePreference {
    pub id: Id<PricePreference>,
    pub attribute: String,
    pub value: String,
    pub is_tax_inclusive: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
