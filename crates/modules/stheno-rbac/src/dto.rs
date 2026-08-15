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
