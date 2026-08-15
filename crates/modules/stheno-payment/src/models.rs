//! Payment entity models (DATA_MODEL §2.7, PAYMENTS §2).
//! 8 core entities + 3 Stheno safety tables.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentCollection {
    pub id: Id<PaymentCollection>,
    pub currency_code: String,
    pub amount: Decimal,
    pub authorized_amount: Decimal,
    pub captured_amount: Decimal,
    pub refunded_amount: Decimal,
    pub status: String, // "not_paid", "awaiting", "authorized", "partially_authorized", "canceled"
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentProvider {
    pub id: String, // e.g. "pp_stripe_stripe"
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentSession {
    pub id: Id<PaymentSession>,
    pub payment_collection_id: String,
    pub provider_id: String,
    pub amount: Decimal,
    pub currency_code: String,
    pub status: String, // 7 states: pending, pending_authorization, requires_more, authorized, captured, canceled, error
    pub data: Option<serde_json::Value>,
    pub context: Option<serde_json::Value>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub status_observed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Payment {
    pub id: Id<Payment>,
    pub payment_collection_id: String,
    pub payment_session_id: Option<String>,
    pub provider_id: String,
    pub currency_code: String,
    pub amount: Decimal,
    pub captured_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RefundReason {
    pub id: Id<RefundReason>,
    pub label: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Capture {
    pub id: Id<Capture>,
    pub payment_id: String,
    pub amount: Decimal,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Refund {
    pub id: Id<Refund>,
    pub payment_id: String,
    pub amount: Decimal,
    pub refund_reason_id: Option<String>,
    pub note: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AccountHolder {
    pub id: Id<AccountHolder>,
    pub provider_id: String,
    pub external_id: String,
    pub email: Option<String>,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ── 3 Stheno Safety Models (PAYMENTS §2.1, §6, §7, §9) ──────────────────────

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentProviderRequest {
    pub id: Id<PaymentProviderRequest>,
    pub idempotency_key: String,
    pub provider_id: String,
    pub operation: String, // "initiate", "authorize", "capture", "refund", "cancel"
    pub resource_type: String,
    pub resource_id: String,
    pub request_hash: String,
    pub state: String, // "in_flight", "succeeded", "failed", "unknown"
    pub response: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub attempts: i32,
    pub first_sent_at: DateTime<Utc>,
    pub settled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentWebhookEvent {
    pub id: Id<PaymentWebhookEvent>,
    pub provider_id: String,
    pub provider_event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub signature_verified: bool,
    pub provider_created_at: DateTime<Utc>,
    pub state: String, // "received", "processed", "ignored", "failed"
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PaymentReconciliationRun {
    pub id: Id<PaymentReconciliationRun>,
    pub provider_id: String,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub checked_count: i32,
    pub drift_count: i32,
    pub drifts: Option<serde_json::Value>,
    pub state: String, // "running", "clean", "drift_found", "failed"
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
