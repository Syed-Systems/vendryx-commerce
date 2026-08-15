//! GitHub OAuth2 Authentication Provider for Stheno (P7-04).

use async_trait::async_trait;
use std::sync::Arc;
use stheno_auth::{
    AuthIdentityDTO, AuthProviderService, AuthService, CreateIdentityInput, OAuthProfile,
};
use stheno_types::{Error, Result};

pub struct GitHubAuthProvider {
    auth_service: Arc<AuthService>,
    client_id: String,
    #[allow(dead_code)]
    client_secret: String,
}

impl GitHubAuthProvider {
    pub fn new(auth_service: Arc<AuthService>, client_id: &str, client_secret: &str) -> Self {
        Self {
            auth_service,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    pub fn parse_github_profile(&self, claims_or_body: &serde_json::Value) -> Result<OAuthProfile> {
        let id_str = claims_or_body
            .get("id")
            .and_then(|v| {
                if v.is_number() {
                    Some(v.to_string())
                } else {
                    v.as_str().map(|s| s.to_string())
                }
            })
            .or_else(|| {
                claims_or_body
                    .get("sub")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .ok_or_else(|| Error::Invalid {
                code: "missing_github_id",
                message: "GitHub profile missing user id".to_string(),
                field: Some("id".to_string()),
            })?;

        let email = claims_or_body
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("user@github.local");

        let name = claims_or_body
            .get("name")
            .or_else(|| claims_or_body.get("login"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let picture = claims_or_body
            .get("avatar_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(OAuthProfile {
            provider_user_id: id_str,
            email: email.to_string(),
            name,
            picture,
            email_verified: true,
            raw_attributes: claims_or_body.clone(),
        })
    }
}

#[async_trait]
impl AuthProviderService for GitHubAuthProvider {
    fn provider_id(&self) -> &str {
        "github"
    }

    async fn authenticate(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO> {
        let profile = self.parse_github_profile(credentials)?;

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
                    "github_client_id": self.client_id,
                    "email": profile.email,
                    "name": profile.name,
                    "avatar_url": profile.picture,
                })),
                app_metadata: None,
                user_metadata: credentials.get("user_metadata").cloned(),
            })
            .await;

        match identity_res {
            Ok(id_dto) => Ok(id_dto),
            Err(Error::Conflict { .. }) => {
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
