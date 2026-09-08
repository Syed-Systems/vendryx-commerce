//! Service layer for `stheno-store`, backed by Postgres (P12-12).

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Store, StoreCurrency, StoreLocale, TenantBranding};

#[derive(Clone)]
pub struct StoreService {
    stores: PgCrudRepository<Store>,
    currencies: PgCrudRepository<StoreCurrency>,
    locales: PgCrudRepository<StoreLocale>,
    branding: PgCrudRepository<TenantBranding>,
}

impl StoreService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            stores: PgCrudRepository::new(pool.clone()),
            currencies: PgCrudRepository::new(pool.clone()),
            locales: PgCrudRepository::new(pool.clone()),
            branding: PgCrudRepository::new(pool),
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

    // ── Branding (P26-02, P26-03) ──────────────────────────────────────────

    /// Return the resolved branding for the current tenant.
    ///
    /// An absent row is not an error: we return `BrandingDTO::default()` so the
    /// storefront can substitute platform defaults without special-casing the caller.
    /// File IDs are not resolved to URLs here — the API layer queries the file
    /// module and injects `logo_url` / `favicon_url` before serialising.
    pub async fn get_branding(&self) -> Result<BrandingDTO> {
        let rows = self
            .branding
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1),
            )
            .await?;

        match rows.into_iter().next() {
            None => Ok(BrandingDTO::default()),
            Some(b) => Ok(Self::to_branding_dto(&b, None, None)),
        }
    }

    /// Upsert the tenant's branding row (insert on first call, update thereafter).
    ///
    /// Idempotent: calling twice with the same input produces the same row.
    pub async fn upsert_branding(&self, input: UpsertBrandingInput) -> Result<BrandingDTO> {
        let rows = self
            .branding
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1),
            )
            .await?;

        let now = Utc::now();
        let row = if let Some(mut existing) = rows.into_iter().next() {
            // Update existing row in place.
            if input.store_name.is_some() { existing.store_name = input.store_name; }
            if input.logo_file_id.is_some() { existing.logo_file_id = input.logo_file_id; }
            if input.favicon_file_id.is_some() { existing.favicon_file_id = input.favicon_file_id; }
            if input.font_key.is_some() { existing.font_key = input.font_key; }
            if input.color_primary.is_some() { existing.color_primary = input.color_primary; }
            if input.color_secondary.is_some() { existing.color_secondary = input.color_secondary; }
            if input.color_accent.is_some() { existing.color_accent = input.color_accent; }
            if input.color_background.is_some() { existing.color_background = input.color_background; }
            if input.color_surface.is_some() { existing.color_surface = input.color_surface; }
            if input.color_text.is_some() { existing.color_text = input.color_text; }
            existing.updated_at = now;
            self.branding.update(None, &existing.id.to_string(), existing).await?
        } else {
            // Create on first upsert.
            let new_row = TenantBranding {
                id: Id::new(),
                store_name: input.store_name,
                logo_file_id: input.logo_file_id,
                favicon_file_id: input.favicon_file_id,
                font_key: input.font_key,
                color_primary: input.color_primary,
                color_secondary: input.color_secondary,
                color_accent: input.color_accent,
                color_background: input.color_background,
                color_surface: input.color_surface,
                color_text: input.color_text,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.branding.create(None, new_row).await?
        };

        Ok(Self::to_branding_dto(&row, None, None))
    }

    fn to_branding_dto(
        b: &TenantBranding,
        logo_url: Option<String>,
        favicon_url: Option<String>,
    ) -> BrandingDTO {
        BrandingDTO {
            id: Some(b.id.to_string()),
            store_name: b.store_name.clone(),
            logo_url: logo_url.or_else(|| b.logo_file_id.clone()),
            favicon_url: favicon_url.or_else(|| b.favicon_file_id.clone()),
            font_key: b.font_key.clone(),
            color_primary: b.color_primary.clone(),
            color_secondary: b.color_secondary.clone(),
            color_accent: b.color_accent.clone(),
            color_background: b.color_background.clone(),
            color_surface: b.color_surface.clone(),
            color_text: b.color_text.clone(),
            updated_at: Some(b.updated_at),
        }
    }
}
