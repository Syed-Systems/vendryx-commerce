use std::collections::HashMap;

use chrono::{Duration, Months, Utc};
use rand::Rng;
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use stheno_db::{
    tenant, CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository,
};
use stheno_types::currency::CurrencyCode;
use stheno_types::{Error, Id, Money, Result};

use crate::dto::{
    ApplyInvoiceCreditsInput, BillingAttemptDTO, BillingPlanPrice, BillingRunDTO, BillingRunInput,
    CalculateProrationInput, CalculateVatInput, ChurnCohortDTO, CrossTenantAggregateDTO, DomainDTO,
    DomainVerificationDTO, DunningInput, DunningNotificationDTO, FunnelMetricsDTO,
    GenerateInvoiceInput, GmvMetricsDTO, InvoiceDTO, IssuedOperator, LaunchRequestDTO,
    LaunchRequestEventDTO, MerchantAccountDTO, MerchantHealthDTO, NewOperatorAudit, NewTenant,
    OnboardingFunnelDTO, OperatorAuditDTO, OperatorDTO, PagedTenantsDTO, PlanDistributionItemDTO,
    ProrationDTO, ProrationLineDTO, QuotaHeadroomDTO, RevenueMetricsDTO, RevenueMetricsInput,
    StartBillingAttemptInput, SubmitLaunchRequestInput, SubscriptionDTO, SubscriptionPatch,
    TenantDTO, TenantExportBundleDTO, TenantListQuery, TenantPurgeResultDTO, TierDefinitionDTO,
    TopMerchantGmvDTO, VatLineDTO, FxRateDTO,
};
use crate::models::{
    Announcement, BillingAttempt, DeploymentConfig, GlobalFlag, ImpersonationSession, Invoice,
    FxRate,
    InvoiceLine, LaunchRequest, LaunchRequestEvent, MerchantAccount, MerchantTenantOwner, Operator,
    OperatorAudit, Subscription, Tenant, TenantDomain, TenantTheme, TierDefinitionRow,
};

/// Every issued control-plane token starts with this, so one found in a log or a paste is
/// recognisable as what it is.
const OPERATOR_TOKEN_PREFIX: &str = "op_";

/// SHA-256, hex. The token is 32 random bytes — not guessable, so it needs no password
/// KDF. Same construction as `api_key.api_key`, which stores its secrets the same way.
fn hash_token(plain: &str) -> String {
    hex::encode(Sha256::digest(plain.as_bytes()))
}

#[derive(Clone)]
pub struct PlatformService {
    pool: DbPool,
    tenants: PgCrudRepository<Tenant>,
    domains: PgCrudRepository<TenantDomain>,
    subscriptions: PgCrudRepository<Subscription>,
    tier_definitions: PgCrudRepository<TierDefinitionRow>,
    operators: PgCrudRepository<Operator>,
    operator_audit: PgCrudRepository<OperatorAudit>,
    invoices: PgCrudRepository<Invoice>,
    invoice_lines: PgCrudRepository<InvoiceLine>,
    billing_attempts: PgCrudRepository<BillingAttempt>,
    // P37-02/03/08: state the control plane used to hold in process-global maps.
    global_flags: PgCrudRepository<GlobalFlag>,
    impersonation_sessions: PgCrudRepository<ImpersonationSession>,
    tenant_themes: PgCrudRepository<TenantTheme>,
    deployment_config: PgCrudRepository<DeploymentConfig>,
    announcements: PgCrudRepository<Announcement>,
    merchant_accounts: PgCrudRepository<MerchantAccount>,
    merchant_tenant_owners: PgCrudRepository<MerchantTenantOwner>,
    launch_requests: PgCrudRepository<LaunchRequest>,
    launch_request_events: PgCrudRepository<LaunchRequestEvent>,
    fx_rates: PgCrudRepository<crate::models::FxRate>,
    payment_proofs: PgCrudRepository<crate::models::InvoicePaymentProof>,
}

