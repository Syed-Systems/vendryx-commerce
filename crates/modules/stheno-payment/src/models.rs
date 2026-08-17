//! Payment entity models (DATA_MODEL §2.7, PAYMENTS §2).
//! 8 core entities + 3 Stheno safety tables.
//! Strictly `pub(crate)` (AGENTS.md §2.9).
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PaymentProvider {
    pub id: String, // e.g. "pp_stripe_stripe"
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RefundReason {
    pub id: Id<RefundReason>,
    pub label: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Capture {
    pub id: Id<Capture>,
    pub payment_id: String,
    pub amount: Decimal,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Refund {
    pub id: Id<Refund>,
    pub payment_id: String,
    pub amount: Decimal,
    pub refund_reason_id: Option<String>,
    pub note: Option<String>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub deleted_at: Option<DateTime<Utc>>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    AccountHolder => "acch",
    Capture => "capt",
    Payment => "pay",
    PaymentCollection => "pay_col",
    PaymentProviderRequest => "ppreq",
    PaymentReconciliationRun => "prec",
    PaymentSession => "payses",
    PaymentWebhookEvent => "pwhevt",
    Refund => "ref",
    RefundReason => "refr",
}

macro_rules! impl_crud_entity {
    ($t:ident, $table:literal, $entity:literal) => {
        impl CrudEntity for $t {
            const TABLE: &'static str = $table;
            const ENTITY: &'static str = $entity;

            fn id(&self) -> String {
                self.id.to_string()
            }
            fn is_deleted(&self) -> bool {
                self.deleted_at.is_some()
            }
            fn set_deleted(&mut self, deleted: bool) {
                self.deleted_at = if deleted { Some(Utc::now()) } else { None };
            }
        }
    };
}

impl_crud_entity!(
    PaymentCollection,
    "payment.payment_collection",
    "payment_collection"
);
impl_crud_entity!(PaymentSession, "payment.payment_session", "payment_session");
impl_crud_entity!(Payment, "payment.payment", "payment");
impl_crud_entity!(RefundReason, "payment.refund_reason", "refund_reason");
impl_crud_entity!(Capture, "payment.capture", "capture");
impl_crud_entity!(Refund, "payment.refund", "refund");
impl_crud_entity!(AccountHolder, "payment.account_holder", "account_holder");
impl_crud_entity!(
    PaymentProviderRequest,
    "payment.payment_provider_request",
    "payment_provider_request"
);
impl_crud_entity!(
    PaymentWebhookEvent,
    "payment.payment_webhook_event",
    "payment_webhook_event"
);
impl_crud_entity!(
    PaymentReconciliationRun,
    "payment.payment_reconciliation_run",
    "payment_reconciliation_run"
);

impl CrudEntity for PaymentProvider {
    const TABLE: &'static str = "payment.payment_provider";
    const ENTITY: &'static str = "payment_provider";

    fn id(&self) -> String {
        self.id.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}
