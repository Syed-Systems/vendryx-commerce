//! Service layer for `stheno-demo`.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::DemoItem;

#[derive(Clone, Default)]
pub struct DemoService {
    items: Arc<RwLock<HashMap<String, DemoItem>>>,
}

impl DemoService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create(&self, input: CreateDemoInput) -> Result<DemoDTO> {
        let id = Id::new();
        let item = DemoItem {
            id,
            name: input.name,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut map = self.items.write().unwrap();
        map.insert(item.id.to_string(), item.clone());

        Ok(DemoDTO {
            id: Id::from_string(item.id.to_string()).unwrap_or_default(),
            name: item.name,
            created_at: item.created_at,
        })
    }

    pub async fn get(&self, id: &str) -> Result<DemoDTO> {
        let map = self.items.read().unwrap();
        let item = map
            .values()
            .find(|i| {
                (i.id.to_string() == id || i.id.to_string().ends_with(id)) && i.deleted_at.is_none()
            })
            .or_else(|| map.get(id).filter(|i| i.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "demo",
                id: id.to_string(),
            })?;

        Ok(DemoDTO {
            id: Id::from_string(item.id.to_string()).unwrap_or_default(),
            name: item.name.clone(),
            created_at: item.created_at,
        })
    }
}
