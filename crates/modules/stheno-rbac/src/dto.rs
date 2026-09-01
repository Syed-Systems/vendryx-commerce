//! Data Transfer Objects for stheno-rbac.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDTO {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system_role: bool,
    pub parent_roles: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDTO {
    pub id: String,
    pub role: String,
    pub resource: String,
    pub action: String,
    pub effect: String, // "allow", "deny"
    pub is_sensitive: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleInput {
    pub name: String,
    pub description: Option<String>,
    pub parent_roles: Option<Vec<String>>,
    pub is_system_role: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyInput {
    pub role: String,
    pub resource: String,
    pub action: String,
    pub effect: String, // "allow", "deny"
    pub is_sensitive: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatePermissionInput {
    pub roles: Vec<String>,
    pub resource: String,
    pub action: String,
    pub actor_id: Option<String>,
    pub mfa_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecision {
    pub is_allowed: bool,
    pub reason: String,
    pub requires_mfa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecordDTO {
    pub id: String,
    pub actor_id: String,
    pub role: String,
    pub action: String,
    pub resource: String,
    pub mfa_verified: bool,
    pub decision: String,
    pub timestamp: DateTime<Utc>,
}

/// Partial update for a role. Absent means unchanged.
///
/// `name` is not editable: it is the identifier every policy row and inheritance edge
/// refers to, so a rename is a migration across three tables, not a field edit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleInput {
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Partial update for a policy. Absent means unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyInput {
    pub role: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub effect: Option<String>,
    pub is_sensitive: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}
