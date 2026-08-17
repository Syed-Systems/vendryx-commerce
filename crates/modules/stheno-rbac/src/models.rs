//! Internal entity models for stheno-rbac.
//!
//! Rule 2.9: Entities are strictly pub(crate).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Role {
    pub id: Id<Role>,
    pub name: String,
    pub description: Option<String>,
    pub is_system_role: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Role {
    const TABLE: &'static str = "rbac.role";
    const ENTITY: &'static str = "role";

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
pub(crate) struct RoleInheritance {
    pub parent_role: String,
    pub child_role: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for RoleInheritance {
    const TABLE: &'static str = "rbac.role_inheritance";
    const ENTITY: &'static str = "role_inheritance";
    const ID_COLUMN: &'static str = "parent_role";

    fn id(&self) -> String {
        self.parent_role.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Policy {
    pub id: Id<Policy>,
    pub role: String,
    pub resource: String, // e.g. "order:*", "product:read", "*"
    pub action: String,   // e.g. "read", "write", "delete", "*"
    pub effect: String,   // "allow", "deny"
    pub is_sensitive: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Policy {
    const TABLE: &'static str = "rbac.policy";
    const ENTITY: &'static str = "policy";

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
pub(crate) struct AuditRecord {
    pub id: Id<AuditRecord>,
    pub actor_id: String,
    pub role: String,
    pub action: String,
    pub resource: String,
    pub mfa_verified: bool,
    pub decision: String, // "allowed", "denied"
    pub timestamp: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for AuditRecord {
    const TABLE: &'static str = "rbac.audit_record";
    const ENTITY: &'static str = "audit_record";

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
    AuditRecord => "arec",
    Policy => "pol",
    Role => "role",
}
