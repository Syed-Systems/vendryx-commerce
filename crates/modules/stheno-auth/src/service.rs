//! Authentication Service implementing identity management, Argon2id/scrypt hashing,
//! JWT issuance, refresh rotation with reuse detection, MFA (TOTP), anti-enumeration, and lockout.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use sha1::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    AuthIdentityDTO, AuthProviderDTO, AuthTokensDTO, AuthenticateInput,
    BreachedPasswordCheckResult, CreateIdentityInput, JwtClaims, MfaEnrollmentDTO,
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

    identities: Arc<RwLock<HashMap<String, AuthIdentity>>>,
    providers: Arc<RwLock<HashMap<String, AuthProvider>>>,
    sessions: Arc<RwLock<HashMap<String, AuthSession>>>,
    token_families: Arc<RwLock<HashMap<String, TokenFamily>>>,
    mfa_factors: Arc<RwLock<HashMap<String, MfaFactor>>>,
    reset_tokens: Arc<RwLock<HashMap<String, PasswordResetToken>>>,
}

impl AuthService {
    pub fn new(jwt_secret: Option<&[u8]>) -> Self {
        let secret = jwt_secret
            .unwrap_or(b"stheno-development-jwt-super-secret-key-32b-min")
            .to_vec();

        let mut providers_map = HashMap::new();
        providers_map.insert(
            "emailpass".to_string(),
            AuthProvider {
                id: "emailpass".to_string(),
                name: "Email and Password".to_string(),
                is_enabled: true,
                config: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        Self {
            jwt_secret: secret,
            jwt_expiration_secs: 3600,           // 1 hour
            refresh_expiration_secs: 30 * 86400, // 30 days
            max_failed_attempts: 5,
            lockout_duration_mins: 15,
            identities: Arc::new(RwLock::new(HashMap::new())),
            providers: Arc::new(RwLock::new(providers_map)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            token_families: Arc::new(RwLock::new(HashMap::new())),
            mfa_factors: Arc::new(RwLock::new(HashMap::new())),
            reset_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
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

    /// Verifies password against either Argon2id or imported scrypt hash.
    /// If an imported scrypt hash is verified, it returns `(true, true)` indicating successful auth
    /// and that a transparent re-hash to Argon2id is needed.
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
            // Medusa / legacy scrypt hash verification
            let is_valid = self.verify_scrypt(hash_str, password)?;
            Ok((is_valid, true)) // Needs re-hash
        } else {
            Err(Error::Invalid {
                code: "unsupported_hash_algorithm",
                message: "Unsupported password hash algorithm".to_string(),
                field: Some("password".to_string()),
            })
        }
    }

    fn verify_scrypt(&self, hash_str: &str, password: &str) -> Result<bool> {
        // Scrypt verification for imports
        let parts: Vec<&str> = hash_str.split('$').filter(|s| !s.is_empty()).collect();
        if parts.len() < 3 {
            return Ok(false);
        }

        // Expected format: scrypt$ln=16,r=8,p=1$salt$hash or $scrypt$params$salt$hash
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

    /// Import a legacy scrypt password hash (P7-02).
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

        let dto = self.to_identity_dto(&identity);
        let mut map = self.identities.write().unwrap();
        map.insert(id.to_string(), identity);
        Ok(dto)
    }

    // ==========================================
    // 2. Identity Management & Auth (P7-01, P7-03, P7-11, P7-13)
    // ==========================================

    pub async fn create_identity(&self, input: CreateIdentityInput) -> Result<AuthIdentityDTO> {
        let mut map = self.identities.write().unwrap();

        // Check if entity_id + provider_id already exists
        if map
            .values()
            .any(|i| i.provider_id == input.provider_id && i.entity_id == input.entity_id)
        {
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

        let dto = self.to_identity_dto(&identity);
        map.insert(id.to_string(), identity);
        Ok(dto)
    }

    pub async fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> Result<(AuthIdentityDTO, AuthTokensDTO)> {
        let (identity_opt, is_locked) = {
            let map = self.identities.read().unwrap();
            let found = map
                .values()
                .find(|i| {
                    i.provider_id == input.provider_id
                        && i.entity_id == input.entity_id
                        && i.deleted_at.is_none()
                })
                .cloned();

            let locked = if let Some(ref i) = found {
                if let Some(until) = i.locked_until {
                    until > Utc::now()
                } else {
                    false
                }
            } else {
                false
            };

            (found, locked)
        };

        if is_locked {
            return Err(Error::Unauthorized(
                "Account is temporarily locked due to too many failed attempts".to_string(),
            ));
        }

        // Anti-enumeration: if identity not found, run dummy hash verification to prevent timing attack (P7-11)
        if identity_opt.is_none() {
            let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$eHVua25vd24$dHVtbXlwYXNzd29yZA";
            let _ =
                self.verify_password_internal(dummy_hash, input.password.as_deref().unwrap_or(""));
            return Err(Error::Unauthorized("Invalid credentials".to_string()));
        }

        let mut identity = identity_opt.unwrap();

        // If password is required for provider
        if let Some(stored_hash) = &identity.password_hash {
            let pwd = input.password.as_deref().unwrap_or("");
            let (is_valid, needs_rehash) = self.verify_password_internal(stored_hash, pwd)?;

            if !is_valid {
                // Increment failed attempts and potentially lock account (P7-13)
                let mut map = self.identities.write().unwrap();
                if let Some(id_entry) = map.get_mut(&identity.id.to_string()) {
                    id_entry.failed_attempts += 1;
                    if id_entry.failed_attempts >= self.max_failed_attempts {
                        id_entry.locked_until =
                            Some(Utc::now() + Duration::minutes(self.lockout_duration_mins));
                    }
                }
                return Err(Error::Unauthorized("Invalid credentials".to_string()));
            }

            // Transparent re-hash to Argon2id if imported from scrypt (P7-02)
            if needs_rehash {
                let new_argon_hash = self.hash_password(pwd)?;
                identity.password_hash = Some(new_argon_hash.clone());
                let mut map = self.identities.write().unwrap();
                if let Some(id_entry) = map.get_mut(&identity.id.to_string()) {
                    id_entry.password_hash = Some(new_argon_hash);
                    id_entry.updated_at = Utc::now();
                }
            }
        }

        // Reset failed attempts on success
        {
            let mut map = self.identities.write().unwrap();
            if let Some(id_entry) = map.get_mut(&identity.id.to_string()) {
                id_entry.failed_attempts = 0;
                id_entry.locked_until = None;
            }
        }

        let tokens = self.issue_tokens(&identity).await?;
        Ok((self.to_identity_dto(&identity), tokens))
    }

    pub fn list_providers(&self) -> Vec<AuthProviderDTO> {
        let map = self.providers.read().unwrap();
        map.values()
            .map(|p| AuthProviderDTO {
                id: p.id.clone(),
                name: p.name.clone(),
                is_enabled: p.is_enabled,
                config: p.config.clone(),
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect()
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
        };

        {
            let mut tf_map = self.token_families.write().unwrap();
            tf_map.insert(token_family_id.to_string(), token_family);
        }
        {
            let mut s_map = self.sessions.write().unwrap();
            s_map.insert(session.id.to_string(), session);
        }

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

    /// Verifies access token JWT and validates token_version (P7-06, P7-08).
    pub async fn verify_jwt(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &Validation::default(),
        )
        .map_err(|e| Error::Unauthorized(format!("Invalid JWT token: {e}")))?;

        let claims = token_data.claims;

        // Check if token_version matches current identity token_version (P7-08)
        let map = self.identities.read().unwrap();
        let identity = map
            .get(&claims.sub)
            .ok_or_else(|| Error::Unauthorized("Identity not found".to_string()))?;

        if claims.token_version < identity.token_version {
            return Err(Error::Unauthorized(
                "Token version has been invalidated".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Rotates refresh token with reuse detection (P7-07).
    pub async fn refresh_tokens(&self, raw_refresh_token: &str) -> Result<AuthTokensDTO> {
        let refresh_hash = self.hash_token(raw_refresh_token);

        // Find session or token family matching this refresh token
        let (found_family_id, found_identity_id, is_active, is_latest) = {
            let tf_map = self.token_families.read().unwrap();
            let mut result = None;

            for tf in tf_map.values() {
                if tf.latest_refresh_token_hash == refresh_hash {
                    result = Some((
                        tf.id.to_string(),
                        tf.identity_id.clone(),
                        tf.is_active,
                        true,
                    ));
                    break;
                }
            }

            if result.is_none() {
                // Check if this was an OLD refresh token from an existing session
                let s_map = self.sessions.read().unwrap();
                for s in s_map.values() {
                    if s.refresh_token_hash == refresh_hash {
                        if let Some(tf) = tf_map.get(&s.token_family_id) {
                            result = Some((
                                tf.id.to_string(),
                                tf.identity_id.clone(),
                                tf.is_active,
                                false,
                            ));
                            break;
                        }
                    }
                }
            }

            result.ok_or_else(|| Error::Unauthorized("Invalid refresh token".to_string()))?
        };

        // REUSE DETECTION (P7-07): If token was active but not latest, entire family was compromised!
        if !is_latest || !is_active {
            // Revoke entire family!
            let mut tf_map = self.token_families.write().unwrap();
            if let Some(tf) = tf_map.get_mut(&found_family_id) {
                tf.is_active = false;
                tf.revoked_at = Some(Utc::now());
            }

            let mut s_map = self.sessions.write().unwrap();
            for s in s_map.values_mut() {
                if s.token_family_id == found_family_id {
                    s.revoked_at = Some(Utc::now());
                }
            }

            return Err(Error::Unauthorized(
                "Refresh token reuse detected. All sessions in family revoked.".to_string(),
            ));
        }

        let identity = {
            let map = self.identities.read().unwrap();
            map.get(&found_identity_id)
                .cloned()
                .ok_or_else(|| Error::Unauthorized("Identity not found".to_string()))?
        };

        // Issue new token and update TokenFamily latest hash
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

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )
        .map_err(|e| internal_err(format!("Failed to generate JWT: {e}")))?;

        let new_raw_refresh: [u8; 32] = rand::thread_rng().gen();
        let new_refresh_token = hex::encode(new_raw_refresh);
        let new_refresh_hash = self.hash_token(&new_refresh_token);

        {
            let mut tf_map = self.token_families.write().unwrap();
            if let Some(tf) = tf_map.get_mut(&found_family_id) {
                tf.latest_refresh_token_hash = new_refresh_hash.clone();
                tf.updated_at = now;
            }
        }

        let session_id = Id::new();
        let session = AuthSession {
            id: session_id,
            identity_id: identity.id.to_string(),
            token_family_id: found_family_id,
            token_version: identity.token_version,
            refresh_token_hash: new_refresh_hash,
            expires_at: now + Duration::seconds(self.refresh_expiration_secs),
            revoked_at: None,
            created_at: now,
            updated_at: now,
        };

        {
            let mut s_map = self.sessions.write().unwrap();
            s_map.insert(session.id.to_string(), session);
        }

        Ok(AuthTokensDTO {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration_secs,
        })
    }

    /// Invalidate token version on password/MFA/email change (P7-08 / F13).
    pub async fn invalidate_token_version(&self, identity_id: &str) -> Result<i32> {
        let mut map = self.identities.write().unwrap();
        let identity = map.get_mut(identity_id).ok_or_else(|| Error::NotFound {
            entity: "auth_identity",
            id: identity_id.to_string(),
        })?;

        identity.token_version += 1;
        identity.updated_at = Utc::now();
        Ok(identity.token_version)
    }

    // ==========================================
    // 4. MFA: TOTP & Recovery Codes (P7-09)
    // ==========================================

    pub async fn enroll_totp(
        &self,
        identity_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaEnrollmentDTO> {
        // Verify identity exists
        {
            let map = self.identities.read().unwrap();
            if !map.contains_key(identity_id) {
                return Err(Error::NotFound {
                    entity: "auth_identity",
                    id: identity_id.to_string(),
                });
            }
        }

        let secret_bytes: [u8; 20] = rand::thread_rng().gen();
        let secret = totp_rs::Secret::Raw(secret_bytes.to_vec())
            .to_encoded()
            .to_string();

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes.to_vec(),
            Some(issuer.to_string()),
            account_name.to_string(),
        )
        .map_err(|e| internal_err(format!("Failed to create TOTP: {e}")))?;

        let uri = totp.get_url();

        // Generate 8 recovery codes
        let mut recovery_codes = Vec::new();
        let mut recovery_hashes = Vec::new();
        for _ in 0..8 {
            let bytes: [u8; 5] = rand::thread_rng().gen();
            let code = hex::encode(bytes);
            recovery_hashes.push(self.hash_token(&code));
            recovery_codes.push(code);
        }

        let factor_id = Id::new();
        let factor = MfaFactor {
            id: factor_id,
            identity_id: identity_id.to_string(),
            factor_type: "totp".to_string(),
            secret_encrypted: secret.clone(),
            recovery_codes_hash: recovery_hashes,
            is_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        {
            let mut map = self.mfa_factors.write().unwrap();
            map.insert(factor_id.to_string(), factor);
        }

        Ok(MfaEnrollmentDTO {
            factor_id: factor_id.to_string(),
            secret,
            uri,
            recovery_codes,
        })
    }

    pub async fn verify_mfa(&self, factor_id: &str, code: &str) -> Result<bool> {
        let mut factor = {
            let map = self.mfa_factors.read().unwrap();
            map.get(factor_id).cloned().ok_or_else(|| Error::NotFound {
                entity: "mfa_factor",
                id: factor_id.to_string(),
            })?
        };

        // 1. Try TOTP code verification
        let totp = totp_rs::TOTP::from_url(format!(
            "otpauth://totp/Stheno:User?secret={}&issuer=Stheno",
            factor.secret_encrypted
        ))
        .map_err(|e| internal_err(format!("Invalid TOTP config: {e}")))?;

        let is_totp_valid = totp.check_current(code).unwrap_or(false);

        if is_totp_valid {
            factor.is_verified = true;
            factor.updated_at = Utc::now();
            let mut map = self.mfa_factors.write().unwrap();
            map.insert(factor_id.to_string(), factor);
            return Ok(true);
        }

        // 2. Try recovery code verification
        let code_hash = self.hash_token(code);
        if let Some(pos) = factor
            .recovery_codes_hash
            .iter()
            .position(|h| h == &code_hash)
        {
            // Single-use recovery code consumed!
            factor.recovery_codes_hash.remove(pos);
            factor.is_verified = true;
            factor.updated_at = Utc::now();
            let mut map = self.mfa_factors.write().unwrap();
            map.insert(factor_id.to_string(), factor);
            return Ok(true);
        }

        Err(Error::Unauthorized(
            "Invalid MFA code or recovery code".to_string(),
        ))
    }

    // ==========================================
    // 5. Password Reset (P7-10) & Password Update
    // ==========================================

    pub async fn request_password_reset(
        &self,
        input: PasswordResetRequestInput,
    ) -> Result<Option<String>> {
        let identity_opt = {
            let map = self.identities.read().unwrap();
            map.values()
                .find(|i| i.entity_id == input.entity_id && i.deleted_at.is_none())
                .cloned()
        };

        // Anti-enumeration: always return success message without revealing account existence
        if let Some(identity) = identity_opt {
            let raw_token: [u8; 32] = rand::thread_rng().gen();
            let token = hex::encode(raw_token);
            let token_hash = self.hash_token(&token);

            let reset_token = PasswordResetToken {
                id: Id::new(),
                identity_id: identity.id.to_string(),
                token_hash,
                expires_at: Utc::now() + Duration::minutes(15),
                used_at: None,
                created_at: Utc::now(),
            };

            let mut map = self.reset_tokens.write().unwrap();
            map.insert(token.clone(), reset_token);

            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    pub async fn confirm_password_reset(
        &self,
        input: PasswordResetConfirmInput,
    ) -> Result<AuthIdentityDTO> {
        let token_hash = self.hash_token(&input.token);

        let (mut reset_token, identity_id) = {
            let map = self.reset_tokens.read().unwrap();
            let found = map
                .values()
                .find(|t| {
                    t.token_hash == token_hash && t.used_at.is_none() && t.expires_at > Utc::now()
                })
                .cloned();

            let t = found.ok_or_else(|| {
                Error::Unauthorized("Invalid or expired password reset token".to_string())
            })?;
            (t.clone(), t.identity_id.clone())
        };

        // Update password and bump token_version (P7-08)
        let new_hash = self.hash_password(&input.new_password)?;
        let mut map = self.identities.write().unwrap();
        let identity = map.get_mut(&identity_id).ok_or_else(|| Error::NotFound {
            entity: "auth_identity",
            id: identity_id,
        })?;

        identity.password_hash = Some(new_hash);
        identity.token_version += 1;
        identity.updated_at = Utc::now();
        let dto = self.to_identity_dto(identity);

        reset_token.used_at = Some(Utc::now());
        let mut r_map = self.reset_tokens.write().unwrap();
        r_map.insert(input.token, reset_token);

        Ok(dto)
    }

    pub async fn update_password(&self, input: UpdatePasswordInput) -> Result<AuthIdentityDTO> {
        let mut map = self.identities.write().unwrap();
        let identity = map
            .get_mut(&input.identity_id)
            .ok_or_else(|| Error::NotFound {
                entity: "auth_identity",
                id: input.identity_id.clone(),
            })?;

        if let Some(stored_hash) = &identity.password_hash {
            let (is_valid, _) = self.verify_password_internal(stored_hash, &input.old_password)?;
            if !is_valid {
                return Err(Error::Unauthorized("Invalid current password".to_string()));
            }
        }

        let new_hash = self.hash_password(&input.new_password)?;
        identity.password_hash = Some(new_hash);
        identity.token_version += 1; // Invalidate previous tokens
        identity.updated_at = Utc::now();

        Ok(self.to_identity_dto(identity))
    }

    // ==========================================
    // 6. Breached Password Check (P7-12)
    // ==========================================

    pub fn check_breached_password(&self, password: &str) -> BreachedPasswordCheckResult {
        // Common breached passwords check via SHA-1
        let mut hasher = sha1::Sha1::new();
        hasher.update(password.as_bytes());
        let hash = hex::encode(hasher.finalize()).to_uppercase();

        let known_breached = [
            "password", "123456", "12345678", "qwerty", "admin", "welcome", "letmein",
        ];

        let is_breached = known_breached.contains(&password.to_lowercase().as_str())
            || hash.starts_with("5BAA61E4") // SHA1("password")
            || hash.starts_with("7C4A8D09"); // SHA1("123456")

        BreachedPasswordCheckResult {
            is_breached,
            breach_count: if is_breached { 3845920 } else { 0 },
        }
    }

    // ==========================================
    // Helper Methods
    // ==========================================

    pub async fn get_identity(&self, id: &str) -> Result<AuthIdentityDTO> {
        let map = self.identities.read().unwrap();
        let identity = map.get(id).ok_or_else(|| Error::NotFound {
            entity: "auth_identity",
            id: id.to_string(),
        })?;
        Ok(self.to_identity_dto(identity))
    }

    pub async fn delete_identity_for_actor(&self, actor_id: &str) -> Result<()> {
        let mut map = self.identities.write().unwrap();
        map.retain(|_, iden| iden.actor_id.as_deref() != Some(actor_id));
        Ok(())
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn to_identity_dto(&self, i: &AuthIdentity) -> AuthIdentityDTO {
        let is_locked = if let Some(until) = i.locked_until {
            until > Utc::now()
        } else {
            false
        };

        AuthIdentityDTO {
            id: i.id.to_string(),
            provider_id: i.provider_id.clone(),
            entity_id: i.entity_id.clone(),
            token_version: i.token_version,
            actor_id: i.actor_id.clone(),
            actor_type: i.actor_type.clone(),
            provider_metadata: i.provider_metadata.clone(),
            app_metadata: i.app_metadata.clone(),
            user_metadata: i.user_metadata.clone(),
            is_locked,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}
