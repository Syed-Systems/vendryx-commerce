//! Domain events emitted by stheno-auth.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCreated {
    pub identity_id: String,
    pub provider_id: String,
    pub entity_id: String,
    pub actor_id: Option<String>,
    pub actor_type: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAuthenticated {
    pub identity_id: String,
    pub provider_id: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChanged {
    pub identity_id: String,
    pub new_token_version: i32,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenVersionInvalidated {
    pub identity_id: String,
    pub new_token_version: i32,
    pub reason: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactorAdded {
    pub identity_id: String,
    pub factor_id: String,
    pub factor_type: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactorVerified {
    pub identity_id: String,
    pub factor_id: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutTriggered {
    pub identity_id: String,
    pub locked_until: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
}
