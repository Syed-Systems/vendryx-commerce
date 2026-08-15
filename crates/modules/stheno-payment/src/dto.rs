//! DTOs, State Machines, Money Boundary, and Provider Traits for `stheno-payment` (PAYMENTS §2-§6).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
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
