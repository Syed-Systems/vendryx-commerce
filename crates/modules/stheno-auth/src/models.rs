//! Internal entity models for stheno-auth.
//!
//! Rule 2.9: Entities are strictly pub(crate) and never derive deserialization.

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AuthIdentity {
    pub id: Id<AuthIdentity>,
    pub provider_id: String,
    pub entity_id: String,
    pub password_hash: Option<String>,
    pub token_version: i32,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub provider_metadata: Option<serde_json::Value>,
    pub app_metadata: Option<serde_json::Value>,
    pub user_metadata: Option<serde_json::Value>,
    pub failed_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AuthProvider {
    pub id: String,
    pub name: String,
    pub is_enabled: bool,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AuthSession {
    pub id: Id<AuthSession>,
    pub identity_id: String,
    pub token_family_id: String,
    pub token_version: i32,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TokenFamily {
    pub id: Id<TokenFamily>,
    pub identity_id: String,
    pub latest_refresh_token_hash: String,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MfaFactor {
    pub id: Id<MfaFactor>,
    pub identity_id: String,
    pub factor_type: String,
    pub secret_encrypted: String,
    pub recovery_codes_hash: Vec<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PasswordResetToken {
    pub id: Id<PasswordResetToken>,
    pub identity_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
