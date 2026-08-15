//! Region models (DATA_MODEL §1).
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Region {
    pub id: Id<Region>,
    pub name: String,
    pub currency_code: String,
    pub automatic_taxes: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Country {
    pub id: Id<Country>,
    pub iso_2: String,
    pub iso_3: String,
    pub num_code: String,
    pub name: String,
    pub display_name: String,
    pub region_id: Option<String>,
}
