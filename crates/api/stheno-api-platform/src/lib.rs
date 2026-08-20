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

mod tiers;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_api_core::{overlay_tier_rows, Tier, TierConfig, TierRow};
use stheno_platform::{NewTenant, PlatformService, SubscriptionDTO, SubscriptionPatch};

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

pub(crate) fn authorised(ctx: &PlatformApiContext, headers: &HeaderMap) -> bool {
    let Some(expected) = ctx.operator_token.as_deref() else {
        // No token configured means the control plane is switched off, never open.
        return false;
    };
    if expected.is_empty() {
        return false;
    }
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|token| secret_eq(token, expected))
}

#[derive(Debug, Serialize)]
pub struct TenantSummary {
    pub id: String,
    pub name: String,
    pub status: String,
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
pub async fn list_tenants(State(ctx): State<PlatformApiContext>, headers: HeaderMap) -> Response {
    if !authorised(&ctx, &headers) {
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

    let tenants: Vec<TenantSummary> = tenants
        .into_iter()
        .map(|tenant| {
            let sub = subscriptions.iter().find(|s| s.tenant_id == tenant.id);
            TenantSummary {
                id: tenant.id,
                name: tenant.name,
                status: tenant.status,
                // A tenant with no subscription row is a provisioning bug, not a free
                // customer. Reported as `unknown` rather than defaulted to a tier, so it
                // shows up on the dashboard instead of looking like a normal account.
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
        .collect();

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
    if !authorised(&ctx, &headers) {
        return deny();
    }

    let config = effective_tier_config(&ctx).await;
    let known = body
        .tier
        .parse::<Tier>()
        .ok()
        .is_some_and(|tier| config.definition(tier).is_ok());

    if !known {
        return bad_request(format!("unknown tier {:?}", body.tier));
    }

    match ctx.platform.set_tier(&id, &body.tier).await {
        Ok(sub) => Json(json!({
            "subscription": {
                "tenant_id": sub.tenant_id,
                "tier": sub.tier,
                "status": sub.status,
            }
        }))
        .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": format!("no tenant {id}") })),
        )
            .into_response(),
    }
}

/// Statuses `platform.subscription`'s CHECK constraint accepts. Validated here so a bad
/// value comes back as a 400 naming the allowed set, not a 500 from Postgres.
const SUBSCRIPTION_STATUSES: [&str; 3] = ["active", "grace", "cancelled"];

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
fn effective_json(
    config: &TierConfig,
    sub: &SubscriptionDTO,
) -> Result<serde_json::Value, String> {
    let plan = stheno_api_core::TenantPlan {
        tier: sub.tier.clone(),
        trial_ends_at: sub.trial_ends_at,
        term_ends_at: sub.term_ends_at,
        quota_overrides: sub.quota_overrides.clone(),
        window_overrides: sub.window_overrides.clone(),
        feature_overrides: sub.feature_overrides.clone(),
    };
    let resolved =
        stheno_api_core::resolve_for_tenant(config, &plan).map_err(|e| e.to_string())?;
    Ok(tiers::entitlements_json(&resolved))
}

/// `GET /platform/tenants/{id}/subscription` — the stored row plus its effect.
pub async fn get_subscription(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if !authorised(&ctx, &headers) {
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
    if !authorised(&ctx, &headers) {
        return deny();
    }

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

    let patch = SubscriptionPatch {
        tier: body.tier,
        status: body.status,
        trial_started_at: body.trial_started_at,
        trial_ends_at: body.trial_ends_at,
        term_ends_at: body.term_ends_at,
        quota_overrides: body.quota_overrides,
        window_overrides: body.window_overrides,
        feature_overrides: body.feature_overrides,
    };

    match ctx.platform.update_subscription(&id, patch).await {
        Ok(sub) => {
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
    if !authorised(&ctx, &headers) {
        return deny();
    }

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
        Ok(tenant) => (
            StatusCode::CREATED,
            Json(json!({ "tenant": { "id": tenant.id, "name": tenant.name } })),
        )
            .into_response(),
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

pub fn platform_routes(ctx: PlatformApiContext) -> Router {
    Router::new()
        .route("/platform/tenants", get(list_tenants).post(create_tenant))
        .route("/platform/tenants/{id}/tier", post(set_tier))
        .route(
            "/platform/tenants/{id}/subscription",
            get(get_subscription).patch(patch_subscription),
        )
        .route("/platform/tiers", get(get_tiers))
        .route("/platform/tiers/{tier}", put(put_tier).delete(delete_tier))
        .with_state(ctx)
}
