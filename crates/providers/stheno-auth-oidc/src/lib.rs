//! Clean-room OpenID Connect (OIDC) Authentication Provider (P7-05, ADR-008).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_auth::{
    AuthIdentityDTO, AuthProviderService, AuthService, CreateIdentityInput, OAuthProfile,
};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub jwks_uri: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub userinfo_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcIdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: serde_json::Value, // Can be string or array of strings
    pub exp: i64,
    pub iat: i64,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub nonce: Option<String>,
}

pub struct OidcAuthProvider {
    auth_service: Arc<AuthService>,
    config: OidcConfig,
}

impl OidcAuthProvider {
    pub fn new(auth_service: Arc<AuthService>, config: OidcConfig) -> Self {
        Self {
            auth_service,
            config,
        }
    }

    /// Verifies OIDC ID token claims against configured issuer and audience (ADR-008).
    pub fn verify_claims(
        &self,
        claims: &OidcIdTokenClaims,
        expected_nonce: Option<&str>,
    ) -> Result<OAuthProfile> {
        // 1. Issuer verification
        if claims.iss != self.config.issuer {
            return Err(Error::Unauthorized(format!(
                "OIDC issuer mismatch: expected '{}', got '{}'",
                self.config.issuer, claims.iss
            )));
        }

        // 2. Audience verification
        let aud_matches = match &claims.aud {
            serde_json::Value::String(s) => s == &self.config.client_id,
            serde_json::Value::Array(arr) => arr
                .iter()
                .any(|v| v.as_str() == Some(&self.config.client_id)),
            _ => false,
        };

        if !aud_matches {
            return Err(Error::Unauthorized(format!(
                "OIDC audience mismatch: client_id '{}' not found in aud",
                self.config.client_id
            )));
        }

        // 3. Expiration verification
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err(Error::Unauthorized("OIDC ID token is expired".to_string()));
        }

        // 4. Nonce verification if expected
        if let Some(expected) = expected_nonce {
            if claims.nonce.as_deref() != Some(expected) {
                return Err(Error::Unauthorized("OIDC nonce mismatch".to_string()));
            }
        }

        let email = claims
            .email
            .clone()
            .unwrap_or_else(|| format!("{}@oidc.local", claims.sub));

        Ok(OAuthProfile {
            provider_user_id: claims.sub.clone(),
            email,
            name: claims.name.clone(),
            picture: claims.picture.clone(),
            email_verified: claims.email_verified.unwrap_or(true),
            raw_attributes: serde_json::to_value(claims).unwrap_or_default(),
        })
    }
}

#[async_trait]
impl AuthProviderService for OidcAuthProvider {
    fn provider_id(&self) -> &str {
        "oidc"
    }

    async fn authenticate(&self, credentials: &serde_json::Value) -> Result<AuthIdentityDTO> {
        // Parse ID Token claims from JSON or decoded token
        let claims_val = credentials
            .get("id_token_claims")
            .or_else(|| credentials.get("claims"))
            .unwrap_or(credentials);

        let claims: OidcIdTokenClaims =
            serde_json::from_value(claims_val.clone()).map_err(|e| Error::Invalid {
                code: "invalid_oidc_claims",
                message: format!("Invalid OIDC claims structure: {e}"),
                field: Some("id_token_claims".to_string()),
            })?;

        let expected_nonce = credentials.get("nonce").and_then(|v| v.as_str());
        let profile = self.verify_claims(&claims, expected_nonce)?;

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
                    "issuer": self.config.issuer,
                    "client_id": self.config.client_id,
                    "email": profile.email,
                    "name": profile.name,
                    "picture": profile.picture,
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
