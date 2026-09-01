//! Auth workflows (WORKFLOWS §2, TASKS P7-21).

use std::sync::Arc;
use stheno_auth::*;
use stheno_types::Result;

pub struct AuthWorkflows {
    auth_service: Arc<AuthService>,
}

impl AuthWorkflows {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    // 1. authenticate_workflow
    pub async fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> Result<(AuthIdentityDTO, AuthTokensDTO)> {
        self.auth_service.authenticate(input).await
    }

    // 2. create_identity_workflow
    pub async fn create_identity(&self, input: CreateIdentityInput) -> Result<AuthIdentityDTO> {
        self.auth_service.create_identity(input).await
    }

    /// The configured auth providers.
    ///
    /// `/auth/{actor_type}/providers` answered with a hardcoded `emailpass` while
    /// `auth.auth_provider` was a real table nobody read, so a merchant who enabled Google
    /// or GitHub saw a login screen that never offered it. The service already falls back
    /// to emailpass when nothing is configured, which is the honest default.
    pub async fn list_providers(&self) -> Result<Vec<AuthProviderDTO>> {
        self.auth_service.list_providers().await
    }

    // 3. get_identity_workflow
    pub async fn get_identity(&self, id: &str) -> Result<AuthIdentityDTO> {
        self.auth_service.get_identity(id).await
    }

    // 4. generate_reset_token_workflow
    pub async fn request_password_reset(
        &self,
        input: PasswordResetRequestInput,
    ) -> Result<Option<String>> {
        self.auth_service.request_password_reset(input).await
    }

    // 5. reset_password_workflow
    pub async fn confirm_password_reset(
        &self,
        input: PasswordResetConfirmInput,
    ) -> Result<AuthIdentityDTO> {
        self.auth_service.confirm_password_reset(input).await
    }

    // 6. update_password_workflow
    pub async fn update_password(&self, input: UpdatePasswordInput) -> Result<AuthIdentityDTO> {
        self.auth_service.update_password(input).await
    }

    // 7. enroll_mfa_workflow
    pub async fn enroll_mfa(
        &self,
        identity_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaEnrollmentDTO> {
        self.auth_service
            .enroll_totp(identity_id, issuer, account_name)
            .await
    }

    // 8. verify_mfa_workflow
    pub async fn verify_mfa(&self, factor_id: &str, code: &str) -> Result<bool> {
        self.auth_service.verify_mfa(factor_id, code).await
    }

    // 9. refresh_token_workflow
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<AuthTokensDTO> {
        self.auth_service.refresh_tokens(refresh_token).await
    }
}
