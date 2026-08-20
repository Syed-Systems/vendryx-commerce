//! Boundary Data Transfer Objects (DTOs) for `stheno-platform`.
//!
//! The `models` in this crate are persistence entities: they carry `deleted_at`, they name
//! their table, and they exist to be round-tripped through `PgCrudRepository`. Callers get
//! these instead (AGENTS.md §3.4), so a change to how a tenant is *stored* is not
//! automatically a change to what every caller sees.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{Subscription, Tenant, TierDefinitionRow};

/// One merchant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDTO {
    pub id: String,
    pub name: String,
    pub status: String,
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
