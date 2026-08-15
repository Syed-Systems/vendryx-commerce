//! Store entity models.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StoreCurrency {
    pub id: Id<StoreCurrency>,
    pub store_id: String,
    pub currency_code: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StoreLocale {
    pub id: Id<StoreLocale>,
    pub store_id: String,
    pub locale: String,
    pub is_default: bool,
}
