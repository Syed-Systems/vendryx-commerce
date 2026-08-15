//! StockLocationService domain service.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::StockLocation;

#[derive(Clone, Default)]
pub struct StockLocationService {
    locations: Arc<RwLock<HashMap<String, StockLocation>>>,
}

impl StockLocationService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_location(
        &self,
        input: CreateStockLocationInput,
    ) -> Result<StockLocationDTO> {
        let loc_id = Id::new();
        let loc = StockLocation {
            id: loc_id,
            name: input.name,
            address_id: input.address_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut map = self.locations.write().unwrap();
        map.insert(loc.id.to_string(), loc.clone());

        Ok(StockLocationDTO {
            id: Id::from_string(loc.id.to_string()).unwrap_or_default(),
            name: loc.name,
            address_id: loc.address_id,
            created_at: loc.created_at,
        })
    }

    pub async fn get_location(&self, id: &str) -> Result<StockLocationDTO> {
        let map = self.locations.read().unwrap();
        let loc = map
            .values()
            .find(|l| {
                (l.id.to_string() == id || l.id.to_string().ends_with(id)) && l.deleted_at.is_none()
            })
            .or_else(|| map.get(id).filter(|l| l.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "stock_location",
                id: id.to_string(),
            })?;

        Ok(StockLocationDTO {
            id: Id::from_string(loc.id.to_string()).unwrap_or_default(),
            name: loc.name.clone(),
            address_id: loc.address_id.clone(),
            created_at: loc.created_at,
        })
    }

    pub async fn list_locations(&self) -> Result<Vec<StockLocationDTO>> {
        let map = self.locations.read().unwrap();
        let list = map
            .values()
            .filter(|l| l.deleted_at.is_none())
            .map(|l| StockLocationDTO {
                id: Id::from_string(l.id.to_string()).unwrap_or_default(),
                name: l.name.clone(),
                address_id: l.address_id.clone(),
                created_at: l.created_at,
            })
            .collect();
        Ok(list)
    }

    pub async fn delete_location(&self, id: &str) -> Result<()> {
        let mut map = self.locations.write().unwrap();
        let loc = map
            .values_mut()
            .find(|l| {
                (l.id.to_string() == id || l.id.to_string().ends_with(id)) && l.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "stock_location",
                id: id.to_string(),
            })?;
        loc.deleted_at = Some(Utc::now());
        Ok(())
    }
}
