//! `stheno-api-platform`: the control-plane surface (ADR-012).
//!
//! This is the API behind `apps/platform` — the operator's view of every merchant on the
//! deployment. It is deliberately separate from the admin surface:
//!
//! * `/admin/*` is a merchant administering their own store. Every route there is
//!   tenant-scoped, and a merchant must never reach the control plane.
//! * `/platform/*` is the business operating the platform. It reads across tenants, so it
//!   runs outside tenant scope and is gated on a different credential entirely.
//!
//! Provisioning a merchant lives here rather than at `POST /admin/stores`, because a
//! store is a tenant: creating one is a platform operation, not a store operation.
//!
//! # Why plan *definitions* are writable here
//!
//! ENTITLEMENTS.md 5b put tier definitions in `config/tiers.toml` so that retuning a plan
//! never needs a release. In a hosted deployment that file is baked into a container
//! image, so the promise only holds if there is a writable layer above it — that is
//! `platform.tier_definition`, and these are the routes that write it. A tier with no
//! stored row still means exactly what the shipped TOML says.
//!
//! Editing a plan is deliberately **not** on `/admin/*`. A merchant who can define what
//! Pro includes is a merchant who is on Pro for free.
//!
//! # Who is calling
//!
//! Every route here resolves the caller to an [`Actor`] before it does anything, and every
//! write it makes is recorded against that actor. Two credential kinds resolve:
//!
//! * A **named operator** — a `platform.operator` row, issued through this API, revocable
//!   on its own and rotatable without disturbing anyone else's.
//! * The **bootstrap token** — `STHENO_PLATFORM_TOKEN`. It is how the first operator gets
//!   issued on a fresh deployment, and it stays valid afterwards on purpose: revoking the
//!   last named operator must not be able to lock everyone out of their own control plane.
//!   Its actions are attributed to `op_bootstrap`, which is exactly as specific as a
//!   shared secret can honestly be.
//!
//! An unset `STHENO_PLATFORM_TOKEN` still disables the bootstrap path entirely rather than
//! opening it; named operators, if any exist, keep working.

pub mod analytics;
pub mod launch_requests;
pub mod merchant;
mod operators;
pub mod oversight;
pub mod security;
pub mod support;
pub mod themes_and_config;
mod tiers;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_api_core::{overlay_tier_rows, Tier, TierConfig, TierRow};
use stheno_flows::EraseTenantStoreIntelligenceWorkflow;
use stheno_platform::{
    ApplyInvoiceCreditsInput, NewOperatorAudit, NewTenant, PlatformService,
    StartBillingAttemptInput, SubscriptionDTO, SubscriptionPatch,
};
use stheno_query::query_tenant_store_intelligence_export;
use stheno_types::currency::CurrencyCode;
use stheno_types::{Error, Money};

pub use analytics::*;
pub use launch_requests::{get_launch_request, list_launch_requests, transition_launch_request};
pub use merchant::{merchant_routes, MerchantApiContext};
pub use operators::{
    create_operator, get_audit, list_operators, revoke_operator, CreateOperatorRequest,
};
pub use oversight::*;
pub use security::*;
pub use support::*;
pub use themes_and_config::*;
pub use tiers::{delete_tier, get_tiers, put_tier, TierDefinitionBody, TierView};

#[derive(Clone)]
pub struct PlatformApiContext {
    pub platform: Arc<PlatformService>,
    /// Tier definitions as shipped, from `config/tiers.toml`. Stored overrides are read
    /// from the database per request and laid over this, so a plan edit is live on the
    /// next request on every node without a restart or a cache to invalidate.
    pub tier_config: Arc<TierConfig>,
    /// Shared secret for the control plane, from `STHENO_PLATFORM_TOKEN`.
    ///
    /// A deliberate placeholder. The control plane should have its own actor type
    /// (`platform_owner`) issued through the auth module, which belongs with the RBAC
    /// work in a later phase. Until then a secret keeps the surface closed rather than
    /// open, and an unset secret disables the surface entirely rather than opening it.
    pub operator_token: Option<String>,
    pub payment_service: Option<Arc<stheno_payment::PaymentService>>,
    pub fulfillment_service: Option<Arc<stheno_fulfillment::FulfillmentService>>,
}

/// 404, not 401: the control plane should not confirm it exists to a caller who cannot
/// use it. Same reasoning as the unmounted-route behaviour in ENTITLEMENTS.md 6.1.
pub(crate) fn deny() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "type": "not_found", "message": "Not found" })),
    )
        .into_response()
}

pub(crate) fn bad_request(message: impl std::fmt::Display) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "type": "invalid_data", "message": message.to_string() })),
    )
        .into_response()
}

/// Compares in time independent of how many leading bytes match.
///
/// `==` on `&str` short-circuits, which leaks the length of the correct prefix to a
/// caller who can time the response. That was already true of the tier switch; it matters
/// more now that the same token also defines what every plan includes.
fn secret_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Attributed to the bootstrap token, which identifies a secret rather than a person.
pub const BOOTSTRAP_ACTOR_ID: &str = "op_bootstrap";

