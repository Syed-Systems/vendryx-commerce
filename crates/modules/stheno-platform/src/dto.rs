//! Boundary Data Transfer Objects (DTOs) for `stheno-platform`.
//!
//! The `models` in this crate are persistence entities: they carry `deleted_at`, they name
//! their table, and they exist to be round-tripped through `PgCrudRepository`. Callers get
//! these instead (AGENTS.md §3.4), so a change to how a tenant is *stored* is not
//! automatically a change to what every caller sees.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_types::Money;

use crate::models::{
    BillingAttempt, Invoice, Operator, OperatorAudit, Subscription, Tenant, TierDefinitionRow,
};

/// One merchant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDTO {
    pub id: String,
    pub name: String,
    pub status: String,
    pub status_reason: Option<String>,
    pub status_changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// What is needed to stand up a new merchant.
#[derive(Debug, Clone)]
pub struct NewTenant {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub domain: Option<String>,
}

/// What a merchant is entitled to.
///
/// `tier` names a plan; what that plan *means* is `config/tiers.toml` overlaid with
/// `platform.tier_definition` (ADR-016). This is the assignment plus the exceptions that
/// apply to this merchant alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDTO {
    pub tenant_id: String,
    pub tier: String,
    pub status: String,
    pub trial_started_at: Option<DateTime<Utc>>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub term_ends_at: Option<DateTime<Utc>>,
    /// `{"max_products": 250}` — a negotiated limit for this merchant.
    pub quota_overrides: serde_json::Value,
    pub window_overrides: serde_json::Value,
    /// `{"payment_gateway": true}` grants; an explicit `false` revokes.
    pub feature_overrides: serde_json::Value,
    pub dunning_attempts: i32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The editable surface of a subscription.
///
/// Every field is "absent means leave it alone". The date fields are doubly optional so
/// that *clearing* a trial end date and *not mentioning* it stay distinguishable — a patch
/// that only changes `status` must not silently end someone's trial.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionPatch {
    pub tier: Option<String>,
    pub status: Option<String>,
    pub trial_started_at: Option<Option<DateTime<Utc>>>,
    pub trial_ends_at: Option<Option<DateTime<Utc>>>,
    pub term_ends_at: Option<Option<DateTime<Utc>>>,
    pub quota_overrides: Option<serde_json::Value>,
    pub window_overrides: Option<serde_json::Value>,
    pub feature_overrides: Option<serde_json::Value>,
}

/// What a plan means on this deployment, when it has been changed from the shipped default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinitionDTO {
    pub tier: String,
    /// `["csv_export", "payment_gateway"]`
    pub features: serde_json::Value,
    /// `{"max_products": 100}` — 0 is the unlimited sentinel, same as the TOML.
    pub quotas: serde_json::Value,
    pub windows: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

