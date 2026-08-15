//! Internal models for `stheno-audit`. Entity structs do not implement deserialization (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub struct AuditLog {
    pub id: Id<AuditLog>,
    pub actor_id: String,
    pub actor_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<Value>,
    pub created_at: DateTime<Utc>,
}
