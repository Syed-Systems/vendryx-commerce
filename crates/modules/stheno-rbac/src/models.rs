//! Internal entity models for stheno-rbac.
//!
//! Rule 2.9: Entities are strictly pub(crate) and never derive deserialization.

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleInheritance {
    pub parent_role: String,
    pub child_role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AuditRecord {
    pub id: Id<AuditRecord>,
    pub actor_id: String,
    pub role: String,
    pub action: String,
    pub resource: String,
    pub mfa_verified: bool,
    pub decision: String, // "allowed", "denied"
    pub timestamp: DateTime<Utc>,
}
