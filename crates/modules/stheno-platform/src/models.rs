use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;

/// One merchant. Under ADR-009 this was an entire deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: String,
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
/// exceptions — the persisted equivalent of the additive env overrides in
/// ENTITLEMENTS.md 5b.
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
