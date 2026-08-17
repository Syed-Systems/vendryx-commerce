//! Internal entity models for stheno-auth.
//!
//! Rule 2.9: Entities are strictly pub(crate).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for AuthIdentity {
    const TABLE: &'static str = "auth.auth_identity";
    const ENTITY: &'static str = "auth_identity";

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
pub(crate) struct AuthProvider {
    pub id: String,
    pub name: String,
    pub is_enabled: bool,
    pub config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for AuthProvider {
    const TABLE: &'static str = "auth.auth_provider";
    const ENTITY: &'static str = "auth_provider";

    fn id(&self) -> String {
        self.id.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for AuthSession {
    const TABLE: &'static str = "auth.auth_session";
    const ENTITY: &'static str = "auth_session";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some() || self.revoked_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
        if deleted {
            self.revoked_at = Some(Utc::now());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokenFamily {
    pub id: Id<TokenFamily>,
    pub identity_id: String,
    pub latest_refresh_token_hash: String,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TokenFamily {
    const TABLE: &'static str = "auth.token_family";
    const ENTITY: &'static str = "token_family";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some() || !self.is_active || self.revoked_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
        if deleted {
            self.is_active = false;
            self.revoked_at = Some(Utc::now());
        } else {
            self.is_active = true;
            self.revoked_at = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MfaFactor {
    pub id: Id<MfaFactor>,
    pub identity_id: String,
    pub factor_type: String,
    pub secret_encrypted: String,
    pub recovery_codes_hash: Vec<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for MfaFactor {
    const TABLE: &'static str = "auth.mfa_factor";
    const ENTITY: &'static str = "mfa_factor";

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
pub(crate) struct PasswordResetToken {
    pub id: Id<PasswordResetToken>,
    pub identity_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for PasswordResetToken {
    const TABLE: &'static str = "auth.password_reset_token";
    const ENTITY: &'static str = "password_reset_token";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some() || self.used_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
        if deleted {
            self.used_at = Some(Utc::now());
        }
    }
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    AuthIdentity => "authid",
    AuthSession => "ases",
    MfaFactor => "mfa",
    PasswordResetToken => "prt",
    TokenFamily => "tokfam",
}
