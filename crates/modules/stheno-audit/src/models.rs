//! Internal models for `stheno-audit`. Entity structs do not implement deserialization (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuditLog {
    pub id: Id<AuditLog>,
    pub actor_id: String,
    pub actor_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for AuditLog {
    const TABLE: &'static str = "audit.audit_log";
    const ENTITY: &'static str = "audit_log";

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
    AuditLog => "alog",
}
