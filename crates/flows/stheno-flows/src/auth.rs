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

    // Multi-factor authentication.
    //
    // Every route behind these used to answer from a literal: enrolment handed out the
    // RFC test-vector secret `JBSWY3DPEHPK3PXP` to every caller, recovery codes were three
    // hardcoded strings, and all three verify endpoints returned `verified: true` while
    // discarding the submitted code. A user who turned MFA on had no second factor and
    // was told they did.
    pub async fn enroll_totp(
        &self,
        identity_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaEnrollmentDTO> {
        self.auth_service
            .enroll_totp(identity_id, issuer, account_name)
            .await
    }

    pub async fn list_mfa_factors(&self, identity_id: &str) -> Result<Vec<MfaFactorDTO>> {
        self.auth_service.list_mfa_factors(identity_id).await
    }

    pub async fn delete_mfa_factor(&self, identity_id: &str, factor_id: &str) -> Result<()> {
        self.auth_service
            .delete_mfa_factor(identity_id, factor_id)
            .await
    }

    pub async fn regenerate_recovery_codes(
        &self,
        identity_id: &str,
        factor_id: &str,
    ) -> Result<Vec<String>> {
        self.auth_service
            .regenerate_recovery_codes(identity_id, factor_id)
            .await
    }

    /// Signs every outstanding token for an identity out.
    ///
    /// Bumps `token_version`, which `verify_jwt` compares against — so this is a real
    /// logout across every device, not a client-side token drop.
    pub async fn invalidate_tokens(&self, identity_id: &str) -> Result<()> {
        self.auth_service.invalidate_tokens(identity_id).await
    }

    pub async fn verify_jwt(&self, token: &str) -> Result<JwtClaims> {
        self.auth_service.verify_jwt(token).await
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
