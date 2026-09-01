use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::currency::CurrencyCode;
use stheno_types::Id;

/// One merchant. Under ADR-009 this was an entire deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: String,
    pub status_reason: Option<String>,
    pub status_changed_at: Option<DateTime<Utc>>,
    /// Emergency incident lock (P34-09). Tenant state, alongside `status`, rather than a
    /// side table — the two must not be able to disagree about one merchant.
    pub is_read_only: bool,
    pub read_only_reason: Option<String>,
    pub read_only_since: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Tenant {
    const TABLE: &'static str = "platform.tenant";
    const ENTITY: &'static str = "tenant";

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

/// Host → tenant. The storefront is unauthenticated, so the Host header is the only
/// tenant signal available on `/store/*`; a JWT can only ever cross-check it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDomain {
    pub domain: String,
    pub tenant_id: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TenantDomain {
    const TABLE: &'static str = "platform.tenant_domain";
    const ENTITY: &'static str = "tenant_domain";
    const ID_COLUMN: &'static str = "domain";

    fn id(&self) -> String {
        self.domain.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

/// What a tenant is entitled to.
///
/// `tier` names a block in `config/tiers.toml`, which stays the source of truth for what
/// a tier *means*. This row records only the assignment plus negotiated per-tenant
/// exceptions — the persisted equivalent of the additive env overrides in the tier doc §5b.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub tenant_id: String,
    pub tier: String,
    pub status: String,
    pub trial_started_at: Option<DateTime<Utc>>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub term_ends_at: Option<DateTime<Utc>>,
    pub quota_overrides: serde_json::Value,
    pub window_overrides: serde_json::Value,
    /// `{"payment_gateway": true}` — a negotiated grant or revocation for this one
    /// merchant, applied after the tier resolves.
    pub feature_overrides: serde_json::Value,
    pub dunning_attempts: i32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Subscription {
    const TABLE: &'static str = "platform.subscription";
    const ENTITY: &'static str = "subscription";
    const ID_COLUMN: &'static str = "tenant_id";

    fn id(&self) -> String {
        self.tenant_id.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

/// What a plan means, when the operator has changed it from the shipped default.
///
/// `config/tiers.toml` is still the default. A row here replaces that tier's definition
/// wholesale so a plan can be retuned from the control plane without a release, which a
/// container-baked config file cannot offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinitionRow {
    pub tier: String,
    /// `["csv_export", "payment_gateway"]`
    pub features: serde_json::Value,
    /// `{"max_products": 100}` — 0 is the unlimited sentinel, same as the TOML.
    pub quotas: serde_json::Value,
    pub windows: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

impl CrudEntity for TierDefinitionRow {
    const TABLE: &'static str = "platform.tier_definition";
    const ENTITY: &'static str = "tier_definition";
    const ID_COLUMN: &'static str = "tier";

    fn id(&self) -> String {
        self.tier.clone()
    }
    // No soft delete: a tier definition is either overridden or it is not. Deleting the
    // row is how an operator returns a plan to the shipped default, and a soft-deleted
    // row would keep overriding it.
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

/// A person who holds the control plane, not a merchant's user.
///
/// One shared `STHENO_PLATFORM_TOKEN` cannot answer "who changed this merchant's tier",
/// cannot be revoked for one person, and cannot be rotated without locking everyone out
/// at once. A row here is one named credential that can do all three.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    pub id: Id<Operator>,
    pub name: String,
    /// SHA-256 of the issued token. The plaintext exists once, in the response to the
    /// call that created it, and is never recoverable afterwards.
    pub token_hash: String,
    /// `op_...abcd` — enough to tell two credentials apart, not enough to use one.
    pub redacted_token: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    /// `owner`, `billing`, `support` or `engineer`. On the operator row, not in a side
    /// map: an unknown operator then has no row at all, so the absent case is NotFound
    /// and denies. The in-memory version this replaced returned `owner` for an id it had
    /// never seen, which made the absent case the *most* privileged one.
    pub role: String,
    /// Set at enrolment. Enforcement waits for `mfa_enabled`, so an operator who starts
    /// setup and never scans the code is not locked out of their own control plane.
    pub mfa_secret: Option<String>,
    pub mfa_enabled: bool,
    /// Empty means no restriction; `[]` rather than NULL so that has one representation.
    pub ip_allowlist: serde_json::Value,
    pub session_ttl_minutes: Option<i32>,
    /// Always `None`. The repository predicates every `retrieve` on `deleted_at IS NULL`,
    /// so the column has to exist; retiring an operator is `revoked_at`.
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Operator {
    const TABLE: &'static str = "platform.operator";
    const ENTITY: &'static str = "operator";

    fn id(&self) -> String {
        self.id.to_string()
    }
    // Revocation is `revoked_at`, not `deleted_at`: a revoked credential must stay
    // listed so an operator can see it is gone, and its audit rows still name it.
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

/// One control-plane write, and who made it.
///
/// Not `audit.audit_log`: that table is tenant-scoped and RLS-forced, so "everything this
/// operator changed" — a cross-tenant read — is impossible there by construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorAudit {
    pub id: Id<OperatorAudit>,
    pub actor_id: String,
    /// Denormalised so the trail still reads after a credential is renamed or revoked.
    pub actor_name: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub diff: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl CrudEntity for OperatorAudit {
    const TABLE: &'static str = "platform.operator_audit";
    const ENTITY: &'static str = "operator_audit";

    fn id(&self) -> String {
        self.id.to_string()
    }
    // Append-only: the service exposes no update and no delete, and the table has no
    // `deleted_at` for a soft one to write to.
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

/// One subscription bill for one merchant and one period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Id<Invoice>,
    pub tenant_id: String,
    pub tier: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub currency: CurrencyCode,
    pub subtotal: Decimal,
    pub tax_total: Decimal,
    pub credit_total: Decimal,
    pub total: Decimal,
    pub status: String,
    pub issued_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Invoice {
    const TABLE: &'static str = "platform.invoice";
    const ENTITY: &'static str = "invoice";

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

/// One signed line making up an invoice total.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: Id<InvoiceLine>,
    pub invoice_id: String,
    pub kind: String,
    pub description: String,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for InvoiceLine {
    const TABLE: &'static str = "platform.invoice_line";
    const ENTITY: &'static str = "invoice_line";

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

/// Outbound ledger row for collecting our subscription revenue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAttempt {
    pub id: Id<BillingAttempt>,
    pub invoice_id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub state: String,
    pub amount: Decimal,
    pub currency: CurrencyCode,
    pub provider_reference: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for BillingAttempt {
    const TABLE: &'static str = "platform.billing_attempt";
    const ENTITY: &'static str = "billing_attempt";

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

/// A named boolean the deployment owns: a gateway kill switch or a feature flag.
///
/// Keyed on the name rather than a generated id — there is exactly one row per flag, and
/// a surrogate key would let two rows claim the same switch. An absent row means enabled;
/// see the migration for why an override fails open while a credential fails closed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalFlag {
    /// A provider id (`pp_bkash`) for a gateway, a flag name for a feature.
    pub key: String,
    /// `gateway` or `feature`.
    pub kind: String,
    pub description: String,
    pub is_enabled: bool,
    pub rollout_percentage: i16,
    /// Why it was turned off. An incident switch with no reason is unreadable later.
    pub reason: Option<String>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for GlobalFlag {
    const TABLE: &'static str = "platform.global_flag";
    const ENTITY: &'static str = "global_flag";
    /// Natural key. Without this the repository builds its `ON CONFLICT` against a column
    /// named `id`, which this table does not have.
    const ID_COLUMN: &'static str = "key";

    fn id(&self) -> String {
        self.key.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

/// One time-boxed support session against one merchant (ADR-018).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpersonationSession {
    pub id: Id<ImpersonationSession>,
    /// SHA-256 of the issued token, never the token. It exists in plaintext once, in the
    /// response to the call that created it — same rule as [`Operator`].
    pub token_hash: String,
    /// `imp_...abcd` — enough to correlate a session, not enough to use one.
    pub redacted_token: String,
    pub tenant_id: String,
    pub operator_id: String,
    /// Required by ADR-018: a support session with no stated reason is not consented to.
    pub reason: String,
    /// Granted at issue time and stored, so narrowing the scope rules later cannot
    /// retroactively widen a session that is already live.
    pub scopes: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ImpersonationSession {
    /// Live means not ended by hand and not yet expired. Expiry is read from the row on
    /// every use rather than tracked by a timer, because a timer dies with its process —
    /// which is the bug this table exists to remove.
    pub fn is_live(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}

impl CrudEntity for ImpersonationSession {
    const TABLE: &'static str = "platform.impersonation_session";
    const ENTITY: &'static str = "impersonation_session";

    fn id(&self) -> String {
        self.id.to_string()
    }
    // Ending a session is `revoked_at`: the audit rows naming it must still resolve.
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

/// Which theme, at which pinned version, one merchant's storefront renders with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantTheme {
    pub tenant_id: String,
    pub theme_id: String,
    /// Pinned deliberately. A version that floats to latest changes a merchant's
    /// storefront with nobody deploying.
    pub version: String,
    pub is_custom: bool,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TenantTheme {
    const TABLE: &'static str = "platform.tenant_theme";
    const ENTITY: &'static str = "tenant_theme";
    /// One row per tenant, keyed on the tenant. See [`GlobalFlag`] for why this matters.
    const ID_COLUMN: &'static str = "tenant_id";

    fn id(&self) -> String {
        self.tenant_id.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

/// Deployment-wide defaults and messaging configuration. Exactly one row, `singleton`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub id: String,
    pub default_region: String,
    pub default_currency: String,
    /// `Decimal`, not `f64`. ADR-006 keeps floats away from anything that multiplies
    /// money, and this multiplies every order total in the deployment.
    pub default_tax_rate: Decimal,
    pub support_email: String,
    pub email_provider: String,
    pub sms_provider: String,
    pub from_email: String,
    pub order_confirmation_template: String,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl DeploymentConfig {
    /// The only id this table accepts, enforced by a CHECK in the migration.
    pub const SINGLETON: &'static str = "singleton";
}

impl CrudEntity for DeploymentConfig {
    const TABLE: &'static str = "platform.deployment_config";
    const ENTITY: &'static str = "deployment_config";

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

/// One broadcast to every merchant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: Id<Announcement>,
    pub title: String,
    pub message: String,
    /// `info`, `warning` or `critical`.
    pub severity: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Expiry is a column, never a cleanup job: a banner that outlives its incident is
    /// worse than one that never appeared, and deleting it would destroy the record of
    /// what merchants were told during an outage.
    pub expires_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Announcement {
    const TABLE: &'static str = "platform.announcement";
    const ENTITY: &'static str = "announcement";

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

/// Merchant identity and resumable setup information before a tenant exists.
///
/// The id is the authenticated merchant subject. It is deliberately not a tenant id: a merchant
/// can save a draft without creating a deployment, and an owner binding happens only at launch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantAccount {
    pub id: String,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub signup_draft: serde_json::Value,
    pub selected_tier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The durable handoff from a pre-tenant merchant account to the tenant it owns.
///
/// `tenant_id` is the primary key because one deployed tenant has exactly one accountable
/// merchant owner. A merchant may own more than one store, so it cannot be the key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantTenantOwner {
    pub merchant_id: String,
    pub tenant_id: String,
    pub bound_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for MerchantTenantOwner {
    const TABLE: &'static str = "platform.merchant_tenant_owner";
    const ENTITY: &'static str = "merchant_tenant_owner";
    const ID_COLUMN: &'static str = "tenant_id";

    fn id(&self) -> String {
        self.tenant_id.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

impl CrudEntity for MerchantAccount {
    const TABLE: &'static str = "platform.merchant_account";
    const ENTITY: &'static str = "merchant_account";
    const ID_COLUMN: &'static str = "id";

    fn id(&self) -> String {
        self.id.clone()
    }
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

/// One merchant request to create an isolated Stheno store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub id: Id<LaunchRequest>,
    pub merchant_id: String,
    pub store_name: String,
    pub subdomain: String,
    pub tier: String,
    pub payment_status: String,
    pub launch_status: String,
    pub logo_url: Option<String>,
    pub accent_color: String,
    pub icon_choice: String,
    pub buyer_accounts: bool,
    pub bespoke_brief: Option<String>,
    pub bespoke_assets: serde_json::Value,
    pub validation_findings: serde_json::Value,
    pub assigned_developer: Option<String>,
    pub tenant_id: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CrudEntity for LaunchRequest {
    const TABLE: &'static str = "platform.launch_request";
    const ENTITY: &'static str = "launch_request";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

/// Append-only customer and operator timeline for a launch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequestEvent {
    pub id: Id<LaunchRequestEvent>,
    pub launch_request_id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub action: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl CrudEntity for LaunchRequestEvent {
    const TABLE: &'static str = "platform.launch_request_event";
    const ENTITY: &'static str = "launch_request_event";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        false
    }
    fn set_deleted(&mut self, _deleted: bool) {}
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    Operator => "op",
    OperatorAudit => "opaud",
    Invoice => "inv",
    InvoiceLine => "invli",
    BillingAttempt => "billatt",
    ImpersonationSession => "imp",
    Announcement => "ann",
    LaunchRequest => "lr",
    LaunchRequestEvent => "lre",
}
