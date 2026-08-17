//! Internal entity models for `stheno-inventory` (TRD §4.1, DATA_MODEL.md).
//! Strictly `pub(crate)` (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for InventoryItem {
    const TABLE: &'static str = "inventory.inventory_item";
    const ENTITY: &'static str = "inventory_item";

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

impl CrudEntity for InventoryLevel {
    const TABLE: &'static str = "inventory.inventory_level";
    const ENTITY: &'static str = "inventory_level";

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

impl CrudEntity for ReservationItem {
    const TABLE: &'static str = "inventory.reservation_item";
    const ENTITY: &'static str = "reservation_item";

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
    InventoryItem => "iitem",
    InventoryLevel => "ilev",
    ReservationItem => "res",
}
