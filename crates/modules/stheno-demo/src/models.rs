//! Internal entity models for `stheno-demo` (AGENTS.md §2.9).
//! Strictly `pub(crate)` and does not deserialize.

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DemoItem {
    pub id: Id<DemoItem>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
