//! Internal entity models for `stheno-inventory` (TRD §4.1, DATA_MODEL.md).
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InventoryItem {
    pub id: Id<InventoryItem>,
    pub sku: Option<String>,
    pub hs_code: Option<String>,
    pub origin_country: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub requires_shipping: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InventoryLevel {
    pub id: Id<InventoryLevel>,
    pub inventory_item_id: String,
    pub location_id: String,
    pub stocked_quantity: i32,
    pub reserved_quantity: i32,
    pub incoming_quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReservationItem {
    pub id: Id<ReservationItem>,
    pub line_item_id: Option<String>,
    pub inventory_item_id: String,
    pub location_id: String,
    pub quantity: i32,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
