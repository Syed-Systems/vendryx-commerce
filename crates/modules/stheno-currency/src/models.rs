//! Internal entity models for `stheno-currency` (DATA_MODEL §1, AGENTS.md §2.9).
//! Strictly `pub(crate)` and does not deserialize.

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Currency {
    pub code: String,
    pub symbol: String,
    pub symbol_native: String,
    pub name: String,
    pub decimal_digits: i16,
    pub rounding: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