/// Who is making this call.
///
/// Carried into every audit row this request writes, which is the entire reason the
/// control plane resolves an identity at all rather than just a yes or no.
#[derive(Debug, Clone)]
pub struct Actor {
    pub id: String,
    pub name: String,
}

impl Actor {
    fn bootstrap() -> Self {
        Self {
            id: BOOTSTRAP_ACTOR_ID.to_string(),
            name: "bootstrap token".to_string(),
        }
    }

    /// Whether this caller is the shared secret rather than a named person.
    pub fn is_bootstrap(&self) -> bool {
        self.id == BOOTSTRAP_ACTOR_ID
    }
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Resolves the presented credential to the operator behind it, or `None`.
///
/// The bootstrap token is checked first: it is a constant-time in-memory comparison, so
/// the common case on a fresh deployment costs no query. A named operator is then looked
/// up by the SHA-256 of the presented token, which is an indexed equality inside Postgres
/// — there is no timing channel on that path because the stored secret never enters this
/// process.
pub(crate) async fn actor(ctx: &PlatformApiContext, headers: &HeaderMap) -> Option<Actor> {
    let presented = bearer(headers)?;

    if let Some(expected) = ctx.operator_token.as_deref() {
        // An empty configured token is treated as unset, never as "matches empty".
        if !expected.is_empty() && secret_eq(presented, expected) {
            return Some(Actor::bootstrap());
        }
    }

    ctx.platform
        .authenticate_operator(presented)
        .await
        .ok()
        .map(|op| Actor {
            id: op.id,
            name: op.name,
        })
}

/// Records one control-plane write against the operator who made it.
///
/// A failure to write the audit row is logged and does not fail the request. The change
/// itself has already committed, so returning an error here would tell the operator their
/// edit did not happen when it did — which is a worse failure than a gap in the trail,
/// and one that invites them to make the change twice.
// ponytail: audit is best-effort and separate from the write it describes; put both in one
// transaction if a missing row ever has to be impossible rather than merely logged.
pub(crate) async fn record(
    ctx: &PlatformApiContext,
    actor: &Actor,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    diff: Option<serde_json::Value>,
) {
    let entry = NewOperatorAudit {
        actor_id: actor.id.clone(),
        actor_name: actor.name.clone(),
        action: action.to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        diff,
    };
    if let Err(e) = ctx.platform.record_operator_action(entry).await {
        tracing::error!(
            action,
            entity_id,
            actor = %actor.id,
            "control-plane action was not audited: {e}"
        );
    }
}

#[derive(Debug, Serialize)]
pub struct TenantSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub status_reason: Option<String>,
    pub status_changed_at: Option<String>,
    pub tier: String,
    pub subscription_status: String,
    pub trial_ends_at: Option<String>,
    pub term_ends_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListTenantsResponse {
    pub tenants: Vec<TenantSummary>,
    pub count: usize,
}

/// `GET /platform/tenants` — every merchant, with the plan each is on.
pub async fn list_tenants(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Query(query): Query<stheno_platform::TenantListQuery>,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }

    let tenants = ctx.platform.list_tenants().await;
    let subscriptions = ctx.platform.list_subscriptions().await;

    let (Ok(tenants), Ok(subscriptions)) = (tenants, subscriptions) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "type": "unexpected_state",
                "message": "could not read the tenant registry"
            })),
        )
            .into_response();
    };

    let mut tenants: Vec<TenantSummary> = tenants
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
        .map(|tenant| {
            let sub = subscriptions.iter().find(|s| s.tenant_id == tenant.id);
            TenantSummary {
                id: tenant.id,
                name: tenant.name,
                status: tenant.status,
                status_reason: tenant.status_reason,
                status_changed_at: tenant.status_changed_at.map(|d| d.to_rfc3339()),
                tier: sub
                    .map(|s| s.tier.clone())
                    .unwrap_or_else(|| "unknown".into()),
                subscription_status: sub
                    .map(|s| s.status.clone())
                    .unwrap_or_else(|| "missing".into()),
                trial_ends_at: sub.and_then(|s| s.trial_ends_at).map(|d| d.to_rfc3339()),
                term_ends_at: sub.and_then(|s| s.term_ends_at).map(|d| d.to_rfc3339()),
                created_at: tenant.created_at.to_rfc3339(),
            }
        })
        .filter(|t| {
            if let Some(tier) = &query.tier {
                if &t.tier != tier {
                    return false;
                }
            }
            true
        })
        .collect();

    tenants.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let count = tenants.len();
    Json(ListTenantsResponse { tenants, count }).into_response()
}

#[derive(Debug, Deserialize)]
pub struct SetTierRequest {
    pub tier: String,
}

