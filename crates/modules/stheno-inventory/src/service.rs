//! InventoryService with atomic reservations and oversell prevention.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{InventoryItem, InventoryLevel, ReservationItem};

#[derive(Clone, Default)]
pub struct InventoryService {
    items: Arc<Mutex<HashMap<String, InventoryItem>>>,
    levels: Arc<Mutex<HashMap<String, InventoryLevel>>>, // key: item_id::location_id
    reservations: Arc<Mutex<HashMap<String, ReservationItem>>>,
}

impl InventoryService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_item(&self, input: CreateInventoryItemInput) -> Result<InventoryItemDTO> {
        let item_id = Id::new();
        let item = InventoryItem {
            id: item_id,
            sku: input.sku,
            hs_code: None,
            origin_country: None,
            mid_code: None,
            material: None,
            requires_shipping: input.requires_shipping.unwrap_or(true),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut map = self.items.lock().unwrap();
        map.insert(item.id.to_string(), item.clone());

        Ok(InventoryItemDTO {
            id: Id::from_string(item.id.to_string()).unwrap_or_default(),
            sku: item.sku,
            requires_shipping: item.requires_shipping,
            created_at: item.created_at,
        })
    }

    pub async fn create_level(
        &self,
        input: CreateInventoryLevelInput,
    ) -> Result<InventoryLevelDTO> {
        let level_id = Id::new();
        let level_key = format!("{}::{}", input.inventory_item_id, input.location_id);

        let level = InventoryLevel {
            id: level_id,
            inventory_item_id: input.inventory_item_id.clone(),
            location_id: input.location_id.clone(),
            stocked_quantity: input.stocked_quantity,
            reserved_quantity: 0,
            incoming_quantity: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut map = self.levels.lock().unwrap();
        map.insert(level_key, level.clone());

        Ok(InventoryLevelDTO {
            id: Id::from_string(level.id.to_string()).unwrap_or_default(),
            inventory_item_id: level.inventory_item_id,
            location_id: level.location_id,
            stocked_quantity: level.stocked_quantity,
            reserved_quantity: level.reserved_quantity,
            available_quantity: level.stocked_quantity - level.reserved_quantity,
        })
    }

    /// Atomically reserves quantity against an inventory level (P2-12, P2-13)
    pub async fn reserve(&self, input: CreateReservationInput) -> Result<ReservationItemDTO> {
        let level_key = format!("{}::{}", input.inventory_item_id, input.location_id);

        let mut levels = self.levels.lock().unwrap();
        let level = levels.get_mut(&level_key).ok_or_else(|| Error::NotFound {
            entity: "inventory_level",
            id: level_key.clone(),
        })?;

        let available = level.stocked_quantity - level.reserved_quantity;
        if available < input.quantity {
            return Err(Error::Conflict {
                code: "insufficient_inventory",
                message: format!(
                    "Insufficient inventory: requested {}, available {}",
                    input.quantity, available
                ),
            });
        }

        // Atomically increment reserved quantity
        level.reserved_quantity += input.quantity;
        level.updated_at = Utc::now();

        let res_id = Id::new();
        let reservation = ReservationItem {
            id: res_id,
            line_item_id: input.line_item_id.clone(),
            inventory_item_id: input.inventory_item_id.clone(),
            location_id: input.location_id.clone(),
            quantity: input.quantity,
            description: input.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut res_map = self.reservations.lock().unwrap();
        res_map.insert(reservation.id.to_string(), reservation.clone());

        Ok(ReservationItemDTO {
            id: Id::from_string(reservation.id.to_string()).unwrap_or_default(),
            line_item_id: reservation.line_item_id,
            inventory_item_id: reservation.inventory_item_id,
            location_id: reservation.location_id,
            quantity: reservation.quantity,
        })
    }

    /// Releases a reservation upon cancellation
    pub async fn release_reservation(&self, reservation_id: &str) -> Result<()> {
        let mut res_map = self.reservations.lock().unwrap();
        let reservation = res_map
            .remove(reservation_id)
            .ok_or_else(|| Error::NotFound {
                entity: "reservation_item",
                id: reservation_id.to_string(),
            })?;

        let level_key = format!(
            "{}::{}",
            reservation.inventory_item_id, reservation.location_id
        );
        let mut levels = self.levels.lock().unwrap();
        if let Some(level) = levels.get_mut(&level_key) {
            level.reserved_quantity -= reservation.quantity;
            level.updated_at = Utc::now();
        }

        Ok(())
    }

    /// Fulfills a reservation by decrementing both stocked and reserved quantities
    pub async fn fulfill_reservation(&self, reservation_id: &str) -> Result<()> {
        let mut res_map = self.reservations.lock().unwrap();
        let reservation = res_map
            .remove(reservation_id)
            .ok_or_else(|| Error::NotFound {
                entity: "reservation_item",
                id: reservation_id.to_string(),
            })?;

        let level_key = format!(
            "{}::{}",
            reservation.inventory_item_id, reservation.location_id
        );
        let mut levels = self.levels.lock().unwrap();
        if let Some(level) = levels.get_mut(&level_key) {
            level.stocked_quantity -= reservation.quantity;
            level.reserved_quantity -= reservation.quantity;
            level.updated_at = Utc::now();
        }

        Ok(())
    }

    pub async fn get_available_quantity(&self, item_id: &str, location_id: &str) -> Result<i32> {
        let level_key = format!("{}::{}", item_id, location_id);
        let levels = self.levels.lock().unwrap();
        let level = levels.get(&level_key).ok_or_else(|| Error::NotFound {
            entity: "inventory_level",
            id: level_key,
        })?;
        Ok(level.stocked_quantity - level.reserved_quantity)
    }
}
