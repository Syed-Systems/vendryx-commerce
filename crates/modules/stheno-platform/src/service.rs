use chrono::Utc;
use stheno_db::{tenant, CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Result};

use crate::models::{Subscription, Tenant, TenantDomain};

/// What is needed to stand up a new merchant.
#[derive(Debug, Clone)]
pub struct NewTenant {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub domain: Option<String>,
}

#[derive(Clone)]
pub struct PlatformService {
    tenants: PgCrudRepository<Tenant>,
    domains: PgCrudRepository<TenantDomain>,
    subscriptions: PgCrudRepository<Subscription>,
}

impl PlatformService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            tenants: PgCrudRepository::new(pool.clone()),
            domains: PgCrudRepository::new(pool.clone()),
            subscriptions: PgCrudRepository::new(pool),
        }
    }

    /// Resolves a Host header to a tenant id.
    ///
    /// An unknown host is `NotFound`, never a fallback to a default tenant. A
    /// "fall back to the default" branch here is how a leak starts.
    pub async fn tenant_for_domain(&self, domain: &str) -> Result<String> {
        let domain = domain.to_ascii_lowercase();
        let found = tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.domains.retrieve(None, &domain).await
        })
        .await?;
        Ok(found.tenant_id)
    }

    pub async fn get_tenant(&self, id: &str) -> Result<Tenant> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.tenants.retrieve(None, id).await
        })
        .await
    }

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.tenants.list(None, &spec).await
        })
        .await
    }

    pub async fn get_subscription(&self, tenant_id: &str) -> Result<Subscription> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.subscriptions.retrieve(None, tenant_id).await
        })
        .await
    }

    pub async fn list_subscriptions(&self) -> Result<Vec<Subscription>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new().with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.subscriptions.list(None, &spec).await
        })
        .await
    }

    /// Changes a tenant's plan. The tier name is validated against `config/tiers.toml`
    /// by the caller, which owns the tier definitions.
    pub async fn set_tier(&self, tenant_id: &str, tier: &str) -> Result<Subscription> {
        let mut sub = self.get_subscription(tenant_id).await?;
        sub.tier = tier.to_string();
        sub.updated_at = Utc::now();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.subscriptions.update(None, tenant_id, sub).await
        })
        .await
    }

    /// Registers a tenant, its subscription, and optionally its domain.
    pub async fn create_tenant(&self, input: NewTenant) -> Result<Tenant> {
        tenant::validate(&input.id)?;
        let now = Utc::now();

        let tenant_row = Tenant {
            id: input.id.clone(),
            name: input.name,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = tenant::scope(tenant::PLATFORM_SCOPE, async {
            let created = self.tenants.create(None, tenant_row).await?;

            self.subscriptions
                .create(
                    None,
                    Subscription {
                        tenant_id: input.id.clone(),
                        tier: input.tier,
                        status: "active".to_string(),
                        trial_started_at: None,
                        trial_ends_at: None,
                        term_ends_at: None,
                        quota_overrides: serde_json::json!({}),
                        window_overrides: serde_json::json!({}),
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                    },
                )
                .await?;

            if let Some(domain) = input.domain {
                self.domains
                    .create(
                        None,
                        TenantDomain {
                            domain: domain.to_ascii_lowercase(),
                            tenant_id: input.id.clone(),
                            is_primary: true,
                            created_at: now,
                            deleted_at: None,
                        },
                    )
                    .await?;
            }

            Ok::<_, Error>(created)
        })
        .await?;

        Ok(created)
    }
}