/// `POST /platform/tenants/{id}/tier` — move a merchant between plans.
pub async fn set_tier(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<SetTierRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let config = effective_tier_config(&ctx).await;
    let known = body
        .tier
        .parse::<Tier>()
        .ok()
        .is_some_and(|tier| config.definition(tier).is_ok());

    if !known {
        return bad_request(format!("unknown tier {:?}", body.tier));
    }

    // Read the plan the merchant is leaving before the write, so the trail records the
    // move rather than only its destination. "Moved to free" is not an answer to "who
    // took them off Pro".
    let previous = ctx
        .platform
        .get_subscription(&id)
        .await
        .map(|s| s.tier)
        .ok();

    match ctx.platform.set_tier(&id, &body.tier).await {
        Ok(sub) => {
            record(
                &ctx,
                &actor,
                "tenant.tier_set",
                "tenant",
                &id,
                Some(json!({ "from": previous, "to": sub.tier })),
            )
            .await;
            Json(json!({
                "subscription": {
                    "tenant_id": sub.tenant_id,
                    "tier": sub.tier,
                    "status": sub.status,
                }
            }))
            .into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": format!("no tenant {id}") })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct TenantLifecycleRequest {
    pub reason: String,
}

/// `POST /platform/tenants/{id}/suspend` — take a merchant offline.
pub async fn suspend_tenant(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<TenantLifecycleRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };
    let previous = ctx.platform.get_tenant(&id).await.map(|t| t.status).ok();
    match ctx
        .platform
        .set_tenant_status(&id, "suspended", &body.reason)
        .await
    {
        Ok(tenant) => {
            record(
                &ctx,
                &actor,
                "tenant.suspend",
                "tenant",
                &id,
                Some(json!({
                    "from": previous,
                    "to": tenant.status,
                    "reason": tenant.status_reason,
                })),
            )
            .await;
            Json(json!({ "tenant": tenant })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

/// `POST /platform/tenants/{id}/reactivate` — restore a suspended merchant.
pub async fn reactivate_tenant(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<TenantLifecycleRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };
    let previous = ctx.platform.get_tenant(&id).await.map(|t| t.status).ok();
    match ctx
        .platform
        .set_tenant_status(&id, "active", &body.reason)
        .await
    {
        Ok(tenant) => {
            record(
                &ctx,
                &actor,
                "tenant.reactivate",
                "tenant",
                &id,
                Some(json!({
                    "from": previous,
                    "to": tenant.status,
                    "reason": tenant.status_reason,
                })),
            )
            .await;
            Json(json!({ "tenant": tenant })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct CloseTenantRequest {
    pub confirm_id: String,
    pub reason: String,
}

/// `POST /platform/tenants/{id}/close` — permanently close a tenant.
/// Requires typing the tenant id as confirmation.
pub async fn close_tenant(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<CloseTenantRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    if body.confirm_id != id {
        return bad_request(format!(
            "Tenant id confirmation mismatch: expected '{id}', got '{}'",
            body.confirm_id
        ));
    }

    if body.reason.trim().is_empty() {
        return bad_request("A reason is required when closing a tenant");
    }

    let previous = ctx.platform.get_tenant(&id).await.map(|t| t.status).ok();
    match ctx
        .platform
        .set_tenant_status(&id, "closed", &body.reason)
        .await
    {
        Ok(tenant) => {
            record(
                &ctx,
                &actor,
                "tenant.close",
                "tenant",
                &id,
                Some(json!({
                    "from": previous,
                    "to": tenant.status,
                    "reason": tenant.status_reason,
                })),
            )
            .await;
            Json(json!({ "tenant": tenant })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddDomainRequest {
    pub domain: String,
    pub is_primary: Option<bool>,
}

pub async fn list_tenant_domains(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.list_tenant_domains(&id).await {
        Ok(domains) => {
            let count = domains.len();
            Json(json!({ "domains": domains, "count": count })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

pub async fn add_tenant_domain(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AddDomainRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx
        .platform
        .add_tenant_domain(&id, &body.domain, body.is_primary.unwrap_or(false))
        .await
    {
        Ok(domain) => {
            record(
                &ctx,
                &actor,
                "tenant.domain.add",
                "tenant",
                &id,
                Some(json!({ "domain": domain.domain, "is_primary": domain.is_primary })),
            )
            .await;
            (StatusCode::CREATED, Json(json!({ "domain": domain }))).into_response()
        }
        Err(e) => platform_error(e),
    }
}

pub async fn remove_tenant_domain(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path((id, domain)): Path<(String, String)>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.remove_tenant_domain(&id, &domain).await {
        Ok(()) => {
            record(
                &ctx,
                &actor,
                "tenant.domain.remove",
                "tenant",
                &id,
                Some(json!({ "domain": domain })),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => platform_error(e),
    }
}

pub async fn set_primary_domain(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path((id, domain)): Path<(String, String)>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.set_primary_domain(&id, &domain).await {
        Ok(d) => {
            record(
                &ctx,
                &actor,
                "tenant.domain.primary",
                "tenant",
                &id,
                Some(json!({ "domain": domain })),
            )
            .await;
            Json(json!({ "domain": d })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

pub async fn get_domain_verification(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path((_id, domain)): Path<(String, String)>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_domain_verification(&domain).await {
        Ok(v) => Json(json!({ "verification": v })).into_response(),
        Err(e) => platform_error(e),
    }
}

pub async fn get_tenant_health(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_merchant_health(&id).await {
        Ok(health) => Json(json!({ "health": health })).into_response(),
        Err(e) => platform_error(e),
    }
}

pub async fn get_onboarding_state(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_onboarding_state(&id).await {
        Ok(state) => Json(json!({ "onboarding": state })).into_response(),
        Err(e) => platform_error(e),
    }
}

pub async fn export_tenant_data(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let bundle = match ctx.platform.export_tenant_data(&id).await {
        Ok(bundle) => bundle,
        Err(error) => return platform_error(error),
    };
    let pool = ctx.platform.db_pool();
    let source_data = query_tenant_store_intelligence_export(&pool, &id).await;
    let source_data = match source_data {
        Ok(data) => data,
        Err(error) => return platform_error(error),
    };
    let mut export = match serde_json::to_value(bundle) {
        Ok(value) => value,
        Err(error) => {
            return platform_error(Error::Database(format!(
                "could not serialize tenant export: {error}"
            )));
        }
    };
    let Some(fields) = export.as_object_mut() else {
        return platform_error(Error::Database(
            "tenant export did not serialize as an object".to_string(),
        ));
    };
    let source_data = match serde_json::to_value(source_data) {
        Ok(value) => value,
        Err(error) => {
            return platform_error(Error::Database(format!(
                "could not serialize Store Intelligence export: {error}"
            )));
        }
    };
    fields.insert("store_intelligence".to_string(), source_data);
    record(&ctx, &actor, "tenant.export", "tenant", &id, None).await;
    Json(json!({ "export": export })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct PurgeTenantsRequest {
    pub retention_days: i64,
}

pub async fn purge_closed_tenants(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<PurgeTenantsRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let pool = ctx.platform.db_pool();
    let erasure = EraseTenantStoreIntelligenceWorkflow::new(pool);
    let result = ctx
        .platform
        .purge_closed_tenants_with(body.retention_days, move |tenant_id| {
            let erasure = erasure.clone();
            Box::pin(async move { erasure.execute(&tenant_id).await })
                as Pin<Box<dyn Future<Output = stheno_types::Result<()>> + Send>>
        })
        .await;
    match result {
        Ok(result) => {
            record(
                &ctx,
                &actor,
                "tenant.purge",
                "platform",
                "platform",
                Some(json!({
                    "retention_days": body.retention_days,
                    "purged_count": result.purged_count,
                    "purged_tenant_ids": result.purged_tenant_ids,
                })),
            )
            .await;
            Json(json!({ "purge": result })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

/// Statuses `platform.subscription`'s CHECK constraint accepts. Validated here so a bad
/// value comes back as a 400 naming the allowed set, not a 500 from Postgres.
const SUBSCRIPTION_STATUSES: [&str; 5] = ["active", "past_due", "grace", "suspended", "cancelled"];

/// Absent means "leave it alone"; an explicit `null` means "clear it".
///
/// `Option<Option<T>>` alone cannot tell those apart — serde maps a JSON `null` to the
/// outer `None`, the same as an absent field. This forces the outer `Some`, so only a
/// genuinely absent key stays `None`.
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, Default)]
pub struct PatchSubscriptionRequest {
    pub tier: Option<String>,
    pub status: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub trial_started_at: Option<Option<chrono::DateTime<chrono::Utc>>>,
    #[serde(default, deserialize_with = "present")]
    pub trial_ends_at: Option<Option<chrono::DateTime<chrono::Utc>>>,
    #[serde(default, deserialize_with = "present")]
    pub term_ends_at: Option<Option<chrono::DateTime<chrono::Utc>>>,
    /// `{"max_products": 250}` — a negotiated exception for this merchant alone.
    pub quota_overrides: Option<serde_json::Value>,
    pub window_overrides: Option<serde_json::Value>,
    /// `{"payment_gateway": true}` — grants; an explicit `false` revokes.
    pub feature_overrides: Option<serde_json::Value>,
}

fn subscription_json(sub: &SubscriptionDTO) -> serde_json::Value {
    json!({
        "tenant_id": sub.tenant_id,
        "tier": sub.tier,
        "status": sub.status,
        "trial_started_at": sub.trial_started_at.map(|d| d.to_rfc3339()),
        "trial_ends_at": sub.trial_ends_at.map(|d| d.to_rfc3339()),
        "term_ends_at": sub.term_ends_at.map(|d| d.to_rfc3339()),
        "quota_overrides": sub.quota_overrides,
        "window_overrides": sub.window_overrides,
        "feature_overrides": sub.feature_overrides,
    })
}

/// The plan and the overrides resolved into what the merchant actually gets.
///
/// Without this the operator is editing two JSON blobs and a tier name and guessing at
/// the product of them. It is the same `resolve_for_tenant` the request path uses, so
/// what is shown here is what the router will mount.
fn effective_json(config: &TierConfig, sub: &SubscriptionDTO) -> Result<serde_json::Value, String> {
    let plan = stheno_api_core::TenantPlan {
        tier: sub.tier.clone(),
        trial_ends_at: sub.trial_ends_at,
        term_ends_at: sub.term_ends_at,
        quota_overrides: sub.quota_overrides.clone(),
        window_overrides: sub.window_overrides.clone(),
        feature_overrides: sub.feature_overrides.clone(),
    };
    let resolved = stheno_api_core::resolve_for_tenant(config, &plan).map_err(|e| e.to_string())?;
    Ok(tiers::entitlements_json(&resolved))
}

/// `GET /platform/tenants/{id}/subscription` — the stored row plus its effect.
pub async fn get_subscription(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }

    let Ok(sub) = ctx.platform.get_subscription(&id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": format!("no tenant {id}") })),
        )
            .into_response();
    };

    let config = effective_tier_config(&ctx).await;
    // A subscription that cannot resolve is exactly what the operator needs to see, so
    // the row is still returned and the failure is reported next to it rather than
    // turning the whole page into an error.
    let (effective, error) = match effective_json(&config, &sub) {
        Ok(value) => (Some(value), None),
        Err(message) => (None, Some(message)),
    };

    Json(json!({
        "subscription": subscription_json(&sub),
        "effective": effective,
        "error": error,
    }))
    .into_response()
}

/// `PATCH /platform/tenants/{id}/subscription` — every lever on one merchant's plan.
pub async fn patch_subscription(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<PatchSubscriptionRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let config = effective_tier_config(&ctx).await;

    if let Some(tier) = &body.tier {
        let known = tier
            .parse::<Tier>()
            .ok()
            .is_some_and(|t| config.definition(t).is_ok());
        if !known {
            return bad_request(format!("unknown tier {tier:?}"));
        }
    }

    if let Some(status) = &body.status {
        if !SUBSCRIPTION_STATUSES.contains(&status.as_str()) {
            return bad_request(format!(
                "unknown status {status:?}; expected one of {}",
                SUBSCRIPTION_STATUSES.join(", ")
            ));
        }
    }

    // Overrides are validated before they are stored, not at the next boot. A typo that
    // reaches the table would make every subsequent request for this merchant fall back
    // to the least-privileged entitlements, which looks like an outage, not a typo.
    if let Some(v) = &body.quota_overrides {
        if let Err(e) = tiers::validate_limits::<stheno_api_core::QuotaKey>(v) {
            return bad_request(e);
        }
    }
    if let Some(v) = &body.window_overrides {
        if let Err(e) = tiers::validate_limits::<stheno_api_core::WindowKey>(v) {
            return bad_request(e);
        }
    }
    if let Some(v) = &body.feature_overrides {
        if let Err(e) = tiers::validate_feature_flags(v) {
            return bad_request(e);
        }
    }

    let status_override = body.status.clone();
    let patch = SubscriptionPatch {
        tier: body.tier,
        status: None,
        trial_started_at: body.trial_started_at,
        trial_ends_at: body.trial_ends_at,
        term_ends_at: body.term_ends_at,
        quota_overrides: body.quota_overrides,
        window_overrides: body.window_overrides,
        feature_overrides: body.feature_overrides,
    };

    // The row as it was, so the trail records what changed rather than only where it
    // landed. Every field here is billing-affecting.
    let before = ctx.platform.get_subscription(&id).await.ok();

    match ctx.platform.update_subscription(&id, patch).await {
        Ok(mut sub) => {
            if let Some(status) = status_override {
                match ctx
                    .platform
                    .override_subscription_status(&id, &status)
                    .await
                {
                    Ok(overridden) => sub = overridden,
                    Err(e) => return bad_request(e),
                }
            }
            record(
                &ctx,
                &actor,
                "subscription.patch",
                "tenant",
                &id,
                Some(json!({
                    "before": before.as_ref().map(subscription_json),
                    "after": subscription_json(&sub),
                })),
            )
            .await;

            let effective = effective_json(&config, &sub).ok();
            Json(json!({
                "subscription": subscription_json(&sub),
                "effective": effective,
            }))
            .into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": format!("no tenant {id}") })),
        )
            .into_response(),
    }
}

fn platform_error(e: Error) -> Response {
    match e {
        Error::NotFound { .. } => (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": e.to_string() })),
        )
            .into_response(),
        Error::Invalid { .. } | Error::InvalidCurrency(_) | Error::CurrencyMismatch { .. } => {
            bad_request(e)
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "type": "unexpected_state", "message": e.to_string() })),
        )
            .into_response(),
    }
}

/// `GET /platform/invoices` — every platform subscription invoice.
pub async fn list_invoices(State(ctx): State<PlatformApiContext>, headers: HeaderMap) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }

    match ctx.platform.list_invoices().await {
        Ok(invoices) => {
            Json(json!({ "invoices": invoices, "count": invoices.len() })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

/// `GET /platform/tenants/{id}/invoices` — one merchant's platform bills.
pub async fn list_tenant_invoices(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }

    match ctx.platform.list_invoices_for_tenant(&id).await {
        Ok(invoices) => {
            Json(json!({ "invoices": invoices, "count": invoices.len() })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

/// `GET /platform/billing-attempts` — collection attempts across invoices.
pub async fn list_billing_attempts(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }

    match ctx.platform.list_billing_attempts().await {
        Ok(attempts) => {
            Json(json!({ "attempts": attempts, "count": attempts.len() })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct RetryInvoiceRequest {
    pub provider_id: String,
    pub idempotency_key: String,
    pub request_hash: String,
}

/// `POST /platform/invoices/{id}/retry` — records a manual collection attempt.
pub async fn retry_invoice(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<RetryInvoiceRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let invoice = match ctx.platform.get_invoice(&id).await {
        Ok(invoice) => invoice,
        Err(e) => return platform_error(e),
    };

    match ctx
        .platform
        .start_billing_attempt(StartBillingAttemptInput {
            invoice_id: id.clone(),
            tenant_id: invoice.tenant_id.clone(),
            provider_id: body.provider_id.clone(),
            idempotency_key: body.idempotency_key.clone(),
            request_hash: body.request_hash.clone(),
            amount: invoice.total,
        })
        .await
    {
        Ok(attempt) => {
            record(
                &ctx,
                &actor,
                "invoice.retry",
                "invoice",
                &id,
                Some(json!({
                    "provider_id": body.provider_id,
                    "idempotency_key": body.idempotency_key,
                    "attempt_id": attempt.id,
                })),
            )
            .await;
            Json(json!({ "attempt": attempt })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreditInvoiceRequest {
    pub credit_minor: i64,
    pub discount_minor: i64,
    pub coupon_minor: i64,
    pub currency: String,
    pub reason: String,
}

/// `POST /platform/invoices/{id}/credit` — applies an invoice-level credit/discount.
pub async fn credit_invoice(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<CreditInvoiceRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let currency: CurrencyCode = match body.currency.parse() {
        Ok(currency) => currency,
        Err(e) => return platform_error(e),
    };

    match ctx
        .platform
        .apply_invoice_credits(ApplyInvoiceCreditsInput {
            invoice_id: id.clone(),
            credit: Money::from_minor_units(body.credit_minor, currency),
            discount: Money::from_minor_units(body.discount_minor, currency),
            coupon: Money::from_minor_units(body.coupon_minor, currency),
            reason: body.reason.clone(),
        })
        .await
    {
        Ok(invoice) => {
            record(
                &ctx,
                &actor,
                "invoice.credit",
                "invoice",
                &id,
                Some(json!({
                    "credit_minor": body.credit_minor,
                    "discount_minor": body.discount_minor,
                    "coupon_minor": body.coupon_minor,
                    "currency": body.currency,
                    "reason": body.reason,
                })),
            )
            .await;
            Json(json!({ "invoice": invoice })).into_response()
        }
        Err(e) => platform_error(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub domain: Option<String>,
}

/// `POST /platform/tenants` — provision a merchant.
pub async fn create_tenant(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<CreateTenantRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    // Provisioning onto a tier that does not exist would leave a merchant whose every
    // request falls back to the least-privileged entitlements — a silent dead store.
    let config = effective_tier_config(&ctx).await;
    let known = body
        .tier
        .parse::<Tier>()
        .ok()
        .is_some_and(|tier| config.definition(tier).is_ok());
    if !known {
        return bad_request(format!("unknown tier {:?}", body.tier));
    }

    let tier = body.tier.clone();
    match ctx
        .platform
        .create_tenant(NewTenant {
            id: body.id,
            name: body.name,
            tier: body.tier,
            domain: body.domain,
        })
        .await
    {
        Ok(tenant) => {
            record(
                &ctx,
                &actor,
                "tenant.create",
                "tenant",
                &tenant.id,
                Some(json!({ "name": tenant.name, "tier": tier })),
            )
            .await;
            (
                StatusCode::CREATED,
                Json(json!({ "tenant": { "id": tenant.id, "name": tenant.name } })),
            )
                .into_response()
        }
        Err(e) => bad_request(e),
    }
}

/// Reads the stored plan definitions and lays them over the shipped TOML.
///
/// Called per request rather than cached in the process. That costs one small query, and
/// it buys a plan edit that is live on every node immediately with no invalidation to get
/// wrong — which matters because a stale node would be serving a plan nobody is on.
// ponytail: one extra query per control-plane request and one per merchant request; add a
// short TTL cache in TenantLayerState if it ever shows up in a profile.
pub async fn effective_tier_config(ctx: &PlatformApiContext) -> TierConfig {
    tier_config_for(&ctx.platform, &ctx.tier_config).await
}

/// The same overlay, for callers that hold the service directly (the tenant layer).
pub async fn tier_config_for(platform: &PlatformService, base: &TierConfig) -> TierConfig {
    match platform.list_tier_definitions().await {
        Ok(rows) => {
            let rows: Vec<TierRow> = rows
                .into_iter()
                .map(|r| TierRow {
                    tier: r.tier,
                    features: r.features,
                    quotas: r.quotas,
                    windows: r.windows,
                })
                .collect();
            overlay_tier_rows(base, &rows)
        }
        // A registry that cannot be read must not silently redefine every plan. The
        // shipped defaults are the safe answer: they are what the deployment shipped with.
        Err(e) => {
            tracing::warn!("tier definitions unreadable, using shipped defaults: {e}");
            base.clone()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpsertPlatformCredentialRequest {
    pub credentials: std::collections::BTreeMap<String, serde_json::Value>,
}

pub async fn list_platform_payment_providers(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let payment_service = match &ctx.payment_service {
        Some(svc) => svc,
        None => {
            return Json(json!({
                "payment_providers": [],
                "count": 0
            }))
            .into_response();
        }
    };

    let mut providers = Vec::new();
    for factory in payment_service.list_factories() {
        let credential = payment_service
            .get_provider_credential("platform", factory.provider_id())
            .await
            .unwrap_or_default();
        let configured = credential.is_some();
        let credential_health = credential
            .as_ref()
            .map(|c| c.health)
            .unwrap_or(stheno_payment::CredentialHealth::Unconfigured);

        let provider_id = factory.provider_id();
        let name = provider_id
            .trim_start_matches("pp_")
            .replace('_', " ")
            .split_whitespace()
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        providers.push(json!({
            "id": provider_id,
            "name": name,
            "configured": configured,
            "credential_health": credential_health,
            "credential_schema": factory.credential_schema(),
            "credential": credential,
        }));
    }

    let count = providers.len();
    Json(json!({
        "payment_providers": providers,
        "count": count
    }))
    .into_response()
}

pub async fn upsert_platform_payment_provider_credentials(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(provider_id): Path<String>,
    Json(body): Json<UpsertPlatformCredentialRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let payment_service = match &ctx.payment_service {
        Some(svc) => svc,
        None => return bad_request("Payment service is not initialized on platform"),
    };

    match payment_service
        .upsert_provider_credential("platform", &provider_id, &body.credentials)
        .await
    {
        Ok(credential) => {
            record(
                &ctx,
                &actor,
                "platform.payment_provider.update",
                "payment_provider",
                &provider_id,
                Some(json!({
                    "provider_id": provider_id,
                    "health": credential.health,
                })),
            )
            .await;

            let factory = match payment_service.get_factory(&provider_id) {
                Ok(f) => f,
                Err(e) => return platform_error(e),
            };

            let name = provider_id
                .trim_start_matches("pp_")
                .replace('_', " ")
                .split_whitespace()
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            Json(json!({
                "payment_provider": {
                    "id": provider_id,
                    "name": name,
                    "configured": true,
                    "credential_health": credential.health,
                    "credential_schema": factory.credential_schema(),
                    "credential": credential,
                }
            }))
            .into_response()
        }
        Err(e) => platform_error(e),
    }
}

pub fn platform_routes(ctx: PlatformApiContext) -> Router {
    Router::new()
        .route("/platform/tenants", get(list_tenants).post(create_tenant))
        .route("/platform/launch-requests", get(list_launch_requests))
        .route("/platform/launch-requests/{id}", get(get_launch_request))
        .route(
            "/platform/launch-requests/{id}/{action}",
            post(transition_launch_request),
        )
        .route("/platform/tenants/purge", post(purge_closed_tenants))
        .route("/platform/tenants/{id}/tier", post(set_tier))
        .route("/platform/tenants/{id}/suspend", post(suspend_tenant))
        .route("/platform/tenants/{id}/reactivate", post(reactivate_tenant))
        .route("/platform/tenants/{id}/close", post(close_tenant))
        .route(
            "/platform/tenants/{id}/domains",
            get(list_tenant_domains).post(add_tenant_domain),
        )
        .route(
            "/platform/tenants/{id}/domains/{domain}",
            delete(remove_tenant_domain),
        )
        .route(
            "/platform/tenants/{id}/domains/{domain}/primary",
            post(set_primary_domain),
        )
        .route(
            "/platform/tenants/{id}/domains/{domain}/verification",
            get(get_domain_verification),
        )
        .route("/platform/tenants/{id}/health", get(get_tenant_health))
        .route(
            "/platform/tenants/{id}/onboarding",
            get(get_onboarding_state),
        )
        .route("/platform/tenants/{id}/export", get(export_tenant_data))
        .route(
            "/platform/tenants/{id}/subscription",
            get(get_subscription).patch(patch_subscription),
        )
        .route("/platform/invoices", get(list_invoices))
        .route("/platform/tenants/{id}/invoices", get(list_tenant_invoices))
        .route("/platform/billing-attempts", get(list_billing_attempts))
        .route("/platform/invoices/{id}/retry", post(retry_invoice))
        .route("/platform/invoices/{id}/credit", post(credit_invoice))
        .route(
            "/platform/payment-providers",
            get(list_platform_payment_providers),
        )
        .route(
            "/platform/payment-providers/{id}/credentials",
            post(upsert_platform_payment_provider_credentials),
        )
        .route("/platform/tiers", get(get_tiers))
        .route("/platform/tiers/{tier}", put(put_tier).delete(delete_tier))
        .route(
            "/platform/operators",
            get(list_operators).post(create_operator),
        )
        .route("/platform/operators/{id}", delete(revoke_operator))
        .route("/platform/audit", get(get_audit))
        // -- Oversight (P32) ---------------------------------------------------------
        .route("/platform/providers", get(list_providers_oversight))
        .route("/platform/providers/{id}/health", get(get_provider_health))
        .route("/platform/ledger/outbound", get(list_outbound_ledger))
        .route("/platform/ledger/sweep-status", get(get_sweep_status))
        .route("/platform/ledger/sweep", post(trigger_reconciliation_sweep))
        .route("/platform/webhooks/health", get(list_webhook_oversight))
        .route(
            "/platform/webhooks/replay",
            post(replay_dead_letter_webhook),
        )
        .route(
            "/platform/gateways/{id}/kill-switch",
            post(toggle_gateway_kill_switch),
        )
        .route(
            "/platform/settlements",
            get(list_settlement_reconciliations),
        )
        .route("/platform/settlements/cod", get(list_cod_settlements))
        .route("/platform/couriers", get(list_couriers_oversight))
        .route(
            "/platform/consignments/stranded",
            get(list_stranded_consignments),
        )
        .route(
            "/platform/alarms/unknown-status",
            get(list_unknown_status_alarms),
        )
        // -- Growth Analytics (P33) ---------------------------------------------------
        .route("/platform/analytics/gmv", get(get_gmv_metrics))
        .route("/platform/analytics/funnel", get(get_funnel_metrics))
        .route(
            "/platform/analytics/plan-distribution",
            get(get_plan_distribution),
        )
        .route("/platform/analytics/top-merchants", get(get_top_merchants))
        .route("/platform/analytics/churn-cohorts", get(get_churn_cohorts))
        .route(
            "/platform/analytics/quota-headroom",
            get(get_quota_headroom),
        )
        // -- Support Operations (P34) --------------------------------------------------
        .route(
            "/platform/tenants/{id}/impersonate",
            post(start_impersonation),
        )
        .route(
            "/platform/tenants/{id}/impersonate/end",
            post(end_impersonation),
        )
        .route("/platform/tenants/{id}/errors", get(get_tenant_errors))
        .route("/platform/jobs/trigger", post(trigger_manual_job))
        .route("/platform/workflows", get(list_platform_workflows))
        .route(
            "/platform/tenants/{id}/read-only",
            post(toggle_tenant_read_only),
        )
        // -- Configuration & Themes (P35) ----------------------------------------------
        .route("/platform/themes", get(list_theme_catalog))
        .route("/platform/tenants/{id}/theme", post(assign_tenant_theme))
        .route("/platform/themes/{id}/rollout", post(rollout_theme_version))
        .route("/platform/feature-flags", get(list_feature_flags))
        .route("/platform/feature-flags/{id}", put(update_feature_flag))
        .route(
            "/platform/config/defaults",
            get(get_platform_defaults).put(update_platform_defaults),
        )
        .route(
            "/platform/config/messaging",
            get(get_messaging_config).put(update_messaging_config),
        )
        .route(
            "/platform/announcements",
            get(list_announcements).post(create_announcement),
        )
        // -- Hardening & Access Control (P36) -------------------------------------------
        .route(
            "/platform/operators/{id}/role",
            get(get_operator_role).put(set_operator_role),
        )
        .route(
            "/platform/operators/{id}/mfa/setup",
            post(setup_operator_mfa),
        )
        .route(
            "/platform/operators/{id}/mfa/verify",
            post(verify_operator_mfa),
        )
        .route(
            "/platform/operators/{id}/network-limits",
            post(set_operator_network_limits),
        )
        .route(
            "/platform/operators/bootstrap-retirement",
            get(get_bootstrap_retirement_status),
        )
        .route(
            "/platform/compliance/evidence",
            get(get_compliance_evidence),
        )
        .with_state(ctx)
}
