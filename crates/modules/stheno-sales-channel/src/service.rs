//! SalesChannelService domain service.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::SalesChannel;

#[derive(Clone, Default)]
pub struct SalesChannelService {
    channels: Arc<RwLock<HashMap<String, SalesChannel>>>,
}

impl SalesChannelService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_channel(&self, input: CreateSalesChannelInput) -> Result<SalesChannelDTO> {
        let sc_id = Id::new();
        let sc = SalesChannel {
            id: sc_id,
            name: input.name,
            description: input.description,
            is_disabled: input.is_disabled.unwrap_or(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut map = self.channels.write().unwrap();
        map.insert(sc.id.to_string(), sc.clone());

        Ok(SalesChannelDTO {
            id: Id::from_string(sc.id.to_string()).unwrap_or_default(),
            name: sc.name,
            description: sc.description,
            is_disabled: sc.is_disabled,
            created_at: sc.created_at,
        })
    }

    pub async fn get_channel(&self, id: &str) -> Result<SalesChannelDTO> {
        let map = self.channels.read().unwrap();
        let sc = map
            .values()
            .find(|c| {
                (c.id.to_string() == id || c.id.to_string().ends_with(id)) && c.deleted_at.is_none()
            })
            .or_else(|| map.get(id).filter(|c| c.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "sales_channel",
                id: id.to_string(),
            })?;

        Ok(SalesChannelDTO {
            id: Id::from_string(sc.id.to_string()).unwrap_or_default(),
            name: sc.name.clone(),
            description: sc.description.clone(),
            is_disabled: sc.is_disabled,
            created_at: sc.created_at,
        })
    }

    pub async fn list_channels(&self) -> Result<Vec<SalesChannelDTO>> {
        let map = self.channels.read().unwrap();
        Ok(map
            .values()
            .filter(|c| c.deleted_at.is_none())
            .map(|sc| SalesChannelDTO {
                id: Id::from_string(sc.id.to_string()).unwrap_or_default(),
                name: sc.name.clone(),
                description: sc.description.clone(),
                is_disabled: sc.is_disabled,
                created_at: sc.created_at,
            })
            .collect())
    }

    pub async fn delete_channel(&self, id: &str) -> Result<()> {
        let mut map = self.channels.write().unwrap();
        let sc = map
            .values_mut()
            .find(|c| {
                (c.id.to_string() == id || c.id.to_string().ends_with(id)) && c.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "sales_channel",
                id: id.to_string(),
            })?;
        sc.deleted_at = Some(Utc::now());
        Ok(())
    }
}
