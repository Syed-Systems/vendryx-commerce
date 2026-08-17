//! Service layer for `stheno-store`, backed by Postgres (P12-12).

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Store, StoreCurrency, StoreLocale};

#[derive(Clone)]
pub struct StoreService {
    stores: PgCrudRepository<Store>,
    currencies: PgCrudRepository<StoreCurrency>,
    locales: PgCrudRepository<StoreLocale>,
}

impl StoreService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            stores: PgCrudRepository::new(pool.clone()),
            currencies: PgCrudRepository::new(pool.clone()),
            locales: PgCrudRepository::new(pool),
        }
    }

    pub async fn create_store(&self, input: CreateStoreInput) -> Result<StoreDTO> {
        let store_id: Id<Store> = Id::new();
        let store_id_str = store_id.to_string();
        let now = Utc::now();

        let store = Store {
            id: store_id,
            name: input.name,
            default_sales_channel_id: input.default_sales_channel_id,
            default_region_id: input.default_region_id,
            default_location_id: input.default_location_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.stores.create(None, store).await?;

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
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                });
            }
        } else {
            store_currencies.push(StoreCurrency {
                id: Id::new(),
                store_id: store_id_str.clone(),
                currency_code: default_curr.to_lowercase(),
                is_default: true,
                created_at: now,
                updated_at: now,
                deleted_at: None,
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
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                });
            }
        } else {
            store_locales.push(StoreLocale {
                id: Id::new(),
                store_id: store_id_str.clone(),
                locale: default_loc.to_lowercase(),
                is_default: true,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            });
        }

        if !store_currencies.is_empty() {
            self.currencies
                .create_many(None, store_currencies.clone())
                .await?;
        }

        if !store_locales.is_empty() {
            self.locales
                .create_many(None, store_locales.clone())
                .await?;
        }

        Ok(StoreDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            name: created.name,
            default_sales_channel_id: created.default_sales_channel_id,
            default_region_id: created.default_region_id,
            default_location_id: created.default_location_id,
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
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn get_store(&self, id: &str) -> Result<StoreDTO> {
        let store = self.stores.retrieve(None, id).await?;
        let store_id_str = store.id.to_string();

        let curr_filter = FilterSpec::new()
            .with_predicate("store_id", FilterOp::Eq(store_id_str.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let store_currencies = self.currencies.list(None, &curr_filter).await?;

        let loc_filter = FilterSpec::new()
            .with_predicate("store_id", FilterOp::Eq(store_id_str))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let store_locales = self.locales.list(None, &loc_filter).await?;

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

    pub async fn get_default_store(&self) -> Result<StoreDTO> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, 1);

        let stores = self.stores.list(None, &filter).await?;
        let store = stores.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "store",
            id: "default".to_string(),
        })?;

        self.get_store(&store.id.to_string()).await
    }

    pub async fn update_store(&self, id: &str, input: UpdateStoreInput) -> Result<StoreDTO> {
        let mut store = self.stores.retrieve(None, id).await?;

        if let Some(name) = input.name {
            store.name = name;
        }
        if input.default_sales_channel_id.is_some() {
            store.default_sales_channel_id = input.default_sales_channel_id;
        }
        if input.default_region_id.is_some() {
            store.default_region_id = input.default_region_id;
        }
        if input.default_location_id.is_some() {
            store.default_location_id = input.default_location_id;
        }
        store.updated_at = Utc::now();

        let store_id_str = store.id.to_string();
        self.stores.update(None, &store_id_str, store).await?;

        if let Some(currs) = input.supported_currencies {
            // Delete existing
            let curr_filter = FilterSpec::new()
                .with_predicate("store_id", FilterOp::Eq(store_id_str.clone()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 100);
            let existing_currs = self.currencies.list(None, &curr_filter).await?;
            for ec in existing_currs {
                self.currencies.delete(None, &ec.id.to_string()).await?;
            }

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
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                });
            }
            if !store_currencies.is_empty() {
                self.currencies.create_many(None, store_currencies).await?;
            }
        }

        if let Some(locs) = input.supported_locales {
            // Delete existing
            let loc_filter = FilterSpec::new()
                .with_predicate("store_id", FilterOp::Eq(store_id_str.clone()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 100);
            let existing_locs = self.locales.list(None, &loc_filter).await?;
            for el in existing_locs {
                self.locales.delete(None, &el.id.to_string()).await?;
            }

            let default_loc = input.default_locale.unwrap_or_else(|| "en".to_string());
            let mut store_locales = Vec::new();
            for l in locs {
                let is_default = l.to_lowercase() == default_loc.to_lowercase();
                store_locales.push(StoreLocale {
                    id: Id::new(),
                    store_id: store_id_str.clone(),
                    locale: l.to_lowercase(),
                    is_default,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                });
            }
            if !store_locales.is_empty() {
                self.locales.create_many(None, store_locales).await?;
            }
        }

        self.get_store(&store_id_str).await
    }

    pub async fn delete_store(&self, id: &str) -> Result<()> {
        let store = self.stores.retrieve(None, id).await?;
        self.stores.soft_delete(None, &store.id.to_string()).await
    }
}