impl From<Tenant> for TenantDTO {
    fn from(row: Tenant) -> Self {
        Self {
            id: row.id,
            name: row.name,
            status: row.status,
            status_reason: row.status_reason,
            status_changed_at: row.status_changed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<Subscription> for SubscriptionDTO {
    fn from(row: Subscription) -> Self {
        Self {
            tenant_id: row.tenant_id,
            tier: row.tier,
            status: row.status,
            trial_started_at: row.trial_started_at,
            trial_ends_at: row.trial_ends_at,
            term_ends_at: row.term_ends_at,
            quota_overrides: row.quota_overrides,
            window_overrides: row.window_overrides,
            feature_overrides: row.feature_overrides,
            dunning_attempts: row.dunning_attempts,
            next_attempt_at: row.next_attempt_at,
            current_period_start: row.current_period_start,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<TierDefinitionRow> for TierDefinitionDTO {
    fn from(row: TierDefinitionRow) -> Self {
        Self {
            tier: row.tier,
            features: row.features,
            quotas: row.quotas,
            windows: row.windows,
            updated_at: row.updated_at,
        }
    }
}

impl From<TierDefinitionDTO> for TierDefinitionRow {
    fn from(dto: TierDefinitionDTO) -> Self {
        Self {
            tier: dto.tier,
            features: dto.features,
            quotas: dto.quotas,
            windows: dto.windows,
            updated_at: dto.updated_at,
        }
    }
}

/// A control-plane credential, as anyone but its holder sees it.
///
/// There is no field for the token. It exists exactly once, in
/// [`IssuedOperator::token`], and is unrecoverable after that response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorDTO {
    pub id: String,
    pub name: String,
    /// `op_...abcd`.
    pub redacted_token: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl OperatorDTO {
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }
}

/// The one and only time the plaintext token is available.
#[derive(Debug, Clone, Serialize)]
pub struct IssuedOperator {
    pub operator: OperatorDTO,
    /// Shown once. Not stored, not logged, not written to the audit trail.
    pub token: String,
}

/// One control-plane write, and who made it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorAuditDTO {
    pub id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// What to record about a control-plane write.
#[derive(Debug, Clone)]
pub struct NewOperatorAudit {
    pub actor_id: String,
    pub actor_name: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<serde_json::Value>,
}

/// The data needed to generate one base subscription invoice.
#[derive(Debug, Clone)]
pub struct GenerateInvoiceInput {
    pub tenant_id: String,
    pub tier: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub amount: Money,
}

/// One subscription invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvoiceDTO {
    pub id: String,
    pub tenant_id: String,
    pub tier: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub subtotal: Money,
    pub tax_total: Money,
    pub credit_total: Money,
    pub total: Money,
    pub status: String,
    pub issued_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

/// Credits and discounts to apply to an existing invoice.
#[derive(Debug, Clone)]
pub struct ApplyInvoiceCreditsInput {
    pub invoice_id: String,
    pub credit: Money,
    pub discount: Money,
    pub coupon: Money,
    pub reason: String,
}

/// One sold plan price available to a billing run.
#[derive(Debug, Clone)]
pub struct BillingPlanPrice {
    pub tier: String,
    pub amount: Money,
    pub interval_months: u32,
}

/// The scheduled billing run input.
#[derive(Debug, Clone)]
pub struct BillingRunInput {
    pub now: DateTime<Utc>,
    pub plan_prices: Vec<BillingPlanPrice>,
}

/// Summary of one scheduled billing run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BillingRunDTO {
    pub invoices_created: usize,
    pub unsold_skipped: usize,
}

/// Current-period bounds for revenue metrics.
#[derive(Debug, Clone)]
pub struct RevenueMetricsInput {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Revenue metrics derived from collected invoices only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevenueMetricsDTO {
    pub mrr: Money,
    pub arr: Money,
    pub expansion: Money,
    pub churned_tenants: usize,
}

/// The data needed to compute mid-cycle plan-change proration.
#[derive(Debug, Clone)]
pub struct CalculateProrationInput {
    pub old_tier: String,
    pub new_tier: String,
    pub period_start: DateTime<Utc>,
    pub change_at: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub old_period_amount: Money,
    pub new_period_amount: Money,
}

/// One signed invoice line produced by a proration calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProrationLineDTO {
    pub kind: String,
    pub description: String,
    pub amount: Money,
}

/// Credit for the unused old plan and charge for the unused new-plan remainder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProrationDTO {
    pub credit: Money,
    pub charge: Money,
    pub net_adjustment: Money,
    pub lines: Vec<ProrationLineDTO>,
}

/// The taxable invoice base and VAT rate for one invoice.
#[derive(Debug, Clone)]
pub struct CalculateVatInput {
    pub taxable_subtotal: Money,
    pub rate: Decimal,
}

/// Bangladesh VAT line metadata for the invoice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VatLineDTO {
    pub kind: String,
    pub jurisdiction: String,
    pub mushak_form: String,
    pub rate: Decimal,
    pub amount: Money,
}

/// The durable row written before attempting subscription collection.
#[derive(Debug, Clone)]
pub struct StartBillingAttemptInput {
    pub invoice_id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub amount: Money,
}

/// One subscription collection attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BillingAttemptDTO {
    pub id: String,
    pub invoice_id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub state: String,
    pub amount: Money,
    pub provider_reference: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The clocked input for one dunning transition.
#[derive(Debug, Clone)]
pub struct DunningInput {
    pub tenant_id: String,
    pub now: DateTime<Utc>,
}

/// One outbound notification the dunning runner should send.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DunningNotificationDTO {
    pub channel: String,
    pub template: String,
}

impl From<Operator> for OperatorDTO {
    fn from(row: Operator) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            redacted_token: row.redacted_token,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            revoked_at: row.revoked_at,
        }
    }
}

impl From<OperatorAudit> for OperatorAuditDTO {
    fn from(row: OperatorAudit) -> Self {
        Self {
            id: row.id.to_string(),
            actor_id: row.actor_id,
            actor_name: row.actor_name,
            action: row.action,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            diff: row.diff,
            created_at: row.created_at,
        }
    }
}

