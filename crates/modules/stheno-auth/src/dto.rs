//! Data Transfer Objects for stheno-auth.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthIdentityDTO {
    pub id: String,
    pub provider_id: String,
    pub entity_id: String,
    pub token_version: i32,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub provider_metadata: Option<serde_json::Value>,
    pub app_metadata: Option<serde_json::Value>,
    pub user_metadata: Option<serde_json::Value>,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProviderDTO {
    pub id: String,
    pub name: String,
    pub is_enabled: bool,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionDTO {
    pub id: String,
    pub identity_id: String,
    pub token_family_id: String,
    pub token_version: i32,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenFamilyDTO {
    pub id: String,
    pub identity_id: String,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactorDTO {
    pub id: String,
    pub identity_id: String,
    pub factor_type: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetTokenDTO {
    pub id: String,
    pub identity_id: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityInput {
    pub provider_id: String,
    pub entity_id: String,
    pub password: Option<String>,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub provider_metadata: Option<serde_json::Value>,
    pub app_metadata: Option<serde_json::Value>,
    pub user_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateInput {
    pub provider_id: String,
    pub entity_id: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokensDTO {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub token_version: i32,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollmentDTO {
    pub factor_id: String,
    pub secret: String,
    pub uri: String,
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMfaInput {
    pub factor_id: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequestInput {
    pub entity_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmInput {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordInput {
    pub identity_id: String,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachedPasswordCheckResult {
    pub is_breached: bool,
    pub breach_count: u64,
}
