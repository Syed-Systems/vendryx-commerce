//! Public DTOs for `stheno-inventory`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemDTO {
    pub id: Id<InventoryItemDTO>,
    pub sku: Option<String>,
    pub requires_shipping: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInventoryItemInput {
    pub sku: Option<String>,
    pub requires_shipping: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLevelDTO {
    pub id: Id<InventoryLevelDTO>,
    pub inventory_item_id: String,
    pub location_id: String,
    pub stocked_quantity: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInventoryLevelInput {
    pub inventory_item_id: String,
    pub location_id: String,
    pub stocked_quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationItemDTO {
    pub id: Id<ReservationItemDTO>,
    pub line_item_id: Option<String>,
    pub inventory_item_id: String,
    pub location_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReservationInput {
    pub line_item_id: Option<String>,
    pub inventory_item_id: String,
    pub location_id: String,
    pub quantity: i32,
    pub description: Option<String>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    InventoryItemDTO => "iitem",
    InventoryLevelDTO => "ilev",
    ReservationItemDTO => "res",
}

/// Partial update for an inventory item. Absent means unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInventoryItemInput {
    pub sku: Option<String>,
    pub requires_shipping: Option<bool>,
}

/// Partial update for a reservation.
///
/// Only `quantity` and `description` are editable. Moving a reservation to a different
/// item or location is a release-and-reserve, not a field edit — it has to re-check
/// availability at the destination, which this shape cannot express.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReservationInput {
    pub quantity: Option<i32>,
    pub description: Option<String>,
}