impl From<Invoice> for InvoiceDTO {
    fn from(row: Invoice) -> Self {
        let currency = row.currency;
        Self {
            id: row.id.to_string(),
            tenant_id: row.tenant_id,
            tier: row.tier,
            period_start: row.period_start,
            period_end: row.period_end,
            subtotal: Money::new(row.subtotal, currency),
            tax_total: Money::new(row.tax_total, currency),
            credit_total: Money::new(row.credit_total, currency),
            total: Money::new(row.total, currency),
            status: row.status,
            issued_at: row.issued_at,
            paid_at: row.paid_at,
        }
    }
}

impl From<BillingAttempt> for BillingAttemptDTO {
    fn from(row: BillingAttempt) -> Self {
        let currency = row.currency;
        Self {
            id: row.id.to_string(),
            invoice_id: row.invoice_id,
            tenant_id: row.tenant_id,
            provider_id: row.provider_id,
            idempotency_key: row.idempotency_key,
            request_hash: row.request_hash,
            state: row.state,
            amount: Money::new(row.amount, currency),
            provider_reference: row.provider_reference,
            error: row.error,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// A custom or default domain mapped to a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDTO {
    pub domain: String,
    pub tenant_id: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

impl From<crate::models::TenantDomain> for DomainDTO {
    fn from(row: crate::models::TenantDomain) -> Self {
        Self {
            domain: row.domain,
            tenant_id: row.tenant_id,
            is_primary: row.is_primary,
            created_at: row.created_at,
        }
    }
}

/// Verification state for a custom domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerificationDTO {
    pub domain: String,
    pub dns_pointed: bool,
    pub tls_issued: bool,
    pub status: String,
}

/// Outcome of a closed-tenant hard purge sweep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantPurgeResultDTO {
    pub purged_count: usize,
    pub purged_tenant_ids: Vec<String>,
}

/// Full data bundle for a tenant export request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantExportBundleDTO {
    pub tenant: TenantDTO,
    pub subscription: Option<SubscriptionDTO>,
    pub domains: Vec<DomainDTO>,
    pub invoices: Vec<InvoiceDTO>,
    pub billing_attempts: Vec<BillingAttemptDTO>,
    pub exported_at: DateTime<Utc>,
}

/// Health metrics computed cross-tenant for a merchant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantHealthDTO {
    pub tenant_id: String,
    pub orders_last_7d: i64,
    pub gmv_last_7d: Money,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub health_verdict: String,
}

/// Onboarding funnel stage for a merchant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingFunnelDTO {
    pub tenant_id: String,
    pub stage: String,
    pub created: bool,
    pub product_added: bool,
    pub payment_connected: bool,
    pub first_order_received: bool,
}

/// Filter, search and pagination parameters for tenant listings.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TenantListQuery {
    pub search: Option<String>,
    pub tier: Option<String>,
    pub status: Option<String>,
    pub before: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedTenantsDTO {
    pub tenants: Vec<TenantDTO>,
    pub count: usize,
    pub next_cursor: Option<String>,
}

/// Strict aggregate-only cross-tenant metric (P33-01 & P33-02).
/// Deliberately refuses to carry row-level merchant data out of the platform kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossTenantAggregateDTO {
    pub total_merchants: usize,
    pub active_merchants: usize,
    pub total_subscriptions: usize,
    pub total_invoices_issued: usize,
    pub total_gmv: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmvMetricsDTO {
    pub daily_gmv: Money,
    pub weekly_gmv: Money,
    pub monthly_gmv: Money,
    pub total_gmv: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelMetricsDTO {
    pub total_signups: usize,
    pub activated_merchants: usize,
    pub paid_subscribers: usize,
    pub activation_rate: f64,
    pub paid_conversion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDistributionItemDTO {
    pub tier: String,
    pub count: usize,
    pub percentage: f64,
    pub mrr: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopMerchantGmvDTO {
    pub tenant_id: String,
    pub name: String,
    pub tier: String,
    pub gmv_30d: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnCohortDTO {
    pub signup_month: String,
    pub initial_count: usize,
    pub active_count: usize,
    pub churned_count: usize,
    pub retention_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaHeadroomDTO {
    pub tenant_id: String,
    pub name: String,
    pub tier: String,
    pub metric: String,
    pub usage: u64,
    pub limit: u64,
    pub usage_percentage: f64,
    pub is_near_limit: bool,
}
