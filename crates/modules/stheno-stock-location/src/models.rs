//! Stock location entity models.
//! Strictly `pub(crate)` (AGENTS.md §2.9).
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StockLocation {
    pub id: Id<StockLocation>,
    pub name: String,
    pub address_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for StockLocation {
    const TABLE: &'static str = "stock_location.stock_location";
    const ENTITY: &'static str = "stock_location";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for StockLocationAddress {
    const TABLE: &'static str = "stock_location.stock_location_address";
    const ENTITY: &'static str = "stock_location_address";

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
    StockLocation => "sloc",
    StockLocationAddress => "slocaddr",
}
