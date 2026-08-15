//! Stock location entity models.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StockLocation {
    pub id: Id<StockLocation>,
    pub name: String,
    pub address_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct StockLocationAddress {
    pub id: Id<StockLocationAddress>,
    pub address_1: String,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: String,
    pub postal_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
