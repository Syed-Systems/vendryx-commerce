//! Region models (DATA_MODEL §1).
//! Strictly `pub(crate)` (AGENTS.md §2.9).
//!
//! These deserialize as well as serialize: `PgCrudRepository` reads rows back as
//! JSON via `row_to_json`, so an entity that cannot be deserialized cannot be
//! persisted.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Region {
    pub id: Id<Region>,
    pub name: String,
    pub currency_code: String,
    pub automatic_taxes: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Region {
    const TABLE: &'static str = "region.region";
    const ENTITY: &'static str = "region";

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
pub(crate) struct Country {
    pub id: Id<Country>,
    pub iso_2: String,
    pub iso_3: String,
    pub num_code: String,
    pub name: String,
    pub display_name: String,
    pub region_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Country {
    const TABLE: &'static str = "region.country";
    const ENTITY: &'static str = "country";

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
    Country => "country",
    Region => "reg",
}
