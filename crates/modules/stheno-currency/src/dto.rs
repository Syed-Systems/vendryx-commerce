//! Boundary Data Transfer Objects (DTOs) for `stheno-currency`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyDTO {
    pub code: String,
    pub symbol: String,
    pub symbol_native: String,
    pub name: String,
    pub decimal_digits: i16,
    pub rounding: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
