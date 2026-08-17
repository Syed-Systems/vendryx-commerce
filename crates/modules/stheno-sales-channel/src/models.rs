//! Sales channel entity models.
//! Strictly `pub(crate)` (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SalesChannel {
    pub id: Id<SalesChannel>,
    pub name: String,
    pub description: Option<String>,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for SalesChannel {
    const TABLE: &'static str = "sales_channel.sales_channel";
    const ENTITY: &'static str = "sales_channel";

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
    SalesChannel => "sc",
}
