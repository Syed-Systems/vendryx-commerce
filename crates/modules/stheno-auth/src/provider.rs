//! Trait definitions for Authentication Providers (Email/Password, OAuth, OIDC).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stheno_types::Result;

use crate::dto::AuthIdentityDTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProfile {
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
    pub raw_attributes: serde_json::Value,
}

#[async_trait]
pub trait AuthProviderService: Send + Sync {
    /// Identifier for the provider (e.g. "emailpass", "google", "github", "oidc").
    fn provider_id(&self) -> &str;

    /// Authenticate credentials or OAuth callback code / tokens.
    async fn authenticate(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO>;

    /// Register a new identity.
    async fn register(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO>;
}
