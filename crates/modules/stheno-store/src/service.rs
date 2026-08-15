//! Service layer for `stheno-store` (TASKS.md P3-03).

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Store, StoreCurrency, StoreLocale};

#[derive(Clone, Default)]
pub struct StoreService {
    stores: Arc<RwLock<HashMap<String, Store>>>,
    currencies: Arc<RwLock<HashMap<String, Vec<StoreCurrency>>>>, // store_id -> currencies
    locales: Arc<RwLock<HashMap<String, Vec<StoreLocale>>>>,      // store_id -> locales
}

impl StoreService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_store(&self, input: CreateStoreInput) -> Result<StoreDTO> {
        let store_id: Id<Store> = Id::new();
        let store_id_str = store_id.to_string();

        let store = Store {
            id: store_id,
            name: input.name,
            default_sales_channel_id: input.default_sales_channel_id,
            default_region_id: input.default_region_id,
            default_location_id: input.default_location_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut store_currencies = Vec::new();
        let default_curr = input
            .default_currency_code
            .clone()
            .unwrap_or_else(|| "usd".to_string());
        if let Some(currs) = input.supported_currencies {
            for c in currs {
                let is_default = c.to_lowercase() == default_curr.to_lowercase();
                store_currencies.push(StoreCurrency {
                    id: Id::new(),
                    store_id: store_id_str.clone(),
                    currency_code: c.to_lowercase(),
                    is_default,
                });
            }
        } else {
            store_currencies.push(StoreCurrency {
                id: Id::new(),
                store_id: store_id_str.clone(),
                currency_code: default_curr.to_lowercase(),
                is_default: true,
            });
        }

        let mut store_locales = Vec::new();
        let default_loc = input
            .default_locale
            .clone()
            .unwrap_or_else(|| "en".to_string());
        if let Some(locs) = input.supported_locales {
            for l in locs {
                let is_default = l.to_lowercase() == default_loc.to_lowercase();
                store_locales.push(StoreLocale {
                    id: Id::new(),
                    store_id: store_id_str.clone(),
                    locale: l.to_lowercase(),
                    is_default,
                });
            }
        } else {
            store_locales.push(StoreLocale {
                id: Id::new(),
                store_id: store_id_str.clone(),
                locale: default_loc.to_lowercase(),
                is_default: true,
            });
        }

        let mut curr_map = self.currencies.write().unwrap();
        curr_map.insert(store_id_str.clone(), store_currencies.clone());

        let mut loc_map = self.locales.write().unwrap();
        loc_map.insert(store_id_str.clone(), store_locales.clone());

        let mut store_map = self.stores.write().unwrap();
        store_map.insert(store_id_str.clone(), store.clone());

        Ok(StoreDTO {
            id: Id::from_string(store.id.to_string()).unwrap_or_default(),
            name: store.name,
            default_sales_channel_id: store.default_sales_channel_id,
            default_region_id: store.default_region_id,
            default_location_id: store.default_location_id,
            supported_currencies: store_currencies
                .iter()
                .map(|c| c.currency_code.clone())
                .collect(),
            default_currency_code: store_currencies
                .iter()
                .find(|c| c.is_default)
                .map(|c| c.currency_code.clone()),
            supported_locales: store_locales.iter().map(|l| l.locale.clone()).collect(),
            default_locale: store_locales
                .iter()
                .find(|l| l.is_default)
                .map(|l| l.locale.clone()),
            created_at: store.created_at,
            updated_at: store.updated_at,
        })
    }

