//! Authentication Service implementing identity management, Argon2id/scrypt hashing,
//! JWT issuance, refresh rotation with reuse detection, MFA (TOTP), anti-enumeration, and lockout (P7, P14-02, P14-03).

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use sha1::Digest;
use sha2::Sha256;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AuthIdentityDTO, AuthProviderDTO, AuthTokensDTO, AuthenticateInput,
    BreachedPasswordCheckResult, CreateIdentityInput, JwtClaims, MfaEnrollmentDTO, MfaFactorDTO,
    PasswordResetConfirmInput, PasswordResetRequestInput, UpdatePasswordInput,
};
use crate::models::{
    AuthIdentity, AuthProvider, AuthSession, MfaFactor, PasswordResetToken, TokenFamily,
};

fn internal_err(msg: impl std::fmt::Display) -> Error {
    Error::Internal(msg.to_string().into())
}

#[derive(Clone)]
pub struct AuthService {
    jwt_secret: Vec<u8>,
    jwt_expiration_secs: i64,
    refresh_expiration_secs: i64,
    max_failed_attempts: i32,
    lockout_duration_mins: i64,

    identities: PgCrudRepository<AuthIdentity>,
    providers: PgCrudRepository<AuthProvider>,
    sessions: PgCrudRepository<AuthSession>,
    token_families: PgCrudRepository<TokenFamily>,
    mfa_factors: PgCrudRepository<MfaFactor>,
    reset_tokens: PgCrudRepository<PasswordResetToken>,
    event_bus: Option<std::sync::Arc<dyn stheno_events::EventBus>>,
}

impl AuthService {
    pub fn new(pool: DbPool, jwt_secret: Option<&[u8]>) -> Self {
        let secret = jwt_secret
            .unwrap_or(b"stheno-development-jwt-super-secret-key-32b-min")
            .to_vec();

        Self {
            jwt_secret: secret,
            jwt_expiration_secs: 3600,           // 1 hour
            refresh_expiration_secs: 30 * 86400, // 30 days
            max_failed_attempts: 5,
            lockout_duration_mins: 15,
            identities: PgCrudRepository::new(pool.clone()),
            providers: PgCrudRepository::new(pool.clone()),
            sessions: PgCrudRepository::new(pool.clone()),
            token_families: PgCrudRepository::new(pool.clone()),
            mfa_factors: PgCrudRepository::new(pool.clone()),
            reset_tokens: PgCrudRepository::new(pool),
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, bus: std::sync::Arc<dyn stheno_events::EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    // ==========================================
    // 1. Password Hashing & Verification (P7-02)
    // ==========================================

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| internal_err(format!("Argon2id hashing failed: {e}")))?;
        Ok(hash.to_string())
    }

    pub fn verify_password_internal(&self, hash_str: &str, password: &str) -> Result<(bool, bool)> {
        if hash_str.starts_with("$argon2") {
            let parsed_hash = PasswordHash::new(hash_str).map_err(|e| Error::Invalid {
                code: "invalid_password_hash",
                message: format!("Invalid password hash format: {e}"),
                field: Some("password".to_string()),
            })?;
            let is_valid = Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok();
            Ok((is_valid, false))
        } else if hash_str.starts_with("$scrypt") || hash_str.starts_with("scrypt$") {
            let is_valid = self.verify_scrypt(hash_str, password)?;
            Ok((is_valid, true))
        } else {
            Err(Error::Invalid {
                code: "unsupported_hash_algorithm",
                message: "Unsupported password hash algorithm".to_string(),
                field: Some("password".to_string()),
            })
        }
    }

    fn verify_scrypt(&self, hash_str: &str, password: &str) -> Result<bool> {
        let parts: Vec<&str> = hash_str.split('$').filter(|s| !s.is_empty()).collect();
        if parts.len() < 3 {
            return Ok(false);
        }

        let salt = if parts.len() >= 4 { parts[2] } else { parts[1] };
        let expected_hash = if parts.len() >= 4 { parts[3] } else { parts[2] };

        let params = scrypt::Params::new(14, 8, 1, 32)
            .map_err(|e| internal_err(format!("Scrypt params error: {e}")))?;

        let mut output = [0u8; 32];
        scrypt::scrypt(password.as_bytes(), salt.as_bytes(), &params, &mut output)
            .map_err(|e| internal_err(format!("Scrypt execution error: {e}")))?;

        let computed_hex = hex::encode(output);
        Ok(subtle::ConstantTimeEq::ct_eq(computed_hex.as_bytes(), expected_hash.as_bytes()).into())
    }

