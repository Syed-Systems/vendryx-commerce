//! Email & Password Authentication Provider for Stheno (P7-03).

use async_trait::async_trait;
use std::sync::Arc;
use stheno_auth::{
    AuthIdentityDTO, AuthProviderService, AuthService, AuthenticateInput, CreateIdentityInput,
};
use stheno_types::{Error, Result};

pub struct EmailPassAuthProvider {
    auth_service: Arc<AuthService>,
}

impl EmailPassAuthProvider {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }
}

#[async_trait]
impl AuthProviderService for EmailPassAuthProvider {
    fn provider_id(&self) -> &str {
        "emailpass"
    }

    async fn authenticate(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO> {
        let email = credentials
            .get("email")
            .or_else(|| credentials.get("entity_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_email",
                message: "Missing 'email' in credentials".to_string(),
                field: Some("email".to_string()),
            })?;

        let password = credentials
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_password",
                message: "Missing 'password' in credentials".to_string(),
                field: Some("password".to_string()),
            })?;

        let (identity, _) = self
            .auth_service
            .authenticate(AuthenticateInput {
                provider_id: self.provider_id().to_string(),
                entity_id: email.to_string(),
                password: Some(password.to_string()),
            })
            .await?;

        Ok(identity)
    }

    async fn register(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO> {
        let email = credentials
            .get("email")
            .or_else(|| credentials.get("entity_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_email",
                message: "Missing 'email' in registration payload".to_string(),
                field: Some("email".to_string()),
            })?;

        let password = credentials
            .get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let actor_id = credentials
            .get("actor_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let actor_type = credentials
            .get("actor_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        self.auth_service
            .create_identity(CreateIdentityInput {
                provider_id: self.provider_id().to_string(),
                entity_id: email.to_string(),
                password,
                actor_id,
                actor_type,
                provider_metadata: credentials.get("provider_metadata").cloned(),
                app_metadata: credentials.get("app_metadata").cloned(),
                user_metadata: credentials.get("user_metadata").cloned(),
            })
            .await
    }
}
