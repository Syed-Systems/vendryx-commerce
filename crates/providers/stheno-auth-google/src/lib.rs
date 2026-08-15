//! Google OAuth2 Authentication Provider for Stheno (P7-04).

use async_trait::async_trait;
use std::sync::Arc;
use stheno_auth::{
    AuthIdentityDTO, AuthProviderService, AuthService, CreateIdentityInput, OAuthProfile,
};
use stheno_types::{Error, Result};

pub struct GoogleAuthProvider {
    auth_service: Arc<AuthService>,
    client_id: String,
    #[allow(dead_code)]
    client_secret: String,
}

impl GoogleAuthProvider {
    pub fn new(auth_service: Arc<AuthService>, client_id: &str, client_secret: &str) -> Self {
        Self {
            auth_service,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    pub fn parse_google_profile(&self, claims_or_body: &serde_json::Value) -> Result<OAuthProfile> {
        let sub = claims_or_body
            .get("sub")
            .or_else(|| claims_or_body.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_google_sub",
                message: "Google profile missing 'sub' identifier".to_string(),
                field: Some("sub".to_string()),
            })?;

        let email = claims_or_body
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_google_email",
                message: "Google profile missing 'email'".to_string(),
                field: Some("email".to_string()),
            })?;

        let name = claims_or_body
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let picture = claims_or_body
            .get("picture")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let email_verified = claims_or_body
            .get("email_verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(OAuthProfile {
            provider_user_id: sub.to_string(),
            email: email.to_string(),
            name,
            picture,
            email_verified,
            raw_attributes: claims_or_body.clone(),
        })
    }
}

#[async_trait]
impl AuthProviderService for GoogleAuthProvider {
    fn provider_id(&self) -> &str {
        "google"
    }

    async fn authenticate(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO> {
        let profile = self.parse_google_profile(credentials)?;

        // Try finding existing identity or create one (auto-link by provider_id + entity_id)
        let identity_res = self
            .auth_service
            .create_identity(CreateIdentityInput {
                provider_id: self.provider_id().to_string(),
                entity_id: profile.provider_user_id.clone(),
                password: None,
                actor_id: credentials
                    .get("actor_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                actor_type: credentials
                    .get("actor_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                provider_metadata: Some(serde_json::json!({
                    "google_client_id": self.client_id,
                    "email": profile.email,
                    "name": profile.name,
                    "picture": profile.picture,
                    "email_verified": profile.email_verified,
                })),
                app_metadata: None,
                user_metadata: credentials.get("user_metadata").cloned(),
            })
            .await;

        match identity_res {
            Ok(id_dto) => Ok(id_dto),
            Err(Error::Conflict { .. }) => {
                // Identity already exists, fetch and return
                // In AuthService, identities are keyed by id, let's look up via authenticate or get
                let (auth_dto, _) = self
                    .auth_service
                    .authenticate(stheno_auth::AuthenticateInput {
                        provider_id: self.provider_id().to_string(),
                        entity_id: profile.provider_user_id,
                        password: None,
                    })
                    .await?;
                Ok(auth_dto)
            }
            Err(e) => Err(e),
        }
    }

    async fn register(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO> {
        self.authenticate(credentials).await
    }
}