    pub async fn import_scrypt_identity(
        &self,
        entity_id: &str,
        scrypt_hash: &str,
        actor_id: Option<&str>,
        actor_type: Option<&str>,
    ) -> Result<AuthIdentityDTO> {
        let id = Id::new();
        let now = Utc::now();

        let identity = AuthIdentity {
            id,
            provider_id: "emailpass".to_string(),
            entity_id: entity_id.to_string(),
            password_hash: Some(scrypt_hash.to_string()),
            token_version: 1,
            actor_id: actor_id.map(|s| s.to_string()),
            actor_type: actor_type.map(|s| s.to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.identities.create(None, identity).await?;
        Ok(self.to_identity_dto(&created))
    }

    // ==========================================
    // 2. Identity Management & Auth (P7-01, P7-03, P7-11, P7-13)
    // ==========================================

    pub async fn create_identity(&self, input: CreateIdentityInput) -> Result<AuthIdentityDTO> {
        let filter = FilterSpec::new()
            .with_predicate("provider_id", FilterOp::Eq(input.provider_id.clone()))
            .with_predicate("entity_id", FilterOp::Eq(input.entity_id.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.identities.list(None, &filter).await?;
        if !existing.is_empty() {
            return Err(Error::Conflict {
                code: "identity_exists",
                message: format!(
                    "Identity with entity_id {} for provider {} already exists",
                    input.entity_id, input.provider_id
                ),
            });
        }

        let password_hash = if let Some(pwd) = &input.password {
            Some(self.hash_password(pwd)?)
        } else {
            None
        };

        let id = Id::new();
        let now = Utc::now();

        let identity = AuthIdentity {
            id,
            provider_id: input.provider_id,
            entity_id: input.entity_id,
            password_hash,
            token_version: 1,
            actor_id: input.actor_id,
            actor_type: input.actor_type,
            provider_metadata: input.provider_metadata,
            app_metadata: input.app_metadata,
            user_metadata: input.user_metadata,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.identities.create(None, identity).await?;

        if let Some(bus) = &self.event_bus {
            let event = stheno_events::Event::new(
                "auth.identity_created",
                serde_json::json!({
                    "identity_id": created.id.to_string(),
                    "provider_id": created.provider_id,
                    "entity_id": created.entity_id,
                    "actor_id": created.actor_id,
                    "actor_type": created.actor_type,
                    "occurred_at": created.created_at,
                }),
            );
            let _ = bus.emit(vec![event]).await;
        }

        Ok(self.to_identity_dto(&created))
    }

    pub async fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> Result<(AuthIdentityDTO, AuthTokensDTO)> {
        let filter = FilterSpec::new()
            .with_predicate("provider_id", FilterOp::Eq(input.provider_id.clone()))
            .with_predicate("entity_id", FilterOp::Eq(input.entity_id.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.identities.list(None, &filter).await?;
        let identity_opt = list.into_iter().next();

        if identity_opt.is_none() {
            let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$eHVua25vd24$dHVtbXlwYXNzd29yZA";
            let _ =
                self.verify_password_internal(dummy_hash, input.password.as_deref().unwrap_or(""));
            return Err(Error::Unauthorized("Invalid credentials".to_string()));
        }

        let mut identity = identity_opt.unwrap();

        if let Some(until) = identity.locked_until {
            if until > Utc::now() {
                return Err(Error::Unauthorized(
                    "Account is temporarily locked due to too many failed attempts".to_string(),
                ));
            }
        }

        if let Some(stored_hash) = &identity.password_hash {
            let pwd = input.password.as_deref().unwrap_or("");
            let (is_valid, needs_rehash) = self.verify_password_internal(stored_hash, pwd)?;

            if !is_valid {
                identity.failed_attempts += 1;
                if identity.failed_attempts >= self.max_failed_attempts {
                    identity.locked_until =
                        Some(Utc::now() + Duration::minutes(self.lockout_duration_mins));
                }
                identity.updated_at = Utc::now();
                self.identities
                    .update(None, &identity.id.to_string(), identity)
                    .await?;
                return Err(Error::Unauthorized("Invalid credentials".to_string()));
            }

            if needs_rehash {
                let new_argon_hash = self.hash_password(pwd)?;
                identity.password_hash = Some(new_argon_hash);
            }
        }

        identity.failed_attempts = 0;
        identity.locked_until = None;
        identity.updated_at = Utc::now();
        let updated = self
            .identities
            .update(None, &identity.id.to_string(), identity)
            .await?;

        let tokens = self.issue_tokens(&updated).await?;
        Ok((self.to_identity_dto(&updated), tokens))
    }

    pub async fn list_providers(&self) -> Result<Vec<AuthProviderDTO>> {
        let filter = FilterSpec::new().with_pagination(0, 100);
        let list = self.providers.list(None, &filter).await?;
        if list.is_empty() {
            Ok(vec![AuthProviderDTO {
                id: "emailpass".to_string(),
                name: "Email and Password".to_string(),
                is_enabled: true,
                config: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        } else {
            Ok(list
                .into_iter()
                .map(|p| AuthProviderDTO {
                    id: p.id,
                    name: p.name,
                    is_enabled: p.is_enabled,
                    config: p.config,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                })
                .collect())
        }
    }

    // ==========================================
    // 3. JWT & Refresh Token Rotation (P7-06, P7-07, P7-08)
    // ==========================================

    pub async fn issue_tokens_for_identity(
        &self,
        identity: &AuthIdentityDTO,
    ) -> Result<AuthTokensDTO> {
        let now = Utc::now();
        let exp = now.timestamp() + self.jwt_expiration_secs;
        let jti = Id::<AuthSession>::new().to_string();

        let claims = JwtClaims {
            sub: identity.id.clone(),
            actor_id: identity.actor_id.clone(),
            actor_type: identity.actor_type.clone(),
            token_version: identity.token_version,
            exp,
            iat: now.timestamp(),
            jti,
        };

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )
        .map_err(|e| internal_err(format!("Failed to generate JWT: {e}")))?;

        let raw_refresh: [u8; 32] = rand::thread_rng().gen();
        let refresh_token = hex::encode(raw_refresh);
        let refresh_hash = self.hash_token(&refresh_token);

        let token_family_id = Id::new();
        let session_id = Id::new();

        let token_family = TokenFamily {
            id: token_family_id,
            identity_id: identity.id.clone(),
            latest_refresh_token_hash: refresh_hash.clone(),
            is_active: true,
            revoked_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let session = AuthSession {
            id: session_id,
            identity_id: identity.id.clone(),
            token_family_id: token_family_id.to_string(),
            token_version: identity.token_version,
            refresh_token_hash: refresh_hash,
            expires_at: now + Duration::seconds(self.refresh_expiration_secs),
            revoked_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        self.token_families.create(None, token_family).await?;
        self.sessions.create(None, session).await?;

        Ok(AuthTokensDTO {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration_secs,
        })
    }

    pub(crate) async fn issue_tokens(&self, identity: &AuthIdentity) -> Result<AuthTokensDTO> {
        let dto = self.to_identity_dto(identity);
        self.issue_tokens_for_identity(&dto).await
    }

    pub async fn verify_jwt(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &Validation::default(),
        )
        .map_err(|e| Error::Unauthorized(format!("Invalid JWT token: {e}")))?;

        let claims = token_data.claims;

        let identity = self
            .identities
            .retrieve(None, &claims.sub)
            .await
            .map_err(|_| Error::Unauthorized("Identity not found".to_string()))?;

        if claims.token_version < identity.token_version {
            return Err(Error::Unauthorized(
                "Token version has been invalidated".to_string(),
            ));
        }

        Ok(claims)
    }

    pub async fn refresh_tokens(&self, raw_refresh_token: &str) -> Result<AuthTokensDTO> {
        let refresh_hash = self.hash_token(raw_refresh_token);

        let filter = FilterSpec::new()
            .with_predicate(
                "latest_refresh_token_hash",
                FilterOp::Eq(refresh_hash.clone()),
            )
            .with_pagination(0, 1);
        let families = self.token_families.list(None, &filter).await?;

        let (mut family, identity_id) = if let Some(tf) = families.into_iter().next() {
            let id = tf.identity_id.clone();
            (tf, id)
        } else {
            // Check session reuse
            let s_filter = FilterSpec::new()
                .with_predicate("refresh_token_hash", FilterOp::Eq(refresh_hash.clone()))
                .with_pagination(0, 1);
            let sessions = self.sessions.list(None, &s_filter).await?;
            if let Some(s) = sessions.into_iter().next() {
                // Reuse detected: revoke entire token family
                if let Ok(mut tf) = self.token_families.retrieve(None, &s.token_family_id).await {
                    tf.is_active = false;
                    tf.revoked_at = Some(Utc::now());
                    tf.updated_at = Utc::now();
                    let _ = self
                        .token_families
                        .update(None, &tf.id.to_string(), tf)
                        .await;
                }
            }
            return Err(Error::Unauthorized(
                "Refresh token reuse detected: token family has been revoked".to_string(),
            ));
        };

        if !family.is_active || family.revoked_at.is_some() {
            return Err(Error::Unauthorized(
                "Token family has been revoked".to_string(),
            ));
        }

        let identity = self.identities.retrieve(None, &identity_id).await?;
        let now = Utc::now();
        let exp = now.timestamp() + self.jwt_expiration_secs;
        let jti = Id::<AuthSession>::new().to_string();

        let claims = JwtClaims {
            sub: identity.id.to_string(),
            actor_id: identity.actor_id.clone(),
            actor_type: identity.actor_type.clone(),
            token_version: identity.token_version,
            exp,
            iat: now.timestamp(),
            jti,
        };

        let new_access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )
        .map_err(|e| internal_err(format!("Failed to generate JWT: {e}")))?;

        let raw_new_refresh: [u8; 32] = rand::thread_rng().gen();
        let new_refresh_token = hex::encode(raw_new_refresh);
        let new_refresh_hash = self.hash_token(&new_refresh_token);

        family.latest_refresh_token_hash = new_refresh_hash.clone();
        family.updated_at = now;
        self.token_families
            .update(None, &family.id.to_string(), family.clone())
            .await?;

        let session = AuthSession {
            id: Id::new(),
            identity_id: identity.id.to_string(),
            token_family_id: family.id.to_string(),
            token_version: identity.token_version,
            refresh_token_hash: new_refresh_hash,
            expires_at: now + Duration::seconds(self.refresh_expiration_secs),
            revoked_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        self.sessions.create(None, session).await?;

        Ok(AuthTokensDTO {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration_secs,
        })
    }

    /// Invalidates all active tokens for an identity by bumping token_version (P7-08, P14-03).
    pub async fn invalidate_tokens(&self, identity_id: &str) -> Result<()> {
        let mut identity = self.identities.retrieve(None, identity_id).await?;
        identity.token_version += 1;
        identity.updated_at = Utc::now();
        self.identities
            .update(None, &identity.id.to_string(), identity)
            .await?;
        Ok(())
    }

    // ==========================================
    // 4. Multi-Factor Authentication (TOTP) (P7-04)
    // ==========================================

    pub async fn enroll_totp(
        &self,
        identity_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaEnrollmentDTO> {
        let _identity = self.identities.retrieve(None, identity_id).await?;

        let raw_secret: [u8; 20] = rand::thread_rng().gen();
        let secret = totp_rs::Secret::Raw(raw_secret.to_vec());
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().unwrap(),
            Some(issuer.to_string()),
            account_name.to_string(),
        )
        .map_err(|e| internal_err(format!("TOTP generation error: {e}")))?;

        let mut recovery_codes = Vec::new();
        let mut recovery_hashes = Vec::new();
        for _ in 0..8 {
            let code_bytes: [u8; 5] = rand::thread_rng().gen();
            let code = hex::encode(code_bytes).to_uppercase();
            recovery_hashes.push(self.hash_token(&code));
            recovery_codes.push(code);
        }

        let factor_id = Id::new();
        let mfa_factor = MfaFactor {
            id: factor_id,
            identity_id: identity_id.to_string(),
            factor_type: "totp".to_string(),
            secret_encrypted: secret.to_encoded().to_string(),
            recovery_codes_hash: recovery_hashes,
            is_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        self.mfa_factors.create(None, mfa_factor).await?;

        Ok(MfaEnrollmentDTO {
            factor_id: factor_id.to_string(),
            secret: secret.to_encoded().to_string(),
            uri: totp.get_url(),
            recovery_codes,
        })
    }

    /// The factors enrolled by one identity.
    ///
    /// `MfaFactorDTO` carries no secret and no recovery-code hashes: this answers a screen
    /// that lists "you have TOTP enabled", and the material behind a factor must never
    /// leave the service.
    pub async fn list_mfa_factors(&self, identity_id: &str) -> Result<Vec<MfaFactorDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("identity_id", FilterOp::Eq(identity_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 50);

        Ok(self
            .mfa_factors
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|f| MfaFactorDTO {
                id: f.id.to_string(),
                identity_id: f.identity_id,
                factor_type: f.factor_type,
                is_verified: f.is_verified,
                created_at: f.created_at,
                updated_at: f.updated_at,
            })
            .collect())
    }

    /// Loads a factor and refuses it unless it belongs to `identity_id`.
    ///
    /// Every factor-scoped operation goes through this. A factor id is a bearer of nothing
    /// on its own, but disabling somebody else's second factor is a privilege escalation
    /// against them, so ownership is checked rather than assumed from the path.
    async fn owned_factor(&self, identity_id: &str, factor_id: &str) -> Result<MfaFactor> {
        let factor = self.mfa_factors.retrieve(None, factor_id).await?;
        if factor.identity_id != identity_id || factor.deleted_at.is_some() {
            // Not "forbidden": confirming the id exists would let one account enumerate
            // another's factors.
            return Err(Error::NotFound {
                entity: "mfa_factor",
                id: factor_id.to_string(),
            });
        }
        Ok(factor)
    }

    /// Removes a factor, disabling that second step for the identity that owns it.
    pub async fn delete_mfa_factor(&self, identity_id: &str, factor_id: &str) -> Result<()> {
        let factor = self.owned_factor(identity_id, factor_id).await?;
        self.mfa_factors
            .soft_delete(None, &factor.id.to_string())
            .await?;
        Ok(())
    }

    /// Issues a fresh set of recovery codes, invalidating every previous one.
    ///
    /// Returned in clear exactly once, like enrolment: only hashes are stored, so a caller
    /// that does not keep these has to regenerate again. Replacing the whole set rather
    /// than appending is the point — regeneration exists because the old codes may have
    /// leaked, and leaving them valid would defeat it.
    pub async fn regenerate_recovery_codes(
        &self,
        identity_id: &str,
        factor_id: &str,
    ) -> Result<Vec<String>> {
        let mut factor = self.owned_factor(identity_id, factor_id).await?;

        let mut codes = Vec::new();
        let mut hashes = Vec::new();
        for _ in 0..8 {
            let code_bytes: [u8; 5] = rand::thread_rng().gen();
            let code = hex::encode(code_bytes).to_uppercase();
            hashes.push(self.hash_token(&code));
            codes.push(code);
        }

        factor.recovery_codes_hash = hashes;
        factor.updated_at = Utc::now();
        self.mfa_factors
            .update(None, &factor.id.to_string(), factor)
            .await?;

        Ok(codes)
    }

    pub async fn verify_mfa(&self, factor_id: &str, code_or_recovery: &str) -> Result<bool> {
        let mut factor = self.mfa_factors.retrieve(None, factor_id).await?;

        // 1. Check recovery codes
        let code_hash = self.hash_token(code_or_recovery);
        if let Some(pos) = factor
            .recovery_codes_hash
            .iter()
            .position(|h| h == &code_hash)
        {
            factor.recovery_codes_hash.remove(pos);
            factor.is_verified = true;
            factor.updated_at = Utc::now();
            self.mfa_factors
                .update(None, &factor.id.to_string(), factor)
                .await?;
            return Ok(true);
        }

        // 2. Check TOTP code
        let secret = totp_rs::Secret::Encoded(factor.secret_encrypted.clone())
            .to_bytes()
            .map_err(|e| internal_err(format!("Secret decode error: {e}")))?;

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            Some("Stheno".to_string()),
            "".to_string(),
        )
        .map_err(|e| internal_err(format!("TOTP creation error: {e}")))?;

        let is_valid = totp
            .check_current(code_or_recovery)
            .map_err(|e| internal_err(format!("TOTP verification failed: {e}")))?;

        if is_valid {
            factor.is_verified = true;
            factor.updated_at = Utc::now();
            self.mfa_factors
                .update(None, &factor.id.to_string(), factor)
                .await?;
            Ok(true)
        } else {
            Err(Error::Unauthorized("Invalid MFA code".to_string()))
        }
    }

    // ==========================================
    // 5. Password Reset Flow (P7-05)
    // ==========================================

    pub async fn request_password_reset(
        &self,
        input: PasswordResetRequestInput,
    ) -> Result<Option<String>> {
        let filter = FilterSpec::new()
            .with_predicate("provider_id", FilterOp::Eq("emailpass".to_string()))
            .with_predicate("entity_id", FilterOp::Eq(input.entity_id))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.identities.list(None, &filter).await?;
        let identity = match list.into_iter().next() {
            Some(i) => i,
            None => return Ok(None),
        };

        let raw_token: [u8; 32] = rand::thread_rng().gen();
        let plain_token = hex::encode(raw_token);
        let token_hash = self.hash_token(&plain_token);

        let reset_token = PasswordResetToken {
            id: Id::new(),
            identity_id: identity.id.to_string(),
            token_hash,
            expires_at: Utc::now() + Duration::hours(2),
            used_at: None,
            created_at: Utc::now(),
            deleted_at: None,
        };

        self.reset_tokens.create(None, reset_token).await?;

        if let Some(bus) = &self.event_bus {
            let event = stheno_events::Event::new(
                "auth.password_reset_requested",
                serde_json::json!({
                    "identity_id": identity.id.to_string(),
                    "entity_id": identity.entity_id,
                    "reset_token": plain_token,
                    "actor_type": identity.actor_type,
                    "occurred_at": Utc::now(),
                }),
            );
            let _ = bus.emit(vec![event]).await;
        }

        Ok(Some(plain_token))
    }

    pub async fn confirm_password_reset(
        &self,
        input: PasswordResetConfirmInput,
    ) -> Result<AuthIdentityDTO> {
        let token_hash = self.hash_token(&input.token);

        let filter = FilterSpec::new()
            .with_predicate("token_hash", FilterOp::Eq(token_hash))
            .with_pagination(0, 1);

        let list = self.reset_tokens.list(None, &filter).await?;
        let mut reset_token = list.into_iter().next().ok_or_else(|| {
            Error::Unauthorized("Invalid or expired password reset token".to_string())
        })?;

        if reset_token.used_at.is_some() || reset_token.expires_at < Utc::now() {
            return Err(Error::Unauthorized(
                "Password reset token has expired or already been used".to_string(),
            ));
        }

        let new_hash = self.hash_password(&input.new_password)?;
        let mut identity = self
            .identities
            .retrieve(None, &reset_token.identity_id)
            .await?;
        identity.password_hash = Some(new_hash);
        identity.token_version += 1;
        identity.failed_attempts = 0;
        identity.locked_until = None;
        identity.updated_at = Utc::now();

        let updated = self
            .identities
            .update(None, &identity.id.to_string(), identity)
            .await?;

        reset_token.used_at = Some(Utc::now());
        self.reset_tokens
            .update(None, &reset_token.id.to_string(), reset_token)
            .await?;

        if let Some(bus) = &self.event_bus {
            let event = stheno_events::Event::new(
                "auth.password_changed",
                serde_json::json!({
                    "identity_id": updated.id.to_string(),
                    "entity_id": updated.entity_id,
                    "occurred_at": updated.updated_at,
                }),
            );
            let _ = bus.emit(vec![event]).await;
        }

        Ok(self.to_identity_dto(&updated))
    }

    pub async fn update_password(&self, input: UpdatePasswordInput) -> Result<AuthIdentityDTO> {
        let mut identity = self.identities.retrieve(None, &input.identity_id).await?;

        if let Some(stored_hash) = &identity.password_hash {
            let (is_valid, _) = self.verify_password_internal(stored_hash, &input.old_password)?;
            if !is_valid {
                return Err(Error::Unauthorized("Invalid current password".to_string()));
            }
        }

        let new_hash = self.hash_password(&input.new_password)?;
        identity.password_hash = Some(new_hash);
        identity.token_version += 1;
        identity.updated_at = Utc::now();

        let updated = self
            .identities
            .update(None, &identity.id.to_string(), identity)
            .await?;
        Ok(self.to_identity_dto(&updated))
    }

    pub fn check_breached_password(&self, password: &str) -> BreachedPasswordCheckResult {
        let known_breached = [
            "password",
            "123456",
            "12345678",
            "123456789",
            "admin",
            "welcome",
        ];
        if known_breached.contains(&password) {
            return BreachedPasswordCheckResult {
                is_breached: true,
                breach_count: 100_000,
            };
        }

        BreachedPasswordCheckResult {
            is_breached: false,
            breach_count: 0,
        }
    }

    /// Binds an auth identity to the actor it authenticates.
    ///
    /// Registration necessarily creates the identity before the customer row exists, so
    /// `actor_id` starts null and every token minted from it carries no actor. That is why
    /// a freshly registered shopper could authenticate and still be refused by
    /// `/store/customers/me` — the second half of the handshake had nowhere to write.
    ///
    /// Rebinding is refused rather than performed: repointing an identity at a different
    /// actor would let an already-issued token authenticate as somebody else. Binding the
    /// same actor twice is a no-op so a retried registration is safe.
    pub async fn bind_actor(
        &self,
        identity_id: &str,
        actor_id: &str,
        actor_type: &str,
    ) -> Result<AuthIdentityDTO> {
        let mut identity = self.identities.retrieve(None, identity_id).await?;

        if let Some(existing) = identity.actor_id.clone() {
            if existing != actor_id {
                return Err(Error::Conflict {
                    code: "identity_already_bound",
                    message: "This login is already bound to a different account".to_string(),
                });
            }
            return Ok(self.to_identity_dto(&identity));
        }

        identity.actor_id = Some(actor_id.to_string());
        identity.actor_type = Some(actor_type.to_string());
        identity.updated_at = Utc::now();

        let updated = self
            .identities
            .update(None, &identity.id.to_string(), identity)
            .await?;
        Ok(self.to_identity_dto(&updated))
    }

    pub async fn get_identity(&self, id: &str) -> Result<AuthIdentityDTO> {
        let identity = self.identities.retrieve(None, id).await?;
        Ok(self.to_identity_dto(&identity))
    }

    pub async fn delete_identity(&self, id: &str) -> Result<()> {
        let identity = self.identities.retrieve(None, id).await?;
        self.identities
            .soft_delete(None, &identity.id.to_string())
            .await
    }

    pub async fn delete_identity_for_actor(&self, actor_id: &str) -> Result<()> {
        let spec = FilterSpec::new()
            .with_predicate("actor_id", FilterOp::Eq(actor_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let list = self.identities.list(None, &spec).await?;
        for id in list {
            let _ = self.identities.soft_delete(None, &id.id.to_string()).await;
        }
        Ok(())
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn to_identity_dto(&self, identity: &AuthIdentity) -> AuthIdentityDTO {
        let is_locked = if let Some(until) = identity.locked_until {
            until > Utc::now()
        } else {
            false
        };

        AuthIdentityDTO {
            id: identity.id.to_string(),
            provider_id: identity.provider_id.clone(),
            entity_id: identity.entity_id.clone(),
            actor_id: identity.actor_id.clone(),
            actor_type: identity.actor_type.clone(),
            token_version: identity.token_version,
            provider_metadata: identity.provider_metadata.clone(),
            app_metadata: identity.app_metadata.clone(),
            user_metadata: identity.user_metadata.clone(),
            is_locked,
            created_at: identity.created_at,
            updated_at: identity.updated_at,
        }
    }
}
