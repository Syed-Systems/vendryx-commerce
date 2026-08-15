//! Sales channel entity models.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SalesChannel {
    pub id: Id<SalesChannel>,
    pub name: String,
    pub description: Option<String>,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
