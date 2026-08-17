//! Public DTOs for `stheno-stock-location`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLocationDTO {
    pub id: Id<StockLocationDTO>,
    pub name: String,
    pub address_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockLocationInput {
    pub name: String,
    pub address_id: Option<String>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    StockLocationDTO => "sloc",
}
