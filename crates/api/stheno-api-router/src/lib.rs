//! `stheno-api-router`: entitlement-gated route composition (ENTITLEMENTS.md 6.1).
//!
//! The rule this crate exists to enforce:
//!
//! > A gate is a mount-point decision, never an `if` inside domain logic.
//!
//! Gated routes are simply not mounted. A request to an unmounted route gets axum's
//! 404 — deliberately not a 403, because a 403 confirms the feature exists and invites
//! probing (ENTITLEMENTS.md 6.1).
//!
//! Domain modules never see `Entitlements`; `scripts/check-no-domain-gates.sh` fails the
//! build if they do.

use std::sync::Arc;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{middleware, Router};
use stheno_api_core::{Entitlements, Feature};

/// A route group that only mounts when its feature is entitled.
pub struct GatedGroup {
    pub feature: Feature,
    pub routes: Router,
}

impl GatedGroup {
    pub fn new(feature: Feature, routes: Router) -> Self {
        Self { feature, routes }
    }
}

/// Mounts `core` unconditionally, then each gated group whose feature is entitled.
///
/// `core` is the never-gateable surface from ENTITLEMENTS.md 4.1 — product, cart, order,
/// COD payment, tax, currency, store, customer, auth, inventory, manual fulfilment and
/// the platform primitives. None of it costs marginal money, so none of it is sellable.
pub fn mount_gated(core: Router, entitlements: &Entitlements, gated: Vec<GatedGroup>) -> Router {
    gated
        .into_iter()
        .filter(|group| entitlements.has(group.feature))
        .fold(core, |app, group| app.merge(group.routes))
}

/// Mounts tenant-aware feature groups for the shared hosted server.
///
/// A single process serves Trial, Free and Pro tenants, so a startup-time router selection
/// cannot be the source of truth. This composition-level guard runs before the feature router
/// is entered and returns the exact response an unmounted route would return. Domain modules
/// remain unaware of plans; the tenant layer places the resolved entitlements in the request.
pub fn mount_gated_for_request(core: Router, gated: Vec<GatedGroup>) -> Router {
    gated.into_iter().fold(core, |app, group| {
        let feature = group.feature;
        let routes = group.routes.layer(middleware::from_fn(
            move |request: Request, next: Next| async move {
                let permitted = request
                    .extensions()
                    .get::<Arc<Entitlements>>()
                    .is_some_and(|entitlements| entitlements.has(feature));
                if permitted {
                    next.run(request).await
                } else {
                    axum::http::StatusCode::NOT_FOUND.into_response()
                }
            },
        ));
        app.merge(routes)
    })
}

/// Gates the buyer half of `/auth/{actor_type}/…`.
///
/// The customer and staff auth surfaces share one route: the actor is a path parameter,
/// so there is no separate mount point to withhold and `mount_gated_for_request` has
/// nothing to act on. This is the same guard keyed on the path segment instead of the
/// mount, and it returns the same 404 — a store whose buyers have no accounts must be
/// indistinguishable from one where the route never existed.
///
/// Deliberately not generic over the feature or the segment. There is one caller and one
/// feature; a second one can generalise it then.
pub fn gate_buyer_auth(routes: Router) -> Router {
    routes.layer(middleware::from_fn(
        |request: Request, next: Next| async move {
            let mut segments = request.uri().path().trim_start_matches('/').split('/');
            let is_buyer_auth =
                segments.next() == Some("auth") && segments.next() == Some("customer");
            let permitted = !is_buyer_auth
                || request
                    .extensions()
                    .get::<Arc<Entitlements>>()
                    .is_some_and(|entitlements| entitlements.has(Feature::BuyerAccounts));
            if permitted {
                next.run(request).await
            } else {
                axum::http::StatusCode::NOT_FOUND.into_response()
            }
        },
    ))
}

/// The features this deployment will actually mount, sorted.
///
/// The tier-matrix test asserts against this: as routes are added, it fails loudly when
/// something is mounted unconditionally that should have been gated
/// (ENTITLEMENTS.md 8).
pub fn mounted_features(entitlements: &Entitlements) -> Vec<Feature> {
    let mut features: Vec<Feature> = entitlements.features().collect();
    features.sort();
    features
}

/// Medusa-compatible feature-flag map for `GET /admin/feature-flags` (ENTITLEMENTS.md 7.1).
///
/// Every known feature appears, true or false — the forked dashboard reads absent and
/// false differently, so the map must be total rather than sparse.
pub fn feature_flags(entitlements: &Entitlements) -> Vec<(&'static str, bool)> {
    ALL_FEATURES
        .iter()
        .map(|f| (f.as_str(), entitlements.has(*f)))
        .collect()
}

/// Every gateable feature. Kept exhaustive so `feature_flags` cannot silently omit one.
///
/// "Kept exhaustive" was a comment and nothing more until P37-22: `CustomTheme` and
/// `Branding` were added to `Feature` and never added here, so the map the dashboard reads
/// omitted them for as long as they existed — and the dashboard reads absent and false
/// differently. `all_features_lists_every_variant` in `tests/gated_mounting.rs` now makes
/// the compiler enforce it, because a comment cannot.
pub const ALL_FEATURES: [Feature; 11] = [
    Feature::SpreadsheetImport,
    Feature::CsvExport,
    Feature::WebPush,
    Feature::AnalyticsBasic,
    Feature::PaymentGateway,
    Feature::CourierDispatch,
    Feature::FraudScore,
    Feature::CustomDomain,
    Feature::CustomTheme,
    Feature::Branding,
    Feature::BuyerAccounts,
];
