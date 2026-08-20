use chrono::Utc;
use stheno_db::{tenant, CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Result};

use crate::dto::{NewTenant, SubscriptionDTO, SubscriptionPatch, TenantDTO, TierDefinitionDTO};
use crate::models::{Subscription, Tenant, TenantDomain, TierDefinitionRow};

#[derive(Clone)]
pub struct PlatformService {
    tenants: PgCrudRepository<Tenant>,
    domains: PgCrudRepository<TenantDomain>,
    subscriptions: PgCrudRepository<Subscription>,
    tier_definitions: PgCrudRepository<TierDefinitionRow>,
}

impl PlatformService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            tenants: PgCrudRepository::new(pool.clone()),
            domains: PgCrudRepository::new(pool.clone()),
            subscriptions: PgCrudRepository::new(pool.clone()),
            tier_definitions: PgCrudRepository::new(pool),
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

    pub async fn get_tenant(&self, id: &str) -> Result<TenantDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.tenants.retrieve(None, id).await
        })
        .await
        .map(TenantDTO::from)
    }

    pub async fn list_tenants(&self) -> Result<Vec<TenantDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.tenants.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(TenantDTO::from).collect())
    }

    pub async fn get_subscription(&self, tenant_id: &str) -> Result<SubscriptionDTO> {
        self.subscription_row(tenant_id)
            .await
            .map(SubscriptionDTO::from)
    }

    /// The stored row, for the read-modify-write in [`Self::update_subscription`]. Private:
    /// the entity is a persistence detail and does not cross the module boundary.
    async fn subscription_row(&self, tenant_id: &str) -> Result<Subscription> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.subscriptions.retrieve(None, tenant_id).await
        })
        .await
    }

    pub async fn list_subscriptions(&self) -> Result<Vec<SubscriptionDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new().with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.subscriptions.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(SubscriptionDTO::from).collect())
    }

    /// Changes a tenant's plan. The tier name is validated against the effective tier
    /// definitions by the caller, which owns them.
    pub async fn set_tier(&self, tenant_id: &str, tier: &str) -> Result<SubscriptionDTO> {
        self.update_subscription(
            tenant_id,
            SubscriptionPatch {
                tier: Some(tier.to_string()),
                ..Default::default()
            },
        )
        .await
    }

    /// Applies a patch to a tenant's subscription.
    ///
    /// Read-modify-write rather than a targeted UPDATE, which is safe here only because
    /// the control plane is the single writer of this row and edits are operator-paced.
    // ponytail: last-write-wins on concurrent operator edits; add an updated_at
    // precondition to the UPDATE if two operators ever edit the same merchant at once.
    pub async fn update_subscription(
        &self,
        tenant_id: &str,
        patch: SubscriptionPatch,
    ) -> Result<SubscriptionDTO> {
        let mut sub = self.subscription_row(tenant_id).await?;

        if let Some(tier) = patch.tier {
            sub.tier = tier;
        }
        if let Some(status) = patch.status {
            sub.status = status;
        }
        if let Some(at) = patch.trial_started_at {
            sub.trial_started_at = at;
        }
        if let Some(at) = patch.trial_ends_at {
            sub.trial_ends_at = at;
        }
        if let Some(at) = patch.term_ends_at {
            sub.term_ends_at = at;
        }
        if let Some(v) = patch.quota_overrides {
            sub.quota_overrides = v;
        }
        if let Some(v) = patch.window_overrides {
            sub.window_overrides = v;
        }
        if let Some(v) = patch.feature_overrides {
            sub.feature_overrides = v;
        }
        sub.updated_at = Utc::now();

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.subscriptions.update(None, tenant_id, sub).await
        })
        .await
        .map(SubscriptionDTO::from)
    }

    /// Stored plan definitions. An empty result is the normal state: it means every plan
    /// still means what `config/tiers.toml` says.
    pub async fn list_tier_definitions(&self) -> Result<Vec<TierDefinitionDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new().with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.tier_definitions.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(TierDefinitionDTO::from).collect())
    }

    /// Writes what a plan means. Keys are validated by the caller against the entitlement
    /// enums before they reach here.
    pub async fn upsert_tier_definition(
        &self,
        definition: TierDefinitionDTO,
    ) -> Result<TierDefinitionDTO> {
        let row = TierDefinitionRow::from(definition);
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.tier_definitions.upsert(None, row).await
        })
        .await
        .map(TierDefinitionDTO::from)
    }

    /// Returns a plan to the shipped default by removing its override row.
    pub async fn reset_tier_definition(&self, tier: &str) -> Result<()> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.tier_definitions.delete(None, tier).await
        })
        .await
    }

    /// Registers a tenant, its subscription, and optionally its domain.
    pub async fn create_tenant(&self, input: NewTenant) -> Result<TenantDTO> {
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
                        feature_overrides: serde_json::json!({}),
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

        Ok(TenantDTO::from(created))
    }
}
