//! Resolves which tenant a request belongs to, and scopes the request to it (ADR-012).
//!
//! # Why the Host header is primary
//!
//! `/store/*` is unauthenticated — a shopper has no token — so a JWT cannot be the source
//! of tenant identity for the storefront. The Host header is the only signal present on
//! every request, authenticated or not. A staff JWT can *cross-check* the answer
//! (see `auth_guard`), but it can never be the source, or the storefront has no answer at
//! all.
//!
//! # The security precondition
//!
//! `Host` is client-supplied. This is only safe behind a proxy that overwrites it, which
//! is how the deployment in `deploy/` is configured. Without that, tenancy is decorative:
//! anyone can claim any tenant by setting a header. That is stated here rather than
//! assumed, because it is the single assumption the whole scheme rests on.
//!
//! # Unknown hosts 404
//!
//! There is deliberately no "fall back to the default tenant" branch. Every leak starts
//! as a convenience branch, and this is exactly where one would go.

use axum::extract::{Request, State};
use axum::http::{header::HOST, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::tenant_entitlements::{resolve_for_tenant, TenantPlan};
use stheno_api_core::{Entitlements, TierConfig};
use stheno_db::tenant;
use stheno_platform::PlatformService;

/// Paths answered without a tenant, because they exist to report on the process itself.
const TENANT_FREE_PATHS: [&str; 2] = ["/health", "/"];

#[derive(Clone)]
pub struct TenantLayerState {
    pub platform: Arc<PlatformService>,
    /// Tier *definitions*, loaded once at boot from `config/tiers.toml`. What a tier
    /// means is deployment config; which tier a tenant is on is a subscription row.
    pub tier_config: Arc<TierConfig>,
    /// Used when no subscription can be read, so a registry hiccup degrades to the
    /// least-privileged plan rather than to whatever was configured at boot.
    pub fallback_entitlements: Arc<Entitlements>,
    /// Used when the Host header resolves to nothing and the deployment is single-tenant.
    ///
    /// This is what keeps ADR-009-style self-hosted installs working: one merchant, no
    /// domain registry, `STHENO_TENANT` naming the only tenant there is. It is `None` for
    /// a hosted deployment, where an unknown host must 404.
    pub fallback_tenant: Option<String>,
}

fn not_found() -> Response {
    // The same shape an unmounted route produces, so probing for valid hosts cannot
    // distinguish "no such tenant" from "no such route".
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "type": "not_found", "message": "Not found" })),
    )
        .into_response()
}

/// Establishes the tenant scope for the whole downstream request.
pub async fn resolve_tenant(
    State(state): State<TenantLayerState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if TENANT_FREE_PATHS.contains(&path) {
        return next.run(request).await;
    }

    // The port is not part of tenant identity: `shop.example.com` and
    // `shop.example.com:8443` are the same merchant.
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(':')
                .next()
                .unwrap_or(value)
                .to_ascii_lowercase()
        });

    let tenant_id = match host {
        Some(host) => match state.platform.tenant_for_domain(&host).await {
            Ok(id) => Some(id),
            Err(e) => {
                // An unregistered host is routine and stays quiet. A registry that cannot
                // be *read* is not: it 404s every request on the deployment, and swallowing
                // the reason turns a broken database grant into "the site is down" with
                // nothing to go on.
                if !matches!(e, stheno_types::Error::NotFound { .. }) {
                    eprintln!("tenant registry unreadable for host {host:?}: {e}");
                }
                state.fallback_tenant.clone()
            }
        },
        None => state.fallback_tenant.clone(),
    };

    let Some(tenant_id) = tenant_id else {
        return not_found();
    };

    // Entitlements are per tenant now, not per process (ADR-012 supersedes ADR-009 §4).
    // Resolved here, once per request, and carried as an extension so handlers and the
    // feature gate read the same value.
    //
    // A tenant whose subscription cannot be read falls back to the least-privileged
    // entitlements rather than to the boot-time value: failing towards *fewer*
    // permissions is the only safe direction.
    let entitlements = load_entitlements_for(&state, &tenant_id).await;

    let mut request = request;
    request.extensions_mut().insert(entitlements);

    tenant::scope(tenant_id, next.run(request)).await
}

async fn load_entitlements_for(state: &TenantLayerState, tenant_id: &str) -> Arc<Entitlements> {
    let subscription = tenant::scope(tenant_id.to_string(), async {
        state.platform.get_subscription(tenant_id).await
    })
    .await;

    let Ok(subscription) = subscription else {
        return state.fallback_entitlements.clone();
    };

    let plan = TenantPlan {
        tier: subscription.tier,
        trial_ends_at: subscription.trial_ends_at,
        term_ends_at: subscription.term_ends_at,
        quota_overrides: subscription.quota_overrides,
        window_overrides: subscription.window_overrides,
    };

    match resolve_for_tenant(&state.tier_config, &plan) {
        Ok(resolved) => Arc::new(resolved),
        Err(_) => state.fallback_entitlements.clone(),
    }
}