    pub async fn get_store(&self, id: &str) -> Result<StoreDTO> {
        let store_map = self.stores.read().unwrap();
        let s = store_map
            .values()
            .find(|s| {
                (s.id.to_string() == id || s.id.to_string().ends_with(id)) && s.deleted_at.is_none()
            })
            .or_else(|| store_map.get(id).filter(|s| s.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "store",
                id: id.to_string(),
            })?;

        let curr_map = self.currencies.read().unwrap();
        let store_currencies = curr_map.get(&s.id.to_string()).cloned().unwrap_or_default();

        let loc_map = self.locales.read().unwrap();
        let store_locales = loc_map.get(&s.id.to_string()).cloned().unwrap_or_default();

        Ok(StoreDTO {
            id: Id::from_string(s.id.to_string()).unwrap_or_default(),
            name: s.name.clone(),
            default_sales_channel_id: s.default_sales_channel_id.clone(),
            default_region_id: s.default_region_id.clone(),
            default_location_id: s.default_location_id.clone(),
            supported_currencies: store_currencies
                .iter()
                .map(|c| c.currency_code.clone())
                .collect(),
            default_currency_code: store_currencies
                .iter()
                .find(|c| c.is_default)
                .map(|c| c.currency_code.clone()),
            supported_locales: store_locales.iter().map(|l| l.locale.clone()).collect(),
            default_locale: store_locales
                .iter()
                .find(|l| l.is_default)
                .map(|l| l.locale.clone()),
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
    }

    pub async fn get_default_store(&self) -> Result<StoreDTO> {
        let default_id = {
            let store_map = self.stores.read().unwrap();
            store_map
                .values()
                .find(|s| s.deleted_at.is_none())
                .map(|s| s.id.to_string())
        };

        if let Some(id) = default_id {
            self.get_store(&id).await
        } else {
            Err(Error::NotFound {
                entity: "store",
                id: "default".to_string(),
            })
        }
    }

    pub async fn update_store(&self, id: &str, input: UpdateStoreInput) -> Result<StoreDTO> {
        let store_id_str = {
            let mut store_map = self.stores.write().unwrap();
            let s = store_map
                .values_mut()
                .find(|s| {
                    (s.id.to_string() == id || s.id.to_string().ends_with(id))
                        && s.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "store",
                    id: id.to_string(),
                })?;

            if let Some(name) = input.name {
                s.name = name;
            }
            if input.default_sales_channel_id.is_some() {
                s.default_sales_channel_id = input.default_sales_channel_id;
            }
            if input.default_region_id.is_some() {
                s.default_region_id = input.default_region_id;
            }
            if input.default_location_id.is_some() {
                s.default_location_id = input.default_location_id;
            }
            s.updated_at = Utc::now();
            let store_id_str = s.id.to_string();

            if let Some(currs) = input.supported_currencies {
                let default_curr = input
                    .default_currency_code
                    .unwrap_or_else(|| "usd".to_string());
                let mut store_currencies = Vec::new();
                for c in currs {
                    let is_default = c.to_lowercase() == default_curr.to_lowercase();
                    store_currencies.push(StoreCurrency {
                        id: Id::new(),
                        store_id: store_id_str.clone(),
                        currency_code: c.to_lowercase(),
                        is_default,
                    });
                }
                let mut curr_map = self.currencies.write().unwrap();
                curr_map.insert(store_id_str.clone(), store_currencies);
            }

            if let Some(locs) = input.supported_locales {
                let default_loc = input.default_locale.unwrap_or_else(|| "en".to_string());
                let mut store_locales = Vec::new();
                for l in locs {
                    let is_default = l.to_lowercase() == default_loc.to_lowercase();
                    store_locales.push(StoreLocale {
                        id: Id::new(),
                        store_id: store_id_str.clone(),
                        locale: l.to_lowercase(),
                        is_default,
                    });
                }
                let mut loc_map = self.locales.write().unwrap();
                loc_map.insert(store_id_str.clone(), store_locales);
            }

            store_id_str
        };

        self.get_store(&store_id_str).await
    }

    pub async fn delete_store(&self, id: &str) -> Result<()> {
        let mut store_map = self.stores.write().unwrap();
        let s = store_map
            .values_mut()
            .find(|s| {
                (s.id.to_string() == id || s.id.to_string().ends_with(id)) && s.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "store",
                id: id.to_string(),
            })?;

        s.deleted_at = Some(Utc::now());
        Ok(())
    }
}
