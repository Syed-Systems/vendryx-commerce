//! Store entity models.
//! Strictly `pub(crate)` (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Store {
    pub id: Id<Store>,
    pub name: String,
    pub default_sales_channel_id: Option<String>,
    pub default_region_id: Option<String>,
    pub default_location_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Store {
    const TABLE: &'static str = "store.store";
    const ENTITY: &'static str = "store";

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
pub(crate) struct StoreCurrency {
    pub id: Id<StoreCurrency>,
    pub store_id: String,
    pub currency_code: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for StoreCurrency {
    const TABLE: &'static str = "store.store_currency";
    const ENTITY: &'static str = "store_currency";

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
pub(crate) struct StoreLocale {
    pub id: Id<StoreLocale>,
    pub store_id: String,
    pub locale: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for StoreLocale {
    const TABLE: &'static str = "store.store_locale";
    const ENTITY: &'static str = "store_locale";

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

/// Tenant-level branding configuration (P26-02).
/// One row per tenant; absent row yields defaults at the service layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TenantBranding {
    pub id: Id<TenantBranding>,
    pub store_name: Option<String>,
    pub logo_file_id: Option<String>,
    pub favicon_file_id: Option<String>,
    pub font_key: Option<String>,
    pub color_primary: Option<String>,
    pub color_secondary: Option<String>,
    pub color_accent: Option<String>,
    pub color_background: Option<String>,
    pub color_surface: Option<String>,
    pub color_text: Option<String>,
    pub storefront_archetype: Option<String>,
    pub product_card_style: Option<String>,
    pub product_image_aspect: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TenantBranding {
    const TABLE: &'static str = "store.tenant_branding";
    const ENTITY: &'static str = "tenant_branding";

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
    Store => "store",
    StoreCurrency => "stocur",
    StoreLocale => "stoloc",
    TenantBranding => "stobrd",
}