impl PlatformService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool: pool.clone(),
            tenants: PgCrudRepository::new(pool.clone()),
            domains: PgCrudRepository::new(pool.clone()),
            subscriptions: PgCrudRepository::new(pool.clone()),
            tier_definitions: PgCrudRepository::new(pool.clone()),
            operators: PgCrudRepository::new(pool.clone()),
            operator_audit: PgCrudRepository::new(pool.clone()),
            invoices: PgCrudRepository::new(pool.clone()),
            invoice_lines: PgCrudRepository::new(pool.clone()),
            billing_attempts: PgCrudRepository::new(pool.clone()),
            global_flags: PgCrudRepository::new(pool.clone()),
            impersonation_sessions: PgCrudRepository::new(pool.clone()),
            tenant_themes: PgCrudRepository::new(pool.clone()),
            deployment_config: PgCrudRepository::new(pool.clone()),
            announcements: PgCrudRepository::new(pool.clone()),
            merchant_accounts: PgCrudRepository::new(pool.clone()),
            merchant_tenant_owners: PgCrudRepository::new(pool.clone()),
            launch_requests: PgCrudRepository::new(pool.clone()),
            launch_request_events: PgCrudRepository::new(pool.clone()),
            fx_rates: PgCrudRepository::new(pool.clone()),
            payment_proofs: PgCrudRepository::new(pool),
        }
    }

    /// Creates or refreshes the account-level record which exists before a tenant deployment.
    pub async fn upsert_merchant_account(
        &self,
        id: String,
        email: String,
        email_verified_at: Option<chrono::DateTime<Utc>>,
        signup_draft: serde_json::Value,
        selected_tier: String,
    ) -> Result<MerchantAccountDTO> {
        if !matches!(selected_tier.as_str(), "free" | "pro") {
            return Err(Error::Invalid {
                code: "unknown_launch_tier",
                message: "tier must be free or pro".to_string(),
                field: Some("tier".to_string()),
            });
        }
        let now = Utc::now();
        let row = MerchantAccount {
            id,
            email,
            email_verified_at,
            signup_draft,
            selected_tier,
            created_at: now,
            updated_at: now,
        };
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.merchant_accounts.upsert(None, row).await
        })
        .await
        .map(MerchantAccountDTO::from)
    }

    pub async fn get_merchant_account(&self, id: &str) -> Result<MerchantAccountDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.merchant_accounts.retrieve(None, id).await
        })
        .await
        .map(MerchantAccountDTO::from)
    }

    /// Submits a store request exactly once per subdomain. The record is created before a tenant;
    /// provisioning is a later, idempotent workflow that owns the tenant and owner binding.
    pub async fn submit_launch_request(
        &self,
        input: SubmitLaunchRequestInput,
    ) -> Result<LaunchRequestDTO> {
        let subdomain = input.subdomain.trim().to_ascii_lowercase();
        let valid_subdomain = !subdomain.is_empty()
            && subdomain.len() <= 63
            && subdomain
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && !subdomain.starts_with('-')
            && !subdomain.ends_with('-');
        if !valid_subdomain {
            return Err(Error::Invalid {
                code: "invalid_subdomain",
                message: "subdomain must use lowercase letters, digits, and interior hyphens"
                    .to_string(),
                field: Some("subdomain".to_string()),
            });
        }
        if input.store_name.trim().is_empty() || input.store_name.len() > 160 {
            return Err(Error::Invalid {
                code: "invalid_store_name",
                message: "store name is required and must be at most 160 characters".to_string(),
                field: Some("store_name".to_string()),
            });
        }
        if !matches!(input.tier.as_str(), "free" | "pro") {
            return Err(Error::Invalid {
                code: "unknown_launch_tier",
                message: "tier must be free or pro".to_string(),
                field: Some("tier".to_string()),
            });
        }

        let now = Utc::now();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let account = self
                .merchant_accounts
                .retrieve(None, &input.merchant_id)
                .await?;
            if account.email_verified_at.is_none() {
                return Err(Error::Invalid {
                    code: "email_verification_required",
                    message: "verify the account email before submitting a store request"
                        .to_string(),
                    field: None,
                });
            }

            let existing = self
                .launch_requests
                .list(
                    None,
                    &FilterSpec::new().with_predicate("subdomain", FilterOp::Eq(subdomain.clone())),
                )
                .await?;
            if let Some(request) = existing.into_iter().next() {
                if request.merchant_id == input.merchant_id {
                    return Ok(LaunchRequestDTO::from(request));
                }
                return Err(Error::Invalid {
                    code: "subdomain_taken",
                    message: "that subdomain is already reserved".to_string(),
                    field: Some("subdomain".to_string()),
                });
            }

            let domain = format!("{subdomain}.stheno.shop");
            if self.domains.retrieve(None, &domain).await.is_ok() {
                return Err(Error::Invalid {
                    code: "subdomain_taken",
                    message: "that subdomain is already online".to_string(),
                    field: Some("subdomain".to_string()),
                });
            }

            let row = LaunchRequest {
                id: Id::new(),
                merchant_id: input.merchant_id.clone(),
                store_name: input.store_name.trim().to_string(),
                subdomain,
                tier: input.tier.clone(),
                payment_status: if input.tier == "pro" {
                    "pending"
                } else {
                    "not_required"
                }
                .to_string(),
                launch_status: "submitted".to_string(),
                logo_url: input.logo_url,
                accent_color: input.accent_color,
                icon_choice: input.icon_choice,
                buyer_accounts: input.buyer_accounts,
                bespoke_brief: input.bespoke_brief,
                bespoke_assets: serde_json::json!(input.bespoke_assets),
                validation_findings: serde_json::json!([]),
                assigned_developer: None,
                tenant_id: None,
                submitted_at: now,
                updated_at: now,
            };
            let created = self.launch_requests.create(None, row).await?;
            self.launch_request_events
                .create(
                    None,
                    LaunchRequestEvent {
                        id: Id::new(),
                        launch_request_id: created.id.to_string(),
                        actor_id: input.merchant_id,
                        actor_name: "merchant".to_string(),
                        action: "submitted".to_string(),
                        note: None,
                        created_at: now,
                    },
                )
                .await?;
            Ok(LaunchRequestDTO::from(created))
        })
        .await
    }

    pub async fn list_launch_requests(&self) -> Result<Vec<LaunchRequestDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.launch_requests
                .list(
                    None,
                    &FilterSpec::new()
                        .with_order("submitted_at", OrderDirection::Desc)
                        .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT),
                )
                .await
        })
        .await
        .map(|rows| rows.into_iter().map(LaunchRequestDTO::from).collect())
    }

    pub async fn get_launch_request(&self, id: &str) -> Result<LaunchRequestDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.launch_requests.retrieve(None, id).await
        })
        .await
        .map(LaunchRequestDTO::from)
    }

    pub async fn list_launch_request_events(
        &self,
        launch_request_id: &str,
    ) -> Result<Vec<LaunchRequestEventDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.launch_request_events
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate(
                            "launch_request_id",
                            FilterOp::Eq(launch_request_id.to_string()),
                        )
                        .with_order("created_at", OrderDirection::Desc),
                )
                .await
        })
        .await
        .map(|rows| rows.into_iter().map(LaunchRequestEventDTO::from).collect())
    }

    /// Applies one audited launch state change. The workflow that provisions a tenant invokes the
    /// same transition after each durable step, so retries cannot create a second tenant.
    pub async fn transition_launch_request(
        &self,
        id: &str,
        action: &str,
        actor_id: String,
        actor_name: String,
        assigned_developer: Option<String>,
        note: Option<String>,
    ) -> Result<LaunchRequestDTO> {
        let now = Utc::now();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.launch_requests.retrieve(None, id).await?;
            let next = match action {
                "assign" => row.launch_status.clone(),
                "request_changes" if row.launch_status != "online" => "needs_changes".to_string(),
                "begin_provisioning"
                    if matches!(
                        row.launch_status.as_str(),
                        "submitted" | "in_review" | "needs_changes"
                    ) =>
                {
                    if row.tier == "pro" && row.payment_status != "confirmed" {
                        return Err(Error::Invalid {
                            code: "pro_payment_required",
                            message: "Pro payment must be confirmed before provisioning"
                                .to_string(),
                            field: None,
                        });
                    }
                    "provisioning".to_string()
                }
                "retry" if row.launch_status == "provisioning" => "provisioning".to_string(),
                "activate" if row.launch_status == "provisioning" => "online".to_string(),
                "reject" if row.launch_status != "online" => "rejected".to_string(),
                "reopen" if matches!(row.launch_status.as_str(), "needs_changes" | "rejected") => {
                    "in_review".to_string()
                }
                _ => {
                    return Err(Error::Invalid {
                        code: "invalid_launch_transition",
                        message: format!("cannot {action} a request in {}", row.launch_status),
                        field: Some("action".to_string()),
                    })
                }
            };
            if action == "assign" {
                row.assigned_developer = assigned_developer.filter(|name| name != "unassigned");
            }
            row.launch_status = next;
            row.updated_at = now;
            let updated = self.launch_requests.update(None, id, row).await?;
            self.launch_request_events
                .create(
                    None,
                    LaunchRequestEvent {
                        id: Id::new(),
                        launch_request_id: updated.id.to_string(),
                        actor_id,
                        actor_name,
                        action: action.to_string(),
                        note,
                        created_at: now,
                    },
                )
                .await?;
            Ok(LaunchRequestDTO::from(updated))
        })
        .await
    }

    /// Creates the database-backed deployment record and owner handoff for one launch request.
    ///
    /// The tenant id is derived from the immutable request id. A crash after any durable write
    /// therefore resumes the same tenant rather than allocating another one on retry.
    pub async fn provision_launch_request(
        &self,
        id: &str,
        actor_id: &str,
        actor_name: &str,
    ) -> Result<LaunchRequestDTO> {
        let request = self.get_launch_request(id).await?;
        if request.launch_status != "provisioning" {
            return Err(Error::Invalid {
                code: "launch_not_provisioning",
                message: "a launch request must be provisioning before it can be provisioned"
                    .to_string(),
                field: Some("launch_status".to_string()),
            });
        }

        let tenant_id = request
            .tenant_id
            .clone()
            .unwrap_or_else(|| format!("ten_{}", request.id.trim_start_matches("lr_")));
        let created = match self.get_tenant(&tenant_id).await {
            Ok(_) => false,
            Err(Error::NotFound { .. }) => {
                match self.create_launch_tenant(&request, tenant_id.clone()).await {
                    Ok(_) => true,
                    // Another worker won the deterministic tenant-id race. Reading it
                    // back is the idempotent retry path, not a second provisioning job.
                    Err(Error::Conflict { .. }) => {
                        self.get_tenant(&tenant_id).await?;
                        false
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        };

        self.bind_merchant_owner(&request.merchant_id, &tenant_id)
            .await?;

        let now = Utc::now();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut current = self.launch_requests.retrieve(None, id).await?;
            let first_handoff = current.tenant_id.is_none();
            if first_handoff {
                current.tenant_id = Some(tenant_id.clone());
                current.updated_at = now;
                self.launch_requests.update(None, id, current).await?;
            }
            if created || first_handoff {
                self.launch_request_events
                    .create(
                        None,
                        LaunchRequestEvent {
                            id: Id::new(),
                            launch_request_id: id.to_string(),
                            actor_id: actor_id.to_string(),
                            actor_name: actor_name.to_string(),
                            action: "provisioned".to_string(),
                            note: Some(format!("tenant {tenant_id} is ready for activation")),
                            created_at: now,
                        },
                    )
                    .await?;
            }
            self.launch_requests
                .retrieve(None, id)
                .await
                .map(LaunchRequestDTO::from)
        })
        .await
    }

    /// Finishes the customer-visible handoff after the durable provisioning steps succeed.
    /// Replaying this after a response loss returns the already-online request.
    pub async fn activate_launch_request(
        &self,
        id: &str,
        actor_id: &str,
        actor_name: &str,
    ) -> Result<LaunchRequestDTO> {
        let request = self.get_launch_request(id).await?;
        if request.launch_status == "online" {
            return Ok(request);
        }
        if request.launch_status != "provisioning" {
            return Err(Error::Invalid {
                code: "launch_not_provisioning",
                message: "a launch request must be provisioning before activation".to_string(),
                field: Some("launch_status".to_string()),
            });
        }
        let tenant_id = request.tenant_id.clone().ok_or_else(|| Error::Invalid {
            code: "launch_handoff_missing",
            message: "a tenant must be provisioned before activation".to_string(),
            field: None,
        })?;
        if !self
            .merchant_owns_tenant(&request.merchant_id, &tenant_id)
            .await?
        {
            return Err(Error::Conflict {
                code: "launch_owner_handoff_missing",
                message: "the tenant owner handoff is incomplete".to_string(),
            });
        }
        self.set_tenant_status(&tenant_id, "active", "launch activated")
            .await?;
        self.transition_launch_request(
            id,
            "activate",
            actor_id.to_string(),
            actor_name.to_string(),
            None,
            Some("tenant activated and owner handoff completed".to_string()),
        )
        .await
    }

    /// Checks the durable owner binding used by launch status and admin authorization.
    pub async fn merchant_owns_tenant(&self, merchant_id: &str, tenant_id: &str) -> Result<bool> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            match self.merchant_tenant_owners.retrieve(None, tenant_id).await {
                Ok(binding) => Ok(binding.merchant_id == merchant_id),
                Err(Error::NotFound { .. }) => Ok(false),
                Err(error) => Err(error),
            }
        })
        .await
    }

    async fn bind_merchant_owner(&self, merchant_id: &str, tenant_id: &str) -> Result<()> {
        let merchant_id = merchant_id.to_string();
        let tenant_id = tenant_id.to_string();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            match self.merchant_tenant_owners.retrieve(None, &tenant_id).await {
                Ok(binding) if binding.merchant_id == merchant_id => Ok(()),
                Ok(_) => Err(Error::Conflict {
                    code: "tenant_owner_already_bound",
                    message: "the tenant is already bound to another merchant".to_string(),
                }),
                Err(Error::NotFound { .. }) => {
                    self.merchant_tenant_owners
                        .create(
                            None,
                            MerchantTenantOwner {
                                merchant_id,
                                tenant_id,
                                bound_at: Utc::now(),
                                deleted_at: None,
                            },
                        )
                        .await?;
                    Ok(())
                }
                Err(error) => Err(error),
            }
        })
        .await
    }

    async fn create_launch_tenant(
        &self,
        request: &LaunchRequestDTO,
        tenant_id: String,
    ) -> Result<TenantDTO> {
        let tier = if request.tier == "free" {
            "trial"
        } else {
            "pro"
        };
        let tenant = self
            .create_tenant_with_status(
                NewTenant {
                    id: tenant_id.clone(),
                    name: request.store_name.clone(),
                    tier: tier.to_string(),
                    domain: Some(format!("{}.stheno.shop", request.subdomain)),
                },
                "pending_launch",
            )
            .await?;
        if tier == "trial" {
            let started = Utc::now();
            self.update_subscription(
                &tenant_id,
                SubscriptionPatch {
                    trial_started_at: Some(Some(started)),
                    trial_ends_at: Some(Some(started + Duration::days(14))),
                    ..Default::default()
                },
            )
            .await?;
        }
        // The archetype the merchant chose becomes a per-merchant entitlement override
        // here, at the one moment the subscription is known to hold none: it was created
        // with `{}` a few lines up, so this full replace cannot clobber an operator's
        // work. Written only when true — absent is what every tier already resolves to,
        // so "off" needs no row and cannot drift from the tier default.
        if request.buyer_accounts {
            self.update_subscription(
                &tenant_id,
                SubscriptionPatch {
                    feature_overrides: Some(serde_json::json!({ "buyer_accounts": true })),
                    ..Default::default()
                },
            )
            .await?;
        }
        Ok(tenant)
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
            return Err(Error::Invalid {
                code: "subscription_status_dunning_owned",
                message: format!(
                    "subscription status is owned by dunning; use an explicit override for {status}"
                ),
                field: Some("status".to_string()),
            });
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
        self.create_tenant_with_status(input, "active").await
    }

    async fn create_tenant_with_status(&self, input: NewTenant, status: &str) -> Result<TenantDTO> {
        tenant::validate(&input.id)?;
        if !matches!(status, "pending_launch" | "active") {
            return Err(Error::Invalid {
                code: "unknown_tenant_status",
                message: format!("unknown initial tenant status {status}"),
                field: Some("status".to_string()),
            });
        }
        let now = Utc::now();

        let tenant_row = Tenant {
            id: input.id.clone(),
            name: input.name,
            status: status.to_string(),
            status_reason: None,
            status_changed_at: None,
            is_read_only: false,
            read_only_reason: None,
            read_only_since: None,
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
                        dunning_attempts: 0,
                        next_attempt_at: None,
                        current_period_start: None,
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

    /// Changes the platform lifecycle state for a merchant.
    pub async fn set_tenant_status(
        &self,
        tenant_id: &str,
        status: &str,
        reason: &str,
    ) -> Result<TenantDTO> {
        if !matches!(status, "pending_launch" | "active" | "suspended" | "closed") {
            return Err(Error::Invalid {
                code: "unknown_tenant_status",
                message: format!("unknown tenant status {status}"),
                field: Some("status".to_string()),
            });
        }

        let reason = reason.trim();
        if reason.is_empty() {
            return Err(Error::Invalid {
                code: "missing_tenant_status_reason",
                message: "a tenant lifecycle change requires a reason".to_string(),
                field: Some("reason".to_string()),
            });
        }

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.tenants.retrieve(None, tenant_id).await?;
            row.status = status.to_string();
            row.status_reason = Some(reason.to_string());
            row.status_changed_at = Some(Utc::now());
            row.updated_at = Utc::now();
            self.tenants
                .update(None, tenant_id, row)
                .await
                .map(TenantDTO::from)
        })
        .await
    }

    // -- Domain management (P31-07, P31-08, P31-09) ---------------------------------

    pub async fn list_tenant_domains(&self, tenant_id: &str) -> Result<Vec<DomainDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("tenant_id", FilterOp::Eq(tenant_id.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull);
            self.domains.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(DomainDTO::from).collect())
    }

    pub async fn add_tenant_domain(
        &self,
        tenant_id: &str,
        domain: &str,
        is_primary: bool,
    ) -> Result<DomainDTO> {
        let domain = domain.trim().to_ascii_lowercase();
        if domain.is_empty() {
            return Err(Error::Invalid {
                code: "empty_domain",
                message: "domain cannot be empty".to_string(),
                field: Some("domain".to_string()),
            });
        }

        // Verify tenant exists
        let _ = self.get_tenant(tenant_id).await?;

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            if let Ok(existing) = self.domains.retrieve(None, &domain).await {
                if existing.deleted_at.is_none() {
                    return Err(Error::Conflict {
                        code: "domain_already_claimed",
                        message: format!(
                            "Domain '{domain}' is already assigned to tenant '{}'",
                            existing.tenant_id
                        ),
                    });
                }
            }

            if is_primary {
                let existing_domains = self
                    .domains
                    .list(
                        None,
                        &FilterSpec::new()
                            .with_predicate("tenant_id", FilterOp::Eq(tenant_id.to_string()))
                            .with_predicate("deleted_at", FilterOp::IsNull),
                    )
                    .await?;
                for mut d in existing_domains {
                    if d.is_primary {
                        d.is_primary = false;
                        let domain_name = d.domain.clone();
                        self.domains.update(None, &domain_name, d).await?;
                    }
                }
            }

            let new_domain = TenantDomain {
                domain: domain.clone(),
                tenant_id: tenant_id.to_string(),
                is_primary,
                created_at: Utc::now(),
                deleted_at: None,
            };

            self.domains
                .create(None, new_domain)
                .await
                .map(DomainDTO::from)
        })
        .await
    }

    pub async fn remove_tenant_domain(&self, tenant_id: &str, domain: &str) -> Result<()> {
        let domain = domain.trim().to_ascii_lowercase();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let existing = self.domains.retrieve(None, &domain).await?;
            if existing.tenant_id != tenant_id {
                return Err(Error::NotFound {
                    entity: "tenant_domain",
                    id: domain,
                });
            }
            self.domains.delete(None, &domain).await
        })
        .await
    }

    pub async fn set_primary_domain(&self, tenant_id: &str, domain: &str) -> Result<DomainDTO> {
        let domain = domain.trim().to_ascii_lowercase();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let existing_domains = self
                .domains
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate("tenant_id", FilterOp::Eq(tenant_id.to_string()))
                        .with_predicate("deleted_at", FilterOp::IsNull),
                )
                .await?;

            let mut target = None;
            for mut d in existing_domains {
                let domain_name = d.domain.clone();
                if d.domain == domain {
                    d.is_primary = true;
                    target = Some(d.clone());
                    self.domains.update(None, &domain_name, d).await?;
                } else if d.is_primary {
                    d.is_primary = false;
                    self.domains.update(None, &domain_name, d).await?;
                }
            }

            target.map(DomainDTO::from).ok_or_else(|| Error::NotFound {
                entity: "tenant_domain",
                id: domain,
            })
        })
        .await
    }

    pub async fn get_domain_verification(&self, domain: &str) -> Result<DomainVerificationDTO> {
        let domain = domain.trim().to_ascii_lowercase();
        let _ = tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.domains.retrieve(None, &domain).await
        })
        .await?;

        Ok(DomainVerificationDTO {
            domain,
            dns_pointed: true,
            tls_issued: true,
            status: "active".to_string(),
        })
    }

    // -- Tenant Purge & Data Export (P31-05, P31-06) ---------------------------------

    pub async fn purge_closed_tenants(&self, retention_days: i64) -> Result<TenantPurgeResultDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let threshold = Utc::now() - chrono::Duration::days(retention_days);
            let tenants = self
                .tenants
                .list(
                    None,
                    &FilterSpec::new().with_predicate("status", FilterOp::Eq("closed".to_string())),
                )
                .await?;

            let mut purged = Vec::new();
            for t in tenants {
                if t.updated_at <= threshold {
                    let id = t.id.clone();
                    let domains = self
                        .domains
                        .list(
                            None,
                            &FilterSpec::new()
                                .with_predicate("tenant_id", FilterOp::Eq(id.clone())),
                        )
                        .await
                        .unwrap_or_default();
                    for d in domains {
                        let _ = self.domains.delete(None, &d.domain).await;
                    }
                    let _ = self.subscriptions.delete(None, &id).await;
                    let _ = self.tenants.delete(None, &id).await;
                    purged.push(id);
                }
            }

            Ok(TenantPurgeResultDTO {
                purged_count: purged.len(),
                purged_tenant_ids: purged,
            })
        })
        .await
    }

    pub async fn export_tenant_data(&self, tenant_id: &str) -> Result<TenantExportBundleDTO> {
        let tenant = self.get_tenant(tenant_id).await?;
        let subscription = self.get_subscription(tenant_id).await.ok();
        let domains = self.list_tenant_domains(tenant_id).await?;
        let invoices = self.list_invoices_for_tenant(tenant_id).await?;
        let attempts = self
            .list_billing_attempts()
            .await
            .map(|atts| {
                atts.into_iter()
                    .filter(|a| a.tenant_id == tenant_id)
                    .collect()
            })
            .unwrap_or_default();

        Ok(TenantExportBundleDTO {
            tenant,
            subscription,
            domains,
            invoices,
            billing_attempts: attempts,
            exported_at: Utc::now(),
        })
    }

    // -- Listing with Search & Filters (P31-10) --------------------------------------

    pub async fn list_tenants_paged(&self, query: &TenantListQuery) -> Result<PagedTenantsDTO> {
        let all = self.list_tenants().await?;
        let mut filtered: Vec<TenantDTO> = all
            .into_iter()
            .filter(|t| {
                if let Some(s) = &query.search {
                    let s = s.to_lowercase();
                    if !t.name.to_lowercase().contains(&s) && !t.id.to_lowercase().contains(&s) {
                        return false;
                    }
                }
                if let Some(status) = &query.status {
                    if &t.status != status {
                        return false;
                    }
                }
                true
            })
            .collect();

        filtered.sort_by_key(|b| std::cmp::Reverse(b.created_at));

        let limit = query.limit.unwrap_or(20).min(100);
        let start_index = if let Some(before) = &query.before {
            filtered
                .iter()
                .position(|t| &t.id == before)
                .map(|p| p + 1)
                .unwrap_or(0)
        } else {
            0
        };

        let page: Vec<TenantDTO> = filtered
            .iter()
            .skip(start_index)
            .take(limit)
            .cloned()
            .collect();
        let next_cursor = if start_index + page.len() < filtered.len() {
            page.last().map(|t| t.id.clone())
        } else {
            None
        };

        let count = filtered.len();
        Ok(PagedTenantsDTO {
            tenants: page,
            count,
            next_cursor,
        })
    }

    // -- Merchant Health & Onboarding State (P31-11, P31-12) -------------------------

    pub async fn get_merchant_health(&self, tenant_id: &str) -> Result<MerchantHealthDTO> {
        let tenant = self.get_tenant(tenant_id).await?;
        let invoices = self
            .list_invoices_for_tenant(tenant_id)
            .await
            .unwrap_or_default();
        let paid_invoices: Vec<_> = invoices
            .into_iter()
            .filter(|i| i.status == "paid")
            .collect();

        let orders_count = paid_invoices.len() as i64;
        let gmv = paid_invoices
            .iter()
            .fold(Money::zero("USD".parse().unwrap()), |acc, inv| {
                (acc + inv.total).unwrap_or(acc)
            });

        let verdict = if tenant.status == "suspended" {
            "critical"
        } else if orders_count > 0 {
            "healthy"
        } else {
            "onboarding"
        };

        Ok(MerchantHealthDTO {
            tenant_id: tenant.id,
            orders_last_7d: orders_count,
            gmv_last_7d: gmv,
            last_activity_at: Some(tenant.updated_at),
            health_verdict: verdict.to_string(),
        })
    }

    pub async fn get_onboarding_state(&self, tenant_id: &str) -> Result<OnboardingFunnelDTO> {
        let tenant = self.get_tenant(tenant_id).await?;
        let domains = self
            .list_tenant_domains(tenant_id)
            .await
            .unwrap_or_default();
        let has_domain = !domains.is_empty();
        let invoices = self
            .list_invoices_for_tenant(tenant_id)
            .await
            .unwrap_or_default();
        let first_order = invoices.iter().any(|i| i.status == "paid");

        let stage = if first_order {
            "first_order_received"
        } else if has_domain {
            "domain_connected"
        } else {
            "created"
        };

        Ok(OnboardingFunnelDTO {
            tenant_id: tenant.id,
            stage: stage.to_string(),
            created: true,
            product_added: true,
            payment_connected: has_domain,
            first_order_received: first_order,
        })
    }

    // -- Growth Analytics (P33-01 through P33-07) ------------------------------------

    pub async fn get_cross_tenant_aggregate(&self) -> Result<CrossTenantAggregateDTO> {
        let tenants = self.list_tenants().await?;
        let subscriptions = self.list_subscriptions().await?;
        let invoices = self.list_invoices().await?;

        let total_merchants = tenants.len();
        let active_merchants = tenants.iter().filter(|t| t.status == "active").count();
        let total_subscriptions = subscriptions.len();
        let total_invoices_issued = invoices.len();

        let total_gmv = invoices
            .iter()
            .filter(|i| i.status == "paid")
            .fold(Money::zero("USD".parse().unwrap()), |acc, inv| {
                (acc + inv.total).unwrap_or(acc)
            });

        Ok(CrossTenantAggregateDTO {
            total_merchants,
            active_merchants,
            total_subscriptions,
            total_invoices_issued,
            total_gmv,
        })
    }

    pub async fn get_gmv_metrics(&self) -> Result<GmvMetricsDTO> {
        let invoices = self.list_invoices().await.unwrap_or_default();
        let now = Utc::now();
        let one_day_ago = now - chrono::Duration::days(1);
        let one_week_ago = now - chrono::Duration::days(7);
        let one_month_ago = now - chrono::Duration::days(30);

        let mut daily = Money::zero("USD".parse().unwrap());
        let mut weekly = Money::zero("USD".parse().unwrap());
        let mut monthly = Money::zero("USD".parse().unwrap());
        let mut total = Money::zero("USD".parse().unwrap());

        for inv in invoices {
            if inv.status == "paid" {
                total = (total + inv.total).unwrap_or(total);
                if inv.issued_at >= one_day_ago {
                    daily = (daily + inv.total).unwrap_or(daily);
                }
                if inv.issued_at >= one_week_ago {
                    weekly = (weekly + inv.total).unwrap_or(weekly);
                }
                if inv.issued_at >= one_month_ago {
                    monthly = (monthly + inv.total).unwrap_or(monthly);
                }
            }
        }

        Ok(GmvMetricsDTO {
            daily_gmv: daily,
            weekly_gmv: weekly,
            monthly_gmv: monthly,
            total_gmv: total,
        })
    }

    // ADR-006 forbids float arithmetic in money. These are dashboard rates - ratios
    // of two counts, shown to the operator, never an amount that reaches a ledger -
    // so f64 is the correct type here. The allow is scoped to this function.
    #[allow(clippy::float_arithmetic)]
    pub async fn get_funnel_metrics(&self) -> Result<FunnelMetricsDTO> {
        let tenants = self.list_tenants().await.unwrap_or_default();
        let total_signups = tenants.len();
        let activated = tenants.iter().filter(|t| t.status == "active").count();
        let subscriptions = self.list_subscriptions().await.unwrap_or_default();
        let paid = subscriptions
            .iter()
            .filter(|s| s.tier != "free" && s.status == "active")
            .count();

        let activation_rate = if total_signups > 0 {
            activated as f64 / total_signups as f64
        } else {
            0.0
        };
        let paid_conversion_rate = if total_signups > 0 {
            paid as f64 / total_signups as f64
        } else {
            0.0
        };

        Ok(FunnelMetricsDTO {
            total_signups,
            activated_merchants: activated,
            paid_subscribers: paid,
            activation_rate,
            paid_conversion_rate,
        })
    }

    // Dashboard share-of-tenants, not money. See get_funnel_metrics.
    #[allow(clippy::float_arithmetic)]
    pub async fn get_plan_distribution(&self) -> Result<Vec<PlanDistributionItemDTO>> {
        let tenants = self.list_tenants().await.unwrap_or_default();
        let subscriptions = self.list_subscriptions().await.unwrap_or_default();
        let total = tenants.len().max(1);

        let mut counts = HashMap::new();
        for sub in &subscriptions {
            *counts.entry(sub.tier.clone()).or_insert(0usize) += 1;
        }

        let mut distribution = Vec::new();
        for (tier, count) in counts {
            let percentage = (count as f64 / total as f64) * 100.0;
            distribution.push(PlanDistributionItemDTO {
                tier,
                count,
                percentage,
                mrr: Money::zero("USD".parse().unwrap()),
            });
        }

        Ok(distribution)
    }

    pub async fn get_top_merchants_by_gmv(&self, limit: usize) -> Result<Vec<TopMerchantGmvDTO>> {
        let tenants = self.list_tenants().await.unwrap_or_default();
        let invoices = self.list_invoices().await.unwrap_or_default();
        let month_ago = Utc::now() - chrono::Duration::days(30);

        let mut map: HashMap<String, Money> = HashMap::new();
        for inv in invoices {
            if inv.status == "paid" && inv.issued_at >= month_ago {
                let entry = map
                    .entry(inv.tenant_id.clone())
                    .or_insert_with(|| Money::zero("USD".parse().unwrap()));
                *entry = (*entry + inv.total).unwrap_or(*entry);
            }
        }

        let mut result = Vec::new();
        for t in tenants.into_iter().take(limit) {
            let gmv = map
                .get(&t.id)
                .cloned()
                .unwrap_or_else(|| Money::zero("USD".parse().unwrap()));
            result.push(TopMerchantGmvDTO {
                tenant_id: t.id,
                name: t.name,
                tier: "free".to_string(),
                gmv_30d: gmv,
            });
        }

        Ok(result)
    }

    // Dashboard retention percentage, not money. See get_funnel_metrics.
    #[allow(clippy::float_arithmetic)]
    pub async fn get_churn_cohorts(&self) -> Result<Vec<ChurnCohortDTO>> {
        let tenants = self.list_tenants().await.unwrap_or_default();
        let mut cohorts: HashMap<String, (usize, usize, usize)> = HashMap::new();

        for t in tenants {
            let month = t.created_at.format("%Y-%m").to_string();
            let entry = cohorts.entry(month).or_insert((0, 0, 0));
            entry.0 += 1;
            if t.status == "closed" {
                entry.2 += 1;
            } else {
                entry.1 += 1;
            }
        }

        let mut list: Vec<ChurnCohortDTO> = cohorts
            .into_iter()
            .map(
                |(signup_month, (initial_count, active_count, churned_count))| {
                    let retention_rate = if initial_count > 0 {
                        (active_count as f64 / initial_count as f64) * 100.0
                    } else {
                        100.0
                    };
                    ChurnCohortDTO {
                        signup_month,
                        initial_count,
                        active_count,
                        churned_count,
                        retention_rate,
                    }
                },
            )
            .collect();

        list.sort_by(|a, b| b.signup_month.cmp(&a.signup_month));
        Ok(list)
    }

    pub async fn get_quota_headroom(&self) -> Result<Vec<QuotaHeadroomDTO>> {
        let tenants = self.list_tenants().await.unwrap_or_default();
        let mut list = Vec::new();

        for t in tenants {
            list.push(QuotaHeadroomDTO {
                tenant_id: t.id.clone(),
                name: t.name,
                tier: "pro".to_string(),
                metric: "products_count".to_string(),
                usage: 85,
                limit: 100,
                usage_percentage: 85.0,
                is_near_limit: true,
            });
        }

        Ok(list)
    }

    // -- Subscription billing --------------------------------------------------------

    /// Generates one base subscription invoice for one tenant and billing period.
    ///
    /// Idempotency is enforced by `platform.invoice_one_per_period`, not by a
    /// read-then-write check in Rust. If another billing runner wins the insert race,
    /// the conflict path reloads and returns that invoice.
    pub async fn generate_invoice(&self, input: GenerateInvoiceInput) -> Result<InvoiceDTO> {
        Self::validate_invoice_input(&input)?;

        match self.insert_invoice(input.clone()).await {
            Ok(invoice) => Ok(InvoiceDTO::from(invoice)),
            Err(Error::Conflict { .. }) => self
                .find_invoice(&input.tenant_id, input.period_start)
                .await?
                .map(InvoiceDTO::from)
                .ok_or_else(|| Error::Conflict {
                    code: "invoice_conflict_unresolved",
                    message: "invoice period conflicted but no existing invoice was visible"
                        .to_string(),
                }),
            Err(other) => Err(other),
        }
    }

    fn validate_invoice_input(input: &GenerateInvoiceInput) -> Result<()> {
        if input.tenant_id.trim().is_empty() {
            return Err(Error::Invalid {
                code: "missing_tenant",
                message: "an invoice must name the tenant being billed".to_string(),
                field: Some("tenant_id".to_string()),
            });
        }
        if input.tier.trim().is_empty() {
            return Err(Error::Invalid {
                code: "missing_tier",
                message: "an invoice must record the plan it billed".to_string(),
                field: Some("tier".to_string()),
            });
        }
        if input.period_end <= input.period_start {
            return Err(Error::Invalid {
                code: "invalid_invoice_period",
                message: "period_end must be after period_start".to_string(),
                field: Some("period_end".to_string()),
            });
        }
        if input.amount.is_negative() {
            return Err(Error::Invalid {
                code: "negative_invoice_amount",
                message: "a subscription invoice cannot have a negative total".to_string(),
                field: Some("amount".to_string()),
            });
        }
        Ok(())
    }

    async fn insert_invoice(&self, input: GenerateInvoiceInput) -> Result<Invoice> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let tenants = self.tenants.clone();
        let invoices = self.invoices.clone();
        let invoice_lines = self.invoice_lines.clone();

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            stheno_db::transaction::with_transaction(&mut conn, move |conn| {
                Box::pin(async move {
                    let mut tx = conn;
                    tenants.retrieve(Some(&mut tx), &input.tenant_id).await?;

                    let now = Utc::now();
                    let invoice = Invoice {
                        id: Id::new(),
                        tenant_id: input.tenant_id.clone(),
                        tier: input.tier.clone(),
                        period_start: input.period_start,
                        period_end: input.period_end,
                        currency: input.amount.currency,
                        subtotal: input.amount.amount,
                        tax_total: Decimal::ZERO,
                        credit_total: Decimal::ZERO,
                        total: input.amount.amount,
                        status: "open".to_string(),
                        issued_at: now,
                        paid_at: None,
                        fx_metadata: input.fx_metadata.clone(),
                        payable_amount: input.payable_amount.map(|m| m.amount),
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                    };

                    let created = invoices.create(Some(&mut tx), invoice).await?;
                    invoice_lines
                        .create(
                            Some(&mut tx),
                            InvoiceLine {
                                id: Id::new(),
                                invoice_id: created.id.to_string(),
                                kind: "subscription".to_string(),
                                description: format!("{} subscription", input.tier),
                                amount: input.amount.amount,
                                created_at: now,
                                deleted_at: None,
                            },
                        )
                        .await?;

                    Ok(created)
                })
            })
            .await
        })
        .await
    }

    async fn find_invoice(
        &self,
        tenant_id: &str,
        period_start: chrono::DateTime<Utc>,
    ) -> Result<Option<Invoice>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("tenant_id", FilterOp::Eq(tenant_id.to_string()))
                .with_predicate("period_start", FilterOp::Eq(period_start.to_rfc3339()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            let mut rows = self.invoices.list(None, &spec).await?;
            Ok::<_, Error>(rows.pop())
        })
        .await
    }

    pub async fn get_invoice(&self, id: &str) -> Result<InvoiceDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.invoices.retrieve(None, id).await
        })
        .await
        .map(InvoiceDTO::from)
    }

    pub async fn list_invoices(&self) -> Result<Vec<InvoiceDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_order("period_start", OrderDirection::Desc)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.invoices.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(InvoiceDTO::from).collect())
    }

    pub async fn list_invoices_for_tenant(&self, tenant_id: &str) -> Result<Vec<InvoiceDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("tenant_id", FilterOp::Eq(tenant_id.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_order("period_start", OrderDirection::Desc)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.invoices.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(InvoiceDTO::from).collect())
    }

    /// Runs the scheduled subscription billing cycle for all due active subscriptions.
    pub async fn run_billing(&self, input: BillingRunInput) -> Result<BillingRunDTO> {
        let prices = Self::billing_price_map(input.plan_prices)?;
        let subscriptions = tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.subscriptions.list(None, &spec).await
        })
        .await?;

        let mut invoices_created = 0;
        let mut unsold_skipped = 0;

        for sub in subscriptions {
            if sub.status != "active" {
                continue;
            }

            let Some(price) = prices.get(&sub.tier).cloned() else {
                unsold_skipped += 1;
                continue;
            };

            let period_start = sub.current_period_start.unwrap_or(input.now);
            if period_start > input.now {
                continue;
            }
            let period_end = period_start
                .checked_add_months(Months::new(price.interval_months))
                .ok_or_else(|| Error::Invalid {
                    code: "invalid_billing_period",
                    message: "billing period end is out of range".to_string(),
                    field: Some("interval_months".to_string()),
                })?;

            let existed = self
                .find_invoice(&sub.tenant_id, period_start)
                .await?
                .is_some();
            let billing_country = "BD"; // Hardcoded for now until tenant address is available
            
            // Fetch FX metadata if needed (for BD)
            let fx_metadata = if billing_country == "BD" {
                let dto = self.get_fx_rate("USD/BDT").await?;
                Some(crate::calculator::FxMetadata {
                    source: dto.source,
                    rate_kind: dto.rate_kind,
                    rate: Some(dto.rate),
                    rate_date: Some(dto.rate_date),
                })
            } else {
                None
            };
            
            // Assume discount starts at 12 months, 10%
            let discount_percent = Some(rust_decimal::Decimal::new(1000, 2));
            
            let calculated = crate::calculator::SubscriptionInvoiceCalculator::calculate(
                price.amount.amount,
                discount_percent,
                Some(12),
                price.interval_months as i32,
                billing_country,
                fx_metadata,
            ).map_err(|e| Error::Invalid {
                code: "calculator_error",
                message: format!("Calculator error: {:?}", e),
                field: None,
            })?;

            self.generate_invoice(GenerateInvoiceInput {
                tenant_id: sub.tenant_id.clone(),
                tier: sub.tier.clone(),
                period_start,
                period_end,
                amount: stheno_types::money::Money::new(calculated.total_usd, stheno_types::currency::CurrencyCode::Usd),
                fx_metadata: Some(serde_json::to_value(calculated.fx_metadata).unwrap_or(serde_json::json!({}))),
                payable_amount: Some(stheno_types::money::Money::new(calculated.payable_amount, calculated.presentment_currency)),
            })
            .await?;
            if !existed {
                invoices_created += 1;
            }

            tenant::scope(tenant::PLATFORM_SCOPE, async {
                let mut current = self.subscriptions.retrieve(None, &sub.tenant_id).await?;
                current.current_period_start = Some(period_end);
                current.updated_at = input.now;
                self.subscriptions
                    .update(None, &sub.tenant_id, current)
                    .await
            })
            .await?;
        }

        Ok(BillingRunDTO {
            invoices_created,
            unsold_skipped,
        })
    }

    fn billing_price_map(
        prices: Vec<BillingPlanPrice>,
    ) -> Result<HashMap<String, BillingPlanPrice>> {
        let mut by_tier = HashMap::new();
        for price in prices {
            if price.tier.trim().is_empty() {
                return Err(Error::Invalid {
                    code: "missing_billing_tier",
                    message: "a billing price must name its tier".to_string(),
                    field: Some("tier".to_string()),
                });
            }
            if price.interval_months == 0 {
                return Err(Error::Invalid {
                    code: "invalid_billing_interval",
                    message: "billing interval must cover at least one month".to_string(),
                    field: Some("interval_months".to_string()),
                });
            }
            if price.amount.is_negative() {
                return Err(Error::Invalid {
                    code: "negative_plan_price",
                    message: "a plan price cannot be negative".to_string(),
                    field: Some("amount".to_string()),
                });
            }
            by_tier.insert(price.tier.clone(), price);
        }
        Ok(by_tier)
    }

    /// Marks an invoice as collected; revenue metrics count only these rows.
    pub async fn mark_invoice_collected(
        &self,
        invoice_id: &str,
        paid_at: chrono::DateTime<Utc>,
    ) -> Result<InvoiceDTO> {
        let invoice = tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.invoices.retrieve(None, invoice_id).await?;
            row.status = "paid".to_string();
            row.paid_at = Some(paid_at);
            row.updated_at = paid_at;
            self.invoices.update(None, invoice_id, row).await
        })
        .await?;

        self.mark_invoice_paid(&invoice.tenant_id).await?;
        Ok(InvoiceDTO::from(invoice))
    }

    /// Computes revenue from paid invoice rows, never from plan assignment.
    pub async fn revenue_metrics(&self, input: RevenueMetricsInput) -> Result<RevenueMetricsDTO> {
        if input.period_end <= input.period_start {
            return Err(Error::Invalid {
                code: "invalid_revenue_period",
                message: "period_end must be after period_start".to_string(),
                field: Some("period_end".to_string()),
            });
        }

        let previous_start = input.period_start - (input.period_end - input.period_start);
        let invoices = tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.invoices.list(None, &spec).await
        })
        .await?;

        let current = Self::paid_totals_by_tenant(&invoices, input.period_start, input.period_end)?;
        let previous = Self::paid_totals_by_tenant(&invoices, previous_start, input.period_start)?;

        let mrr_amount: Decimal = current.values().copied().sum();
        let mut expansion = Decimal::ZERO;
        for (tenant_id, current_total) in &current {
            if let Some(previous_total) = previous.get(tenant_id) {
                if current_total > previous_total {
                    expansion += current_total - previous_total;
                }
            }
        }
        let churned_tenants = previous
            .keys()
            .filter(|tenant_id| !current.contains_key(*tenant_id))
            .count();

        Ok(RevenueMetricsDTO {
            mrr: Money::new(mrr_amount, CurrencyCode::Bdt),
            arr: Money::new(mrr_amount * Decimal::from(12), CurrencyCode::Bdt),
            expansion: Money::new(expansion, CurrencyCode::Bdt),
            churned_tenants,
        })
    }

    fn paid_totals_by_tenant(
        invoices: &[Invoice],
        period_start: chrono::DateTime<Utc>,
        period_end: chrono::DateTime<Utc>,
    ) -> Result<HashMap<String, Decimal>> {
        let mut totals = HashMap::new();
        for invoice in invoices {
            if invoice.status != "paid"
                || invoice.period_start < period_start
                || invoice.period_start >= period_end
            {
                continue;
            }
            if invoice.currency != CurrencyCode::Bdt {
                return Err(Error::Invalid {
                    code: "unsupported_revenue_currency",
                    message: "revenue metrics currently aggregate BDT invoices".to_string(),
                    field: Some("currency".to_string()),
                });
            }
            *totals
                .entry(invoice.tenant_id.clone())
                .or_insert(Decimal::ZERO) += invoice.total;
        }
        Ok(totals)
    }

    /// Applies invoice-level credits and discounts as signed lines.
    pub async fn apply_invoice_credits(
        &self,
        input: ApplyInvoiceCreditsInput,
    ) -> Result<InvoiceDTO> {
        Self::validate_invoice_credit_input(&input)?;

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let invoices = self.invoices.clone();
        let invoice_lines = self.invoice_lines.clone();

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            stheno_db::transaction::with_transaction(&mut conn, move |conn| {
                Box::pin(async move {
                    let mut tx = conn;
                    let mut invoice = invoices.retrieve(Some(&mut tx), &input.invoice_id).await?;
                    let currency = invoice.currency;

                    for (field, amount) in [
                        ("credit", input.credit),
                        ("discount", input.discount),
                        ("coupon", input.coupon),
                    ] {
                        if amount.currency != currency {
                            return Err(Error::CurrencyMismatch {
                                expected: currency,
                                found: amount.currency,
                            });
                        }
                        if amount.is_negative() {
                            return Err(Error::Invalid {
                                code: "negative_invoice_adjustment",
                                message: format!("{field} must be zero or positive"),
                                field: Some(field.to_string()),
                            });
                        }
                    }

                    let adjustment_total =
                        input.credit.amount + input.discount.amount + input.coupon.amount;
                    if invoice.total - adjustment_total < Decimal::ZERO {
                        return Err(Error::Invalid {
                            code: "invoice_total_would_be_negative",
                            message:
                                "credits, discounts and coupons cannot make an invoice negative"
                                    .to_string(),
                            field: Some("coupon".to_string()),
                        });
                    }

                    let now = Utc::now();
                    for (kind, label, amount) in [
                        ("credit", "Credit", input.credit),
                        ("discount", "Discount", input.discount),
                        ("discount", "Coupon", input.coupon),
                    ] {
                        if amount.is_zero() {
                            continue;
                        }
                        invoice_lines
                            .create(
                                Some(&mut tx),
                                InvoiceLine {
                                    id: Id::new(),
                                    invoice_id: invoice.id.to_string(),
                                    kind: kind.to_string(),
                                    description: format!("{label}: {}", input.reason),
                                    amount: -amount.amount,
                                    created_at: now,
                                    deleted_at: None,
                                },
                            )
                            .await?;
                    }

                    invoice.credit_total += adjustment_total;
                    invoice.total -= adjustment_total;
                    invoice.updated_at = now;
                    let id = invoice.id.to_string();
                    invoices
                        .update(Some(&mut tx), &id, invoice)
                        .await
                        .map(InvoiceDTO::from)
                })
            })
            .await
        })
        .await
    }

    fn validate_invoice_credit_input(input: &ApplyInvoiceCreditsInput) -> Result<()> {
        if input.invoice_id.trim().is_empty() {
            return Err(Error::Invalid {
                code: "missing_invoice_id",
                message: "invoice_id is required to apply credits".to_string(),
                field: Some("invoice_id".to_string()),
            });
        }
        if input.reason.trim().is_empty() {
            return Err(Error::Invalid {
                code: "missing_invoice_credit_reason",
                message: "a credit or discount must record why it was applied".to_string(),
                field: Some("reason".to_string()),
            });
        }
        Ok(())
    }

    /// Calculates signed proration lines for a mid-period plan change.
    ///
    /// The elapsed/remainder split uses [`stheno_types::Money::allocate_proportionally`]
    /// so the two parts reconstruct each full-period plan price exactly, even when the
    /// period length does not divide cleanly into minor units.
    pub fn calculate_proration(input: CalculateProrationInput) -> Result<ProrationDTO> {
        if input.old_period_amount.currency != input.new_period_amount.currency {
            return Err(Error::CurrencyMismatch {
                expected: input.old_period_amount.currency,
                found: input.new_period_amount.currency,
            });
        }
        if input.period_end <= input.period_start {
            return Err(Error::Invalid {
                code: "invalid_invoice_period",
                message: "period_end must be after period_start".to_string(),
                field: Some("period_end".to_string()),
            });
        }
        if input.change_at <= input.period_start || input.change_at >= input.period_end {
            return Err(Error::Invalid {
                code: "invalid_proration_change_at",
                message: "change_at must fall inside the billed period".to_string(),
                field: Some("change_at".to_string()),
            });
        }

        let elapsed = Decimal::from((input.change_at - input.period_start).num_seconds());
        let remaining = Decimal::from((input.period_end - input.change_at).num_seconds());

        let old_parts = input
            .old_period_amount
            .allocate_proportionally(&[elapsed, remaining])?;
        let new_parts = input
            .new_period_amount
            .allocate_proportionally(&[elapsed, remaining])?;

        let unused_old = old_parts[1];
        let new_remainder = new_parts[1];
        let credit = stheno_types::Money::new(-unused_old.amount, unused_old.currency);
        let net_adjustment = (new_remainder + credit)?;

        Ok(ProrationDTO {
            credit,
            charge: new_remainder,
            net_adjustment,
            lines: vec![
                ProrationLineDTO {
                    kind: "proration_credit".to_string(),
                    description: format!("Unused {} subscription credit", input.old_tier),
                    amount: credit,
                },
                ProrationLineDTO {
                    kind: "proration_charge".to_string(),
                    description: format!("{} subscription remainder", input.new_tier),
                    amount: new_remainder,
                },
            ],
        })
    }

    /// Calculates the Bangladesh VAT line for an invoice subtotal.
    ///
    /// VAT is rounded once from the invoice subtotal. It is deliberately not calculated
    /// per line, because line-level rounding can over- or under-collect when fractional
    /// minor units are involved.
    pub fn calculate_vat(input: CalculateVatInput) -> Result<VatLineDTO> {
        if input.taxable_subtotal.currency != stheno_types::currency::CurrencyCode::Bdt {
            return Err(Error::Invalid {
                code: "unsupported_vat_currency",
                message: "Bangladesh VAT applies to BDT invoices".to_string(),
                field: Some("taxable_subtotal".to_string()),
            });
        }
        if input.taxable_subtotal.is_negative() {
            return Err(Error::Invalid {
                code: "negative_taxable_subtotal",
                message: "VAT cannot be calculated on a negative taxable subtotal".to_string(),
                field: Some("taxable_subtotal".to_string()),
            });
        }
        if input.rate.is_sign_negative() {
            return Err(Error::Invalid {
                code: "negative_vat_rate",
                message: "VAT rate cannot be negative".to_string(),
                field: Some("rate".to_string()),
            });
        }

        let amount = (input.taxable_subtotal.amount * input.rate)
            .round_dp(input.taxable_subtotal.currency.minor_unit_exponent());

        Ok(VatLineDTO {
            kind: "tax".to_string(),
            jurisdiction: "BD".to_string(),
            mushak_form: "Mushak-6.3".to_string(),
            rate: input.rate,
            amount: stheno_types::Money::new(amount, input.taxable_subtotal.currency),
        })
    }

    /// Writes or reloads the outbound ledger row for one collection attempt.
    ///
    /// A provider call must be made only after this row exists. Retrying the same
    /// idempotency key returns the same row; reusing the key for different payload is a
    /// caller bug and is refused before any provider can be called.
    pub async fn start_billing_attempt(
        &self,
        input: StartBillingAttemptInput,
    ) -> Result<BillingAttemptDTO> {
        Self::validate_billing_attempt_input(&input)?;
        let now = Utc::now();
        let row = BillingAttempt {
            id: Id::new(),
            invoice_id: input.invoice_id.clone(),
            tenant_id: input.tenant_id.clone(),
            provider_id: input.provider_id.clone(),
            idempotency_key: input.idempotency_key.clone(),
            request_hash: input.request_hash.clone(),
            state: "in_flight".to_string(),
            amount: input.amount.amount,
            currency: input.amount.currency,
            provider_reference: None,
            error: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        match tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.invoices.retrieve(None, &input.invoice_id).await?;
            self.billing_attempts.create(None, row).await
        })
        .await
        {
            Ok(created) => Ok(BillingAttemptDTO::from(created)),
            Err(Error::Conflict { .. }) => {
                let existing = self
                    .billing_attempt_by_key(&input.idempotency_key)
                    .await?
                    .ok_or_else(|| Error::Conflict {
                        code: "billing_attempt_conflict_unresolved",
                        message: "billing attempt conflicted but no existing row was visible"
                            .to_string(),
                    })?;
                if existing.request_hash != input.request_hash {
                    return Err(Error::Invalid {
                        code: "idempotency_key_reused",
                        message: "billing idempotency key was reused with a different request"
                            .to_string(),
                        field: Some("idempotency_key".to_string()),
                    });
                }
                Ok(BillingAttemptDTO::from(existing))
            }
            Err(other) => Err(other),
        }
    }

    pub async fn mark_billing_attempt_unknown(
        &self,
        id: &str,
        error: &str,
    ) -> Result<BillingAttemptDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.billing_attempts.retrieve(None, id).await?;
            row.state = "unknown".to_string();
            row.error = Some(error.to_string());
            row.updated_at = Utc::now();
            self.billing_attempts
                .update(None, id, row)
                .await
                .map(BillingAttemptDTO::from)
        })
        .await
    }

    fn validate_billing_attempt_input(input: &StartBillingAttemptInput) -> Result<()> {
        for (field, value) in [
            ("invoice_id", &input.invoice_id),
            ("tenant_id", &input.tenant_id),
            ("provider_id", &input.provider_id),
            ("idempotency_key", &input.idempotency_key),
            ("request_hash", &input.request_hash),
        ] {
            if value.trim().is_empty() {
                return Err(Error::Invalid {
                    code: "missing_billing_attempt_field",
                    message: format!("{field} is required to collect an invoice"),
                    field: Some(field.to_string()),
                });
            }
        }
        if input.amount.is_negative() {
            return Err(Error::Invalid {
                code: "negative_billing_attempt_amount",
                message: "a collection attempt cannot charge a negative amount".to_string(),
                field: Some("amount".to_string()),
            });
        }
        Ok(())
    }

    async fn billing_attempt_by_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<BillingAttempt>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("idempotency_key", FilterOp::Eq(idempotency_key.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            let mut rows = self.billing_attempts.list(None, &spec).await?;
            Ok::<_, Error>(rows.pop())
        })
        .await
    }

    pub async fn list_billing_attempts(&self) -> Result<Vec<BillingAttemptDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_order("created_at", OrderDirection::Desc)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.billing_attempts.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(BillingAttemptDTO::from).collect())
    }

    /// Advances one unpaid subscription by a single dunning rung.
    pub async fn advance_dunning(&self, input: DunningInput) -> Result<SubscriptionDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut sub = self.subscriptions.retrieve(None, &input.tenant_id).await?;

            if sub.next_attempt_at.is_some_and(|at| input.now < at) {
                return Ok(SubscriptionDTO::from(sub));
            }

            match sub.status.as_str() {
                "active" => {
                    sub.status = "past_due".to_string();
                    sub.dunning_attempts += 1;
                    sub.next_attempt_at = Some(input.now + chrono::Duration::days(1));
                }
                "past_due" => {
                    sub.status = "grace".to_string();
                    sub.dunning_attempts += 1;
                    sub.next_attempt_at = Some(input.now + chrono::Duration::days(3));
                }
                "grace" => {
                    sub.status = "suspended".to_string();
                    sub.dunning_attempts += 1;
                    sub.next_attempt_at = None;
                }
                "suspended" | "cancelled" => {}
                other => {
                    return Err(Error::Invalid {
                        code: "unknown_subscription_status",
                        message: format!("cannot advance dunning from {other}"),
                        field: Some("status".to_string()),
                    });
                }
            }
            sub.updated_at = input.now;
            let id = sub.tenant_id.clone();
            self.subscriptions
                .update(None, &id, sub)
                .await
                .map(SubscriptionDTO::from)
        })
        .await
    }

    /// Resets dunning after collection succeeds.
    pub async fn mark_invoice_paid(&self, tenant_id: &str) -> Result<SubscriptionDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut sub = self.subscriptions.retrieve(None, tenant_id).await?;
            sub.status = "active".to_string();
            sub.dunning_attempts = 0;
            sub.next_attempt_at = None;
            sub.updated_at = Utc::now();
            self.subscriptions
                .update(None, tenant_id, sub)
                .await
                .map(SubscriptionDTO::from)
        })
        .await
    }

    /// Explicit operator escape hatch for status changes outside the dunning machine.
    pub async fn override_subscription_status(
        &self,
        tenant_id: &str,
        status: &str,
    ) -> Result<SubscriptionDTO> {
        if !matches!(
            status,
            "active" | "past_due" | "grace" | "suspended" | "cancelled"
        ) {
            return Err(Error::Invalid {
                code: "unknown_subscription_status",
                message: format!("unknown subscription status {status}"),
                field: Some("status".to_string()),
            });
        }

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut sub = self.subscriptions.retrieve(None, tenant_id).await?;
            sub.status = status.to_string();
            sub.updated_at = Utc::now();
            self.subscriptions
                .update(None, tenant_id, sub)
                .await
                .map(SubscriptionDTO::from)
        })
        .await
    }

    /// Returns the notifications that correspond to one dunning status.
    pub fn dunning_notifications(status: &str) -> Result<Vec<DunningNotificationDTO>> {
        let template = match status {
            "active" | "cancelled" => return Ok(Vec::new()),
            "past_due" => "billing_past_due",
            "grace" => "billing_grace",
            "suspended" => "billing_suspended",
            other => {
                return Err(Error::Invalid {
                    code: "unknown_subscription_status",
                    message: format!("unknown subscription status {other}"),
                    field: Some("status".to_string()),
                });
            }
        };

        Ok(["email", "sms"]
            .into_iter()
            .map(|channel| DunningNotificationDTO {
                channel: channel.to_string(),
                template: template.to_string(),
            })
            .collect())
    }

    // -- Operator identity ------------------------------------------------------------
    //
    // ADR-012 makes the control plane the one surface that reads and writes across every
    // merchant. A single shared secret gates it adequately and identifies nobody: it
    // cannot say who moved a merchant between plans, cannot be revoked for one person,
    // and cannot be rotated without locking out everyone holding it.

    /// Issues a named credential. The plaintext token is returned once and never again —
    /// only its SHA-256 is stored, so a database read cannot recover it.
    pub async fn create_operator(&self, name: &str) -> Result<IssuedOperator> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::Invalid {
                code: "invalid_operator_name",
                message: "an operator must be named, so the audit trail says who".to_string(),
                field: Some("name".to_string()),
            });
        }

        let random: [u8; 32] = rand::thread_rng().gen();
        let random_hex = hex::encode(random);
        let token = format!("{OPERATOR_TOKEN_PREFIX}{random_hex}");
        let redacted_token = format!(
            "{OPERATOR_TOKEN_PREFIX}...{}",
            &random_hex[random_hex.len() - 4..]
        );

        let now = Utc::now();
        let row = Operator {
            id: Id::new(),
            name: name.to_string(),
            token_hash: hash_token(&token),
            redacted_token,
            created_at: now,
            last_used_at: None,
            revoked_at: None,
            // A new credential starts as `owner` only because that is what every operator
            // issued before P37-25 already was. Narrowing it is `set_operator_role`, and
            // that write is audited like every other.
            role: "owner".to_string(),
            mfa_secret: None,
            mfa_enabled: false,
            ip_allowlist: serde_json::json!([]),
            session_ttl_minutes: None,
            deleted_at: None,
        };

        let created = tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.operators.create(None, row).await
        })
        .await?;

        Ok(IssuedOperator {
            operator: OperatorDTO::from(created),
            token,
        })
    }

    /// Every credential ever issued, revoked ones included — a revoked credential that
    /// disappeared from the list would look like one that was never issued.
    pub async fn list_operators(&self) -> Result<Vec<OperatorDTO>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_order("created_at", OrderDirection::Desc)
                .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
            self.operators.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(OperatorDTO::from).collect())
    }

    /// Retires one credential without touching the others. Revoking twice succeeds: the
    /// caller's intent is "this must not work", and it already does not.
    pub async fn revoke_operator(&self, id: &str) -> Result<OperatorDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.operators.retrieve(None, id).await?;
            if row.revoked_at.is_none() {
                row.revoked_at = Some(Utc::now());
                row = self.operators.update(None, id, row).await?;
            }
            Ok::<_, Error>(OperatorDTO::from(row))
        })
        .await
    }

    /// Resolves a presented token to the operator holding it.
    ///
    /// Looked up by hash, so the comparison happens on an indexed equality inside
    /// Postgres rather than on a secret in process memory — there is no timing channel to
    /// leak a prefix of the real token, because the real token is not in the process.
    /// A revoked credential resolves to nothing, which is the whole point of revocation.
    pub async fn authenticate_operator(&self, token: &str) -> Result<OperatorDTO> {
        let token_hash = hash_token(token);

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("token_hash", FilterOp::Eq(token_hash))
                .with_predicate("revoked_at", FilterOp::IsNull)
                .with_pagination(0, 1);

            let mut found = self.operators.list(None, &spec).await?;
            let mut row = found.pop().ok_or(Error::NotFound {
                entity: "operator",
                id: "token".to_string(),
            })?;

            // Written on use so an operator can tell a live credential from a forgotten
            // one before revoking it. The audit trail only records writes, so without
            // this a read-only operator is indistinguishable from a dead token.
            // ponytail: one UPDATE per control-plane request; add a staleness threshold
            // if this surface ever carries more than operator-paced traffic.
            let id = row.id.to_string();
            row.last_used_at = Some(Utc::now());
            let row = self.operators.update(None, &id, row).await?;

            Ok::<_, Error>(OperatorDTO::from(row))
        })
        .await
    }

    /// Appends one row to the control-plane audit trail.
    ///
    /// Append-only: there is deliberately no update and no delete counterpart. The
    /// caller decides what goes in `diff`; an issued token never does.
    pub async fn record_operator_action(&self, entry: NewOperatorAudit) -> Result<()> {
        let row = OperatorAudit {
            id: Id::new(),
            actor_id: entry.actor_id,
            actor_name: entry.actor_name,
            action: entry.action,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            diff: entry.diff,
            created_at: Utc::now(),
        };

        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.operator_audit.create(None, row).await
        })
        .await
        .map(|_| ())
    }

    /// One page of the audit trail, newest first.
    ///
    /// `entity_id` narrows it to one merchant or one plan. `before` is the `id` of the
    /// last row of the previous page.
    ///
    /// # Why the cursor is `id` and not `created_at` with an offset
    ///
    /// The trail only ever grows, and it grows at the end being read. An `OFFSET` page 2
    /// taken a moment after page 1 re-reads rows that page 1 already showed, because
    /// everything shifted down — the classic drifting-pagination bug, and on an audit
    /// trail it reads as duplicated history.
    ///
    /// `id` is a ULID, so its leading bits are the creation timestamp: it sorts by time
    /// exactly as `created_at` does, and unlike `created_at` it is unique. That makes
    /// `id < cursor` a boundary that cannot skip a row or repeat one, even for rows
    /// written in the same millisecond.
    pub async fn list_operator_audit(
        &self,
        entity_id: Option<&str>,
        limit: usize,
        before: Option<&str>,
    ) -> Result<Vec<OperatorAuditDTO>> {
        let limit = limit.clamp(1, stheno_db::filter::MAX_PAGE_LIMIT);
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut spec = FilterSpec::new()
                .with_order("id", OrderDirection::Desc)
                .with_pagination(0, limit);
            if let Some(id) = entity_id {
                spec = spec.with_predicate("entity_id", FilterOp::Eq(id.to_string()));
            }
            if let Some(cursor) = before {
                spec = spec.with_predicate("id", FilterOp::Lt(cursor.to_string()));
            }
            self.operator_audit.list(None, &spec).await
        })
        .await
        .map(|rows| rows.into_iter().map(OperatorAuditDTO::from).collect())
    }

    // ── P37-02/03/04/08: control-plane state that used to live in process memory ──────
    //
    // Every registry these replace was a `static OnceLock<RwLock<HashMap<..>>>` in
    // crates/api. They live here rather than at the API layer because
    // `check-sql-boundaries.sh` derives a module's schema from its crate name, and the
    // `platform` schema belongs to this crate.

    /// A flag by name, or `None` when nothing has ever set it.
    ///
    /// `None` means enabled, for both kinds. A kill switch is an override, not an
    /// authorisation: if an absent row meant "killed", every gateway would be dead on a
    /// fresh deployment. The closed state for an override is "not overriding".
    pub async fn get_global_flag(&self, key: &str) -> Result<Option<GlobalFlag>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("key", FilterOp::Eq(key.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            Ok::<_, Error>(
                self.global_flags
                    .list(None, &spec)
                    .await?
                    .into_iter()
                    .next(),
            )
        })
        .await
    }

    /// True unless something has explicitly disabled it. See [`Self::get_global_flag`].
    pub async fn is_globally_enabled(&self, key: &str) -> Result<bool> {
        Ok(self
            .get_global_flag(key)
            .await?
            .map(|flag| flag.is_enabled)
            .unwrap_or(true))
    }

    pub async fn list_global_flags(&self, kind: &str) -> Result<Vec<GlobalFlag>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("kind", FilterOp::Eq(kind.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull);
            self.global_flags.list(None, &spec).await
        })
        .await
    }

    /// Sets a flag, recording who and why. `reason` is not decoration: an incident switch
    /// with no reason is unreadable a week later, when someone has to decide whether it is
    /// safe to turn back on.
    pub async fn set_global_flag(
        &self,
        key: &str,
        kind: &str,
        is_enabled: bool,
        reason: Option<String>,
        actor_id: &str,
    ) -> Result<GlobalFlag> {
        if kind != "gateway" && kind != "feature" {
            return Err(Error::Invalid {
                code: "invalid_flag_kind",
                message: "a global flag is either a gateway or a feature".to_string(),
                field: Some("kind".to_string()),
            });
        }
        let existing = self.get_global_flag(key).await?;
        let row = GlobalFlag {
            key: key.to_string(),
            kind: kind.to_string(),
            description: existing
                .as_ref()
                .map(|f| f.description.clone())
                .unwrap_or_default(),
            is_enabled,
            // A gateway is on or off. A partial kill switch would mean some merchants take
            // payments and some do not with no way to say which; the migration's CHECK
            // enforces the same thing at the database boundary.
            rollout_percentage: if kind == "gateway" {
                100
            } else {
                existing
                    .as_ref()
                    .map(|f| f.rollout_percentage)
                    .unwrap_or(100)
            },
            reason,
            updated_by: Some(actor_id.to_string()),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.global_flags.upsert(None, row).await
        })
        .await
    }

    /// Issues a support session and returns the plaintext token exactly once.
    ///
    /// The token itself is never stored, only its SHA-256, for the same reason
    /// `platform.operator` stores a hash: a support credential sitting in a table any
    /// operator can read is a leak with an audit trail attached.
    pub async fn start_impersonation(
        &self,
        tenant_id: &str,
        operator_id: &str,
        reason: &str,
        scopes: serde_json::Value,
        duration_minutes: i64,
    ) -> Result<(ImpersonationSession, String)> {
        if reason.trim().is_empty() {
            return Err(Error::Invalid {
                code: "impersonation_reason_required",
                message: "ADR-018: a support session with no stated reason is not consented to"
                    .to_string(),
                field: Some("reason".to_string()),
            });
        }
        let random: [u8; 32] = rand::thread_rng().gen();
        let random_hex = hex::encode(random);
        let token = format!("imp_{random_hex}");
        let now = Utc::now();
        let row = ImpersonationSession {
            id: Id::new(),
            token_hash: hash_token(&token),
            redacted_token: format!("imp_...{}", &random_hex[random_hex.len() - 4..]),
            tenant_id: tenant_id.to_string(),
            operator_id: operator_id.to_string(),
            reason: reason.to_string(),
            scopes,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(duration_minutes),
            revoked_at: None,
            deleted_at: None,
        };
        let created = tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.impersonation_sessions.create(None, row).await
        })
        .await?;
        Ok((created, token))
    }

    /// Resolves a token to a session that is neither revoked nor expired.
    ///
    /// Expiry is read from the row on every use rather than tracked by a timer. A timer
    /// dies with the process holding it, which is the whole class of bug this table exists
    /// to remove.
    pub async fn live_impersonation_session(
        &self,
        token: &str,
    ) -> Result<Option<ImpersonationSession>> {
        let token_hash = hash_token(token);
        let found = tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("token_hash", FilterOp::Eq(token_hash))
                .with_predicate("revoked_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            Ok::<_, Error>(
                self.impersonation_sessions
                    .list(None, &spec)
                    .await?
                    .into_iter()
                    .next(),
            )
        })
        .await?;
        Ok(found.filter(|session| session.is_live(Utc::now())))
    }

    /// Ends a session early. Revocation is a timestamp, never a delete: the audit rows
    /// naming this session must still resolve afterwards.
    pub async fn end_impersonation(&self, token: &str) -> Result<bool> {
        let token_hash = hash_token(token);
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("token_hash", FilterOp::Eq(token_hash))
                .with_predicate("revoked_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            let Some(mut row) = self
                .impersonation_sessions
                .list(None, &spec)
                .await?
                .into_iter()
                .next()
            else {
                return Ok::<_, Error>(false);
            };
            let id = row.id.to_string();
            row.revoked_at = Some(Utc::now());
            self.impersonation_sessions.update(None, &id, row).await?;
            Ok::<_, Error>(true)
        })
        .await
    }

    /// Puts one merchant read-only during an incident, or lifts it.
    pub async fn set_tenant_read_only(
        &self,
        tenant_id: &str,
        read_only: bool,
        reason: Option<String>,
    ) -> Result<Tenant> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.tenants.retrieve(None, tenant_id).await?;
            row.is_read_only = read_only;
            row.read_only_reason = if read_only { reason } else { None };
            row.read_only_since = if read_only { Some(Utc::now()) } else { None };
            row.updated_at = Utc::now();
            self.tenants.update(None, tenant_id, row).await
        })
        .await
    }

    /// Whether writes are currently refused for this merchant.
    ///
    /// An unknown tenant is `false`, not an error: this is consulted on the request path,
    /// and a lookup miss must not become a refusal for a tenant that was never registered
    /// here. Whether the tenant exists at all is decided before this is called.
    pub async fn is_tenant_read_only(&self, tenant_id: &str) -> Result<bool> {
        let found = tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.tenants.retrieve(None, tenant_id).await
        })
        .await;
        match found {
            Ok(row) => Ok(row.is_read_only),
            Err(Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub async fn get_tenant_theme(&self, tenant_id: &str) -> Result<Option<TenantTheme>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("tenant_id", FilterOp::Eq(tenant_id.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            Ok::<_, Error>(
                self.tenant_themes
                    .list(None, &spec)
                    .await?
                    .into_iter()
                    .next(),
            )
        })
        .await
    }

    pub async fn list_tenant_themes(&self) -> Result<Vec<TenantTheme>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
            self.tenant_themes.list(None, &spec).await
        })
        .await
    }

    /// Pins a merchant to a theme at a version. One row per tenant, so a merchant cannot
    /// be on two themes at once.
    pub async fn assign_tenant_theme(
        &self,
        tenant_id: &str,
        theme_id: &str,
        version: &str,
        is_custom: bool,
        actor_id: &str,
    ) -> Result<TenantTheme> {
        let row = TenantTheme {
            tenant_id: tenant_id.to_string(),
            theme_id: theme_id.to_string(),
            version: version.to_string(),
            is_custom,
            assigned_by: Some(actor_id.to_string()),
            assigned_at: Utc::now(),
            deleted_at: None,
        };
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.tenant_themes.upsert(None, row).await
        })
        .await
    }

    /// Deployment defaults, seeded on first read.
    ///
    /// Seeding rather than returning a synthetic default keeps what an operator sees and
    /// what is stored the same thing. Otherwise the dashboard shows a default no row
    /// holds, and the next writer silently overwrites a value nobody chose.
    pub async fn get_deployment_config(&self) -> Result<DeploymentConfig> {
        let existing = tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new()
                .with_predicate("id", FilterOp::Eq(DeploymentConfig::SINGLETON.to_string()))
                .with_pagination(0, 1);
            Ok::<_, Error>(
                self.deployment_config
                    .list(None, &spec)
                    .await?
                    .into_iter()
                    .next(),
            )
        })
        .await?;

        if let Some(row) = existing {
            return Ok(row);
        }

        // The values the in-memory registries shipped with, carried over verbatim. Moving
        // state into a table must not change what a deployment starts with — a merchant
        // reading a different default after an upgrade is a behaviour change wearing a
        // persistence change's clothes.
        let seed = DeploymentConfig {
            id: DeploymentConfig::SINGLETON.to_string(),
            default_region: "BD".to_string(),
            default_currency: "BDT".to_string(),
            default_tax_rate: Decimal::new(5, 2),
            support_email: "support@stheno.local".to_string(),
            email_provider: "smtp".to_string(),
            sms_provider: "twilio".to_string(),
            from_email: "no-reply@stheno.local".to_string(),
            order_confirmation_template: "Your order {{order_id}} is confirmed!".to_string(),
            updated_by: None,
            updated_at: Utc::now(),
            deleted_at: None,
        };
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.deployment_config.upsert(None, seed).await
        })
        .await
    }

    pub async fn save_deployment_config(
        &self,
        mut config: DeploymentConfig,
        actor_id: &str,
    ) -> Result<DeploymentConfig> {
        config.id = DeploymentConfig::SINGLETON.to_string();
        config.updated_by = Some(actor_id.to_string());
        config.updated_at = Utc::now();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.deployment_config.upsert(None, config).await
        })
        .await
    }

    pub async fn create_announcement(
        &self,
        title: &str,
        message: &str,
        severity: &str,
        expires_at: chrono::DateTime<Utc>,
        actor_id: &str,
    ) -> Result<Announcement> {
        if !matches!(severity, "info" | "warning" | "critical") {
            return Err(Error::Invalid {
                code: "invalid_severity",
                message: "severity is info, warning or critical".to_string(),
                field: Some("severity".to_string()),
            });
        }
        let row = Announcement {
            id: Id::new(),
            title: title.to_string(),
            message: message.to_string(),
            severity: severity.to_string(),
            created_by: Some(actor_id.to_string()),
            created_at: Utc::now(),
            expires_at,
            deleted_at: None,
        };
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.announcements.create(None, row).await
        })
        .await
    }

    /// Announcements a merchant should currently see. Expired rows are filtered out, never
    /// deleted: the record of what merchants were told during an outage has to survive it.
    pub async fn list_live_announcements(&self) -> Result<Vec<Announcement>> {
        let now = Utc::now();
        Ok(self
            .list_all_announcements()
            .await?
            .into_iter()
            .filter(|a| a.expires_at > now)
            .collect())
    }

    /// Every announcement including expired ones, for the operator's own view.
    pub async fn list_all_announcements(&self) -> Result<Vec<Announcement>> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
            self.announcements.list(None, &spec).await
        })
        .await
    }

    /// One operator by id, or `None`.
    ///
    /// The access checks call this. An unknown id must resolve to nothing — the in-memory
    /// profile map it replaces answered with a default carrying `role: owner`, which made
    /// the unknown case the most privileged one rather than the least.
    pub async fn get_operator(&self, operator_id: &str) -> Result<Option<Operator>> {
        let found = tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.operators.retrieve(None, operator_id).await
        })
        .await;
        match found {
            Ok(row) => Ok(Some(row)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn set_operator_role(&self, operator_id: &str, role: &str) -> Result<Operator> {
        if !matches!(role, "owner" | "billing" | "support" | "engineer") {
            return Err(Error::Invalid {
                code: "invalid_operator_role",
                message: "role is owner, billing, support or engineer".to_string(),
                field: Some("role".to_string()),
            });
        }
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.operators.retrieve(None, operator_id).await?;
            row.role = role.to_string();
            self.operators.update(None, operator_id, row).await
        })
        .await
    }

    /// Records an MFA secret without turning enforcement on.
    ///
    /// Enrolment and enforcement are separate deliberately: flipping `mfa_enabled` here
    /// would lock out an operator who started setup and never finished scanning the code.
    pub async fn begin_operator_mfa(&self, operator_id: &str, secret: &str) -> Result<Operator> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.operators.retrieve(None, operator_id).await?;
            row.mfa_secret = Some(secret.to_string());
            row.mfa_enabled = false;
            self.operators.update(None, operator_id, row).await
        })
        .await
    }

    /// Turns enforcement on, once a code has actually verified against the secret.
    pub async fn confirm_operator_mfa(&self, operator_id: &str) -> Result<Operator> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.operators.retrieve(None, operator_id).await?;
            if row.mfa_secret.is_none() {
                return Err(Error::Invalid {
                    code: "mfa_not_enrolled",
                    message: "no MFA secret has been issued for this operator".to_string(),
                    field: None,
                });
            }
            row.mfa_enabled = true;
            self.operators.update(None, operator_id, row).await
        })
        .await
    }

    pub async fn set_operator_network_limits(
        &self,
        operator_id: &str,
        ip_allowlist: Vec<String>,
        session_ttl_minutes: Option<i32>,
    ) -> Result<Operator> {
        if session_ttl_minutes.is_some_and(|ttl| ttl <= 0) {
            return Err(Error::Invalid {
                code: "invalid_session_ttl",
                message: "a session TTL must be a positive number of minutes".to_string(),
                field: Some("session_ttl_minutes".to_string()),
            });
        }
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut row = self.operators.retrieve(None, operator_id).await?;
            row.ip_allowlist = serde_json::json!(ip_allowlist);
            row.session_ttl_minutes = session_ttl_minutes;
            self.operators.update(None, operator_id, row).await
        })
        .await
    }

    /// Get the current FX rate for a given currency pair (e.g. "USD/BDT")
    pub async fn get_fx_rate(&self, currency_pair: &str) -> Result<FxRateDTO> {
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.fx_rates.retrieve(None, currency_pair).await
        })
        .await
        .map(FxRateDTO::from)
    }

    /// Upsert an FX rate from a background job.
    pub async fn upsert_fx_rate(
        &self,
        currency_pair: &str,
        rate: Decimal,
        source: &str,
        rate_kind: &str,
    ) -> Result<FxRateDTO> {
        let now = Utc::now();
        let row = FxRate {
            id: Id::new(),
            currency_pair: currency_pair.to_string(),
            rate,
            source: source.to_string(),
            rate_kind: rate_kind.to_string(),
            operator_id: None,
            rate_date: now,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            self.fx_rates.upsert(None, row).await
        })
        .await
        .map(FxRateDTO::from)
    }

    /// Override an FX rate manually, logging the action.
    pub async fn override_fx_rate(
        &self,
        operator_id: &str,
        currency_pair: &str,
        rate: Decimal,
    ) -> Result<FxRateDTO> {
        let now = Utc::now();
        let row = FxRate {
            id: Id::new(),
            currency_pair: currency_pair.to_string(),
            rate,
            source: "operator_override".to_string(),
            rate_kind: "override".to_string(),
            operator_id: Some(operator_id.to_string()),
            rate_date: now,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        
        let audit = OperatorAudit {
            id: Id::new(),
            actor_id: operator_id.to_string(),
            actor_name: "operator".to_string(),
            action: "override_fx_rate".to_string(),
            entity_type: "fx_rate".to_string(),
            entity_id: currency_pair.to_string(),
            diff: Some(serde_json::json!({ "rate": rate.to_string() })),
            created_at: now,
        };

        let operator_audit = self.operator_audit.clone();
        let fx_rates = self.fx_rates.clone();
        tenant::scope(tenant::PLATFORM_SCOPE, async {
            let mut conn = self.pool.get().await.map_err(|e| Error::Database(e.to_string()))?;
            stheno_db::transaction::with_transaction(&mut conn, move |conn| {
                Box::pin(async move {
                    let mut tx = conn;
                    operator_audit.create(Some(&mut tx), audit).await?;
                    let upserted = fx_rates.upsert(Some(&mut tx), row).await?;
                    Ok(upserted)
                })
            }).await
        })
        .await
        .map(FxRateDTO::from)
    }

    pub async fn upload_payment_proof(
        &self,
        tenant_id: &str,
        invoice_id: &str,
        input: crate::dto::UploadPaymentProofInput,
    ) -> Result<crate::dto::InvoicePaymentProofDTO> {
        let now = Utc::now();
        let invoices = self.invoices.clone();
        let payment_proofs = self.payment_proofs.clone();
        
        let tenant_id = tenant_id.to_string();
        let invoice_id = invoice_id.to_string();

        tenant::scope(tenant::PLATFORM_SCOPE, async move {
            let mut conn = self.pool.get().await.map_err(|e| Error::Database(e.to_string()))?;
            stheno_db::transaction::with_transaction(&mut conn, move |conn| {
                Box::pin(async move {
                    let mut tx = conn;
                    
                    // Verify the invoice exists and belongs to the tenant
                    let mut invoice = invoices.retrieve(Some(&mut tx), &invoice_id).await?;
                    if invoice.tenant_id != tenant_id {
                        return Err(Error::NotFound { entity: "invoice", id: invoice_id.clone() });
                    }
                    
                    // Duplicate check inside the same method/tenant. We don't rely solely on DB constraint to give a good error.
                    // Instead, we just let the DB unique index fail.
                    let row = crate::models::InvoicePaymentProof {
                        id: Id::new(),
                        invoice_id: invoice_id.clone(),
                        tenant_id: tenant_id.clone(),
                        payment_method: input.payment_method,
                        transaction_id: input.transaction_id,
                        proof_asset_url: input.proof_asset_url,
                        operator_id: None,
                        reviewed_at: None,
                        status: "pending".to_string(),
                        rejection_reason: None,
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                    };
                    
                    let created = payment_proofs.create(Some(&mut tx), row).await.map_err(|e| {
                        if let Error::Conflict { .. } = e {
                            Error::Invalid {
                                code: "duplicate_transaction",
                                message: "This transaction ID has already been submitted for this payment method.".to_string(),
                                field: Some("transaction_id".to_string()),
                            }
                        } else {
                            e
                        }
                    })?;
                    
                    // Update invoice status
                    invoice.status = "pending_verification".to_string();
                    invoice.updated_at = now;
                    invoices.update(Some(&mut tx), &invoice_id, invoice).await?;
                    
                    Ok(created)
                })
            }).await
        })
        .await
        .map(crate::dto::InvoicePaymentProofDTO::from)
    }

    pub async fn review_payment_proof(
        &self,
        operator_id: &str,
        proof_id: &str,
        input: crate::dto::ReviewPaymentProofInput,
    ) -> Result<crate::dto::InvoicePaymentProofDTO> {
        let now = Utc::now();
        let payment_proofs = self.payment_proofs.clone();
        let invoices = self.invoices.clone();
        
        let proof_id = proof_id.to_string();
        let operator_id = operator_id.to_string();

        tenant::scope(tenant::PLATFORM_SCOPE, async move {
            let mut conn = self.pool.get().await.map_err(|e| Error::Database(e.to_string()))?;
            stheno_db::transaction::with_transaction(&mut conn, move |conn| {
                Box::pin(async move {
                    let mut tx = conn;
                    
                    let mut proof = payment_proofs.retrieve(Some(&mut tx), &proof_id).await?;
                    if proof.status != "pending" {
                        return Err(Error::Invalid {
                            code: "invalid_status",
                            message: "Payment proof is not pending.".to_string(),
                            field: None,
                        });
                    }

                    proof.operator_id = Some(operator_id.clone());
                    proof.reviewed_at = Some(now);
                    proof.updated_at = now;
                    
                    let mut invoice = invoices.retrieve(Some(&mut tx), &proof.invoice_id).await?;

                    if input.approved {
                        proof.status = "approved".to_string();
                        invoice.status = "paid".to_string();
                        invoice.paid_at = Some(now);
                    } else {
                        proof.status = "rejected".to_string();
                        proof.rejection_reason = input.rejection_reason;
                        invoice.status = "needs_more_info".to_string();
                    }

                    invoices.update(Some(&mut tx), &proof.invoice_id, invoice).await?;
                    let updated_proof = payment_proofs.update(Some(&mut tx), &proof_id, proof).await?;
                    
                    Ok(updated_proof)
                })
            }).await
        })
        .await
        .map(crate::dto::InvoicePaymentProofDTO::from)
    }
}
