//! DTOs, State Machines, Money Boundary, and Provider Traits for `stheno-payment` (PAYMENTS §2-§6).

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use stheno_types::{Error, Result};

// ── State Machines (PAYMENTS §3.1, §3.2) ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSessionStatus {
    Pending,
    PendingAuthorization,
    RequiresMore,
    Authorized,
    Captured,
    Canceled,
    Error,
}

impl PaymentSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::PendingAuthorization => "pending_authorization",
            Self::RequiresMore => "requires_more",
            Self::Authorized => "authorized",
            Self::Captured => "captured",
            Self::Canceled => "canceled",
            Self::Error => "error",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "pending_authorization" => Some(Self::PendingAuthorization),
            "requires_more" => Some(Self::RequiresMore),
            "authorized" => Some(Self::Authorized),
            "captured" => Some(Self::Captured),
            "canceled" => Some(Self::Canceled),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Monotonic state transition rule (PAYMENTS §3.1, §7.2).
    /// Prevents backwards or illegal transitions from out-of-order webhooks/responses.
    pub fn can_transition_to(&self, next: PaymentSessionStatus) -> bool {
        if *self == next {
            return true;
        }
        match self {
            Self::Pending => matches!(
                next,
                Self::PendingAuthorization
                    | Self::RequiresMore
                    | Self::Authorized
                    | Self::Canceled
                    | Self::Error
            ),
            Self::PendingAuthorization => matches!(
                next,
                Self::RequiresMore | Self::Authorized | Self::Canceled | Self::Error
            ),
            Self::RequiresMore => {
                matches!(next, Self::Authorized | Self::Canceled | Self::Error)
            }
            Self::Authorized => matches!(next, Self::Captured | Self::Canceled | Self::Error),
            Self::Captured => false, // Captured is terminal (except for refunds on payment aggregate)
            Self::Canceled => false, // Terminal
            Self::Error => false,    // Terminal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookAction {
    Authorized,
    Captured,
    Failed,
    Pending,
    RequiresMore,
    Canceled,
    PendingAuthorization,
    NotSupported,
}

// Provider Credential Schema (ADR-019, P21-C2)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
    Text,
    Secret,
    Pem,
    Url,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CredentialField {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: CredentialKind,
    pub required: bool,
}

pub type CredentialMap = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialHealth {
    Configured,
    Unconfigured,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialFieldReadDTO {
    pub key: String,
    pub label: String,
    pub kind: CredentialKind,
    pub required: bool,
    pub configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentProviderCredentialDTO {
    pub id: String,
    pub provider_id: String,
    pub credential_version: i32,
    pub webhook_slug: String,
    pub health: CredentialHealth,
    pub fields: Vec<CredentialFieldReadDTO>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedCredential {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub trait PaymentProviderFactory: Send + Sync + 'static {
    fn provider_id(&self) -> &str;
    fn credential_schema(&self) -> &'static [CredentialField];
    fn build(&self, creds: &CredentialMap) -> Result<Arc<dyn PaymentProvider>>;
}

pub fn validate_credential_map(schema: &[CredentialField], creds: &CredentialMap) -> Result<()> {
    for key in creds.keys() {
        if !schema.iter().any(|field| field.key == key) {
            return Err(Error::Invalid {
                code: "credential_field_unknown",
                message: format!("Unknown credential field '{key}'"),
                field: Some(key.clone()),
            });
        }
    }

    for field in schema {
        let value = creds.get(field.key);
        if field.required && value_is_absent(value) {
            return Err(Error::Invalid {
                code: "credential_field_required",
                message: format!("Credential field '{}' is required", field.key),
                field: Some(field.key.to_string()),
            });
        }

        let Some(value) = value else {
            continue;
        };
        if value_is_absent(Some(value)) {
            continue;
        }

        match field.kind {
            CredentialKind::Text | CredentialKind::Secret => {
                require_string(field, value, "credential_string_required")?;
            }
            CredentialKind::Pem => {
                let raw = require_string(field, value, "credential_pem_invalid")?;
                if !is_valid_pem(raw) {
                    return Err(Error::Invalid {
                        code: "credential_pem_invalid",
                        message: format!(
                            "Credential field '{}' must be a valid PEM block",
                            field.key
                        ),
                        field: Some(field.key.to_string()),
                    });
                }
            }
            CredentialKind::Url => {
                let raw = require_string(field, value, "credential_url_invalid")?;
                let parsed = url::Url::parse(raw).map_err(|_| Error::Invalid {
                    code: "credential_url_invalid",
                    message: format!("Credential field '{}' must be a valid URL", field.key),
                    field: Some(field.key.to_string()),
                })?;

                if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
                    return Err(Error::Invalid {
                        code: "credential_url_invalid",
                        message: format!("Credential field '{}' must be an HTTP(S) URL", field.key),
                        field: Some(field.key.to_string()),
                    });
                }
            }
            CredentialKind::Bool => {
                if !value.is_boolean() {
                    return Err(Error::Invalid {
                        code: "credential_bool_required",
                        message: format!("Credential field '{}' must be a boolean", field.key),
                        field: Some(field.key.to_string()),
                    });
                }
            }
        }
    }

    Ok(())
}

pub fn encrypt_credential_map(
    tenant_id: &str,
    provider_id: &str,
    creds: &CredentialMap,
) -> Result<EncryptedCredential> {
    let cipher = credential_cipher_from_env()?;
    let plaintext = serde_json::to_vec(creds).map_err(|e| Error::Invalid {
        code: "credential_serialize_failed",
        message: e.to_string(),
        field: None,
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let aad = credential_aad(tenant_id, provider_id);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: &plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| Error::Invalid {
            code: "credential_encrypt_failed",
            message: "Credential encryption failed".to_string(),
            field: None,
        })?;

    Ok(EncryptedCredential {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

pub fn decrypt_credential_map(
    tenant_id: &str,
    provider_id: &str,
    encrypted: &EncryptedCredential,
) -> Result<CredentialMap> {
    if encrypted.nonce.len() != 12 {
        return Err(Error::Invalid {
            code: "credential_nonce_invalid",
            message: "Credential nonce is invalid".to_string(),
            field: None,
        });
    }

    let cipher = credential_cipher_from_env()?;
    let nonce = Nonce::from_slice(&encrypted.nonce);
    let aad = credential_aad(tenant_id, provider_id);
    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &encrypted.ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| Error::Invalid {
            code: "credential_decrypt_failed",
            message: "Credential decryption failed".to_string(),
            field: None,
        })?;

    serde_json::from_slice(&plaintext).map_err(|_| Error::Invalid {
        code: "credential_payload_invalid",
        message: "Credential payload is invalid".to_string(),
        field: None,
    })
}

fn value_is_absent(value: Option<&serde_json::Value>) -> bool {
    match value {
        None | Some(serde_json::Value::Null) => true,
        Some(serde_json::Value::String(raw)) => raw.trim().is_empty(),
        _ => false,
    }
}

fn credential_cipher_from_env() -> Result<Aes256Gcm> {
    let raw = std::env::var("STHENO_CREDENTIAL_KEY").map_err(|_| Error::Invalid {
        code: "credential_key_missing",
        message: "STHENO_CREDENTIAL_KEY is required".to_string(),
        field: None,
    })?;

    let key = if raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        hex::decode(raw).map_err(|_| Error::Invalid {
            code: "credential_key_invalid",
            message: "STHENO_CREDENTIAL_KEY must be 32 bytes".to_string(),
            field: None,
        })?
    } else {
        raw.into_bytes()
    };

    if key.len() != 32 {
        return Err(Error::Invalid {
            code: "credential_key_invalid",
            message: "STHENO_CREDENTIAL_KEY must be 32 bytes".to_string(),
            field: None,
        });
    }

    Aes256Gcm::new_from_slice(&key).map_err(|_| Error::Invalid {
        code: "credential_key_invalid",
        message: "STHENO_CREDENTIAL_KEY must be 32 bytes".to_string(),
        field: None,
    })
}

fn credential_aad(tenant_id: &str, provider_id: &str) -> Vec<u8> {
    let mut aad = Vec::with_capacity(tenant_id.len() + provider_id.len() + 1);
    aad.extend_from_slice(tenant_id.as_bytes());
    aad.push(0);
    aad.extend_from_slice(provider_id.as_bytes());
    aad
}

fn require_string<'a>(
    field: &CredentialField,
    value: &'a serde_json::Value,
    code: &'static str,
) -> Result<&'a str> {
    value.as_str().ok_or_else(|| Error::Invalid {
        code,
        message: format!("Credential field '{}' must be a string", field.key),
        field: Some(field.key.to_string()),
    })
}

fn is_valid_pem(raw: &str) -> bool {
    let lines: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.len() < 3 {
        return false;
    }

    let Some(label) = lines[0]
        .strip_prefix("-----BEGIN ")
        .and_then(|s| s.strip_suffix("-----"))
    else {
        return false;
    };

    if lines[lines.len() - 1] != format!("-----END {label}-----") {
        return false;
    }

    let body: String = lines[1..lines.len() - 1].join("");
    if body.is_empty() || body.len() % 4 == 1 {
        return false;
    }

    let first_padding = body.find('=').unwrap_or(body.len());
    body[..first_padding]
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/')
        && body[first_padding..].bytes().all(|b| b == b'=')
        && body[first_padding..].len() <= 2
}

// ── Money Boundary (PAYMENTS §5) ─────────────────────────────────────────────

pub fn minor_unit_exponent(currency: &str) -> u32 {
    let cur = currency.to_uppercase();
    match cur.as_str() {
        // Zero-decimal currencies
        "BIF" | "CLP" | "DJF" | "GNF" | "JPY" | "KMF" | "KRW" | "MGA" | "PYG" | "RWF" | "UGX"
        | "VND" | "VUV" | "XAF" | "XOF" | "XPF" => 0,
        // Three-decimal currencies
        "BHD" | "IQD" | "JOD" | "KWD" | "OMR" | "TND" => 3,
        // Default
        _ => 2,
    }
}

/// Converts `Decimal` to provider integer minor units.
/// Implements three-decimal upward rounding to nearest 10 for Kuwaiti Dinar / Bahraini Dinar etc. (PAYMENTS §5).
pub fn to_minor_units(amount: Decimal, currency_code: &str) -> Result<i64> {
    let exp = minor_unit_exponent(currency_code);
    let multiplier = Decimal::from(10i64.pow(exp));
    let scaled = (amount * multiplier).round();
    let mut v = scaled.to_i64().ok_or(Error::AmountOverflow)?;

    // Three-decimal trap: Stripe requires the last digit to be 0
    if exp == 3 {
        v = ((v + 9) / 10) * 10;
    }

    Ok(v)
}

/// Converts provider integer minor units back to `Decimal` (PAYMENTS §5).
pub fn from_minor_units(v: i64, currency_code: &str) -> Decimal {
    let exp = minor_unit_exponent(currency_code);
    let divisor = Decimal::from(10i64.pow(exp));
    Decimal::from(v) / divisor
}

// ── Provider Capabilities & Trait Types (PAYMENTS §4) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub partial_capture: bool,
    pub multi_capture: bool,
    pub partial_refund: bool,
    pub cancel_after_authorize: bool,
    pub vault: bool,
    pub async_settlement: bool,
    pub authorization_ttl: Option<Duration>,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            partial_capture: true,
            multi_capture: true,
            partial_refund: true,
            cancel_after_authorize: true,
            vault: false,
            async_settlement: false,
            authorization_ttl: Some(Duration::from_secs(7 * 24 * 3600)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateInput {
    pub amount: Decimal,
    pub currency_code: String,
    pub context: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateOutput {
    pub id: String,
    pub status: PaymentSessionStatus,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInput {
    pub session_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOutput {
    pub status: PaymentSessionStatus,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteInput {
    pub session_id: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeInput {
    pub session_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub context: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeOutput {
    pub status: PaymentSessionStatus,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureInput {
    pub payment_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureOutput {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundInput {
    pub payment_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundOutput {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelInput {
    pub payment_id: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOutput {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveInput {
    pub reference_id: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveOutput {
    pub status: PaymentSessionStatus,
    pub amount: Decimal,
    pub currency_code: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInput {
    pub reference_id: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedEvent {
    pub provider_event_id: String,
    pub event_type: String,
    pub provider_created_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: String,
    pub method_type: String,
    pub last4: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub brand: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountHolderInput {
    pub email: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountHolderInput {
    pub external_id: String,
    pub email: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveAccountHolderInput {
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAccountHolderInput {
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHolderOutput {
    pub external_id: String,
    pub email: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMethodsInput {
    pub account_holder_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMethodInput {
    pub account_holder_id: String,
    pub payment_method_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMethodInput {
    pub payment_method_id: String,
}

// ── 18-Method Provider Traits (PAYMENTS §4) ──────────────────────────────────

#[async_trait]
pub trait PaymentProvider: Send + Sync + 'static {
    fn id(&self) -> &str;

    // ---- session lifecycle ----
    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput>;
    async fn update(&self, i: UpdateInput) -> Result<UpdateOutput>;
    async fn delete(&self, i: DeleteInput) -> Result<()>;

    // ---- money movement ----
    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput>;
    async fn capture(&self, i: CaptureInput) -> Result<CaptureOutput>;
    async fn refund(&self, i: RefundInput) -> Result<RefundOutput>;
    async fn cancel(&self, i: CancelInput) -> Result<CancelOutput>;

    // ---- reads ----
    async fn retrieve(&self, i: RetrieveInput) -> Result<RetrieveOutput>;
    async fn get_status(&self, i: StatusInput) -> Result<PaymentSessionStatus>;

    // ---- inbound ----
    /// The HTTP header carrying this provider's webhook signature, if it signs in a
    /// header at all. The hooks route reads it and hands the value to `verify_webhook`.
    ///
    /// A provider that authenticates its callback some other way — SSLCommerz validates
    /// an IPN by posting the payload back — returns `None` and does its own checking
    /// inside `verify_webhook`. Returning `None` is a claim that the body is
    /// self-authenticating, never a licence to skip verification.
    fn signature_header(&self) -> Option<&'static str> {
        None
    }

    fn verify_webhook(&self, raw: &[u8], signature_header: Option<&str>) -> Result<VerifiedEvent>;
    fn map_webhook(&self, event: &VerifiedEvent) -> Result<WebhookAction>;

    // ---- capability probe ----
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }
}

#[async_trait]
pub trait PaymentVault: PaymentProvider {
    async fn create_account_holder(
        &self,
        i: CreateAccountHolderInput,
    ) -> Result<AccountHolderOutput>;
    async fn update_account_holder(
        &self,
        i: UpdateAccountHolderInput,
    ) -> Result<AccountHolderOutput>;
    async fn retrieve_account_holder(
        &self,
        i: RetrieveAccountHolderInput,
    ) -> Result<AccountHolderOutput>;
    async fn delete_account_holder(&self, i: DeleteAccountHolderInput) -> Result<()>;
    async fn list_payment_methods(&self, i: ListMethodsInput) -> Result<Vec<PaymentMethod>>;
    async fn save_payment_method(&self, i: SaveMethodInput) -> Result<PaymentMethod>;
    async fn delete_payment_method(&self, i: DeleteMethodInput) -> Result<()>;
}

// ── Payment DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureDTO {
    pub id: String,
    pub payment_id: String,
    pub amount: Decimal,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundDTO {
    pub id: String,
    pub payment_id: String,
    pub amount: Decimal,
    pub refund_reason_id: Option<String>,
    pub note: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundReasonDTO {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDTO {
    pub id: String,
    pub payment_collection_id: String,
    pub payment_session_id: Option<String>,
    pub provider_id: String,
    pub currency_code: String,
    pub amount: Decimal,
    pub captured_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub captures: Vec<CaptureDTO>,
    pub refunds: Vec<RefundDTO>,
    pub captured_amount: Decimal,
    pub refunded_amount: Decimal,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSessionDTO {
    pub id: String,
    pub payment_collection_id: String,
    pub provider_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub status: PaymentSessionStatus,
    pub data: Option<serde_json::Value>,
    pub context: Option<serde_json::Value>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub status_observed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCollectionDTO {
    pub id: String,
    pub currency_code: String,
    pub amount: Decimal,
    pub authorized_amount: Decimal,
    pub captured_amount: Decimal,
    pub refunded_amount: Decimal,
    pub status: String,
    pub payment_sessions: Vec<PaymentSessionDTO>,
    pub payments: Vec<PaymentDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHolderDTO {
    pub id: String,
    pub provider_id: String,
    pub external_id: String,
    pub email: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProviderRequestDTO {
    pub id: String,
    pub idempotency_key: String,
    pub provider_id: String,
    pub operation: String,
    pub resource_type: String,
    pub resource_id: String,
    pub request_hash: String,
    pub state: String,
    pub response: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub attempts: i32,
    pub first_sent_at: DateTime<Utc>,
    pub settled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentWebhookEventDTO {
    pub id: String,
    pub provider_id: String,
    pub provider_event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub signature_verified: bool,
    pub provider_created_at: DateTime<Utc>,
    pub state: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Request Inputs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentCollectionInput {
    pub currency_code: String,
    pub amount: Decimal,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentSessionInput {
    pub payment_collection_id: String,
    pub provider_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub data: Option<serde_json::Value>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizePaymentSessionInput {
    pub session_id: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturePaymentInput {
    pub payment_id: String,
    pub amount: Option<Decimal>, // None means full remaining authorized amount
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundPaymentInput {
    pub payment_id: String,
    pub amount: Decimal,
    pub refund_reason_id: Option<String>,
    pub note: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnsettledReconcileReport {
    /// Requests old enough to be considered stranded.
    pub scanned: usize,
    /// Converged to `succeeded` — the provider confirmed the money moved.
    pub converged: usize,
    /// Provider says it did not happen; the request is now `failed`.
    pub resolved_failed: usize,
    /// Still ambiguous after asking. These are the ones a human must look at.
    pub still_unsettled: usize,
}

// ── Reconciliation DTOs (PAYMENTS §9, Task P8-22) ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PspReportItem {
    pub provider_payment_id: String,
    pub payment_id: Option<String>,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub fee: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationDiscrepancy {
    pub payment_id: String,
    pub provider_payment_id: String,
    pub field: String,
    pub internal_value: String,
    pub psp_value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReportDTO {
    pub id: String,
    pub provider_id: String,
    pub total_records_checked: usize,
    pub matched_count: usize,
    pub discrepancy_count: usize,
    pub discrepancies: Vec<ReconciliationDiscrepancy>,
    pub generated_at: DateTime<Utc>,
}
