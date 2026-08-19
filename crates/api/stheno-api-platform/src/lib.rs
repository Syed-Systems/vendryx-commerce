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

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_api_core::{Tier, TierConfig};
use stheno_platform::{NewTenant, PlatformService};

#[derive(Clone)]
pub struct PlatformApiContext {
    pub platform: Arc<PlatformService>,
    /// Tier definitions, so a tier change is rejected at the boundary rather than stored
    /// and silently falling back to defaults at resolution time.
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
fn deny() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "type": "not_found", "message": "Not found" })),
    )
        .into_response()
}

fn authorised(ctx: &PlatformApiContext, headers: &HeaderMap) -> bool {
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
        .is_some_and(|token| token == expected)
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

    let known = body
        .tier
        .parse::<Tier>()
        .ok()
        .is_some_and(|tier| ctx.tier_config.definition(tier).is_ok());

    if !known {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "type": "invalid_data",
                "message": format!("unknown tier {:?}", body.tier)
            })),
        )
            .into_response();
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
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "type": "invalid_data", "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub fn platform_routes(ctx: PlatformApiContext) -> Router {
    Router::new()
        .route("/platform/tenants", get(list_tenants).post(create_tenant))
        .route("/platform/tenants/{id}/tier", post(set_tier))
        .with_state(ctx)
}
