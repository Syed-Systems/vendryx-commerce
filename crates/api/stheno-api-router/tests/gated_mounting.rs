//! P9-14d: conditional router mounting per feature; disabled routes 404.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use std::collections::BTreeMap;
use stheno_api_core::{Feature, Tier, TierConfig};
use stheno_api_router::{
    feature_flags, gate_buyer_auth, mount_gated, mount_gated_for_request, GatedGroup, ALL_FEATURES,
};
use tower::ServiceExt;

const TIERS: &str = include_str!("../../../../config/tiers.toml");

fn entitlements(tier: Tier) -> stheno_api_core::Entitlements {
    TierConfig::from_toml(TIERS)
        .expect("config/tiers.toml must parse")
        .resolve(tier, &BTreeMap::new())
        .expect("tier must resolve")
}

fn core_routes() -> Router {
    Router::new().route("/store/products", get(|| async { "products" }))
}

fn gateway_routes() -> Router {
    Router::new().route("/store/payment-gateway", get(|| async { "gateway" }))
}

fn courier_routes() -> Router {
    Router::new().route("/admin/courier-dispatch", get(|| async { "courier" }))
}

fn app(tier: Tier) -> Router {
    mount_gated(
        core_routes(),
        &entitlements(tier),
        vec![
            GatedGroup::new(Feature::PaymentGateway, gateway_routes()),
            GatedGroup::new(Feature::CourierDispatch, courier_routes()),
        ],
    )
}

async fn status(router: Router, uri: &str) -> StatusCode {
    router
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn core_routes_mount_on_every_tier() {
    for tier in [Tier::Trial, Tier::Free, Tier::Pro] {
        assert_eq!(
            status(app(tier), "/store/products").await,
            StatusCode::OK,
            "core commerce must never be gated on {tier:?}"
        );
    }
}

#[tokio::test]
async fn gated_routes_are_absent_when_the_feature_is_off() {
    // Free does not include payment_gateway or courier_dispatch; Trial receives the Pro feature set.
    assert_eq!(
        status(app(Tier::Free), "/store/payment-gateway").await,
        StatusCode::NOT_FOUND,
        "payment gateway must be unmounted on Free"
    );
    assert_eq!(
        status(app(Tier::Free), "/admin/courier-dispatch").await,
        StatusCode::NOT_FOUND,
        "courier dispatch must be unmounted on Free"
    );
}

#[tokio::test]
async fn gated_routes_mount_when_the_feature_is_on() {
    for tier in [Tier::Trial, Tier::Pro] {
        assert_eq!(
            status(app(tier), "/store/payment-gateway").await,
            StatusCode::OK,
            "payment gateway must mount on {tier:?}"
        );
        assert_eq!(
            status(app(tier), "/admin/courier-dispatch").await,
            StatusCode::OK,
            "courier dispatch must mount on {tier:?}"
        );
    }
}

#[tokio::test]
async fn a_disabled_route_is_404_not_403() {
    // 403 would confirm the feature exists and invite probing (ENTITLEMENTS.md 6.1).
    let code = status(app(Tier::Free), "/store/payment-gateway").await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert_ne!(code, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn feature_flag_map_is_total_and_matches_the_tier() {
    let free = feature_flags(&entitlements(Tier::Free));
    assert_eq!(
        free.len(),
        ALL_FEATURES.len(),
        "the flag map must list every known feature"
    );

    let lookup: BTreeMap<_, _> = free.into_iter().collect();
    assert!(lookup["web_push"], "web push is free on every tier");
    assert!(!lookup["payment_gateway"], "gateway is Pro-only");

    let pro: BTreeMap<_, _> = feature_flags(&entitlements(Tier::Pro))
        .into_iter()
        .collect();
    assert!(pro["payment_gateway"]);
    assert!(pro["custom_domain"]);
}

/// P37-22: `ALL_FEATURES` must list every `Feature` variant, enforced by the compiler.
///
/// The previous guard was `assert_eq!(free.len(), 8)` — a magic number that asserted
/// totality only by coincidence. Two variants (`CustomTheme`, `Branding`) were added to
/// the enum and never to the constant, and the number 8 stayed true because it was
/// describing the constant rather than the enum.
///
/// The match below is the real gate: add a variant to `Feature` without adding it to
/// `every`, and this stops compiling. Miss it in `ALL_FEATURES`, and the assert fails.
#[test]
fn all_features_lists_every_variant() {
    use stheno_api_core::Feature::*;

    let every = [
        SpreadsheetImport,
        CsvExport,
        WebPush,
        AnalyticsBasic,
        PaymentGateway,
        CourierDispatch,
        FraudScore,
        CustomDomain,
        CustomTheme,
        Branding,
        BuyerAccounts,
        AiAssistant,
    ];

    for feature in every {
        // Non-exhaustive match = compile error. That is the whole point of this arm.
        match feature {
            SpreadsheetImport | CsvExport | WebPush | AnalyticsBasic | PaymentGateway
            | CourierDispatch | FraudScore | CustomDomain | CustomTheme | Branding
            | BuyerAccounts | AiAssistant => {}
        }
        assert!(
            ALL_FEATURES.contains(&feature),
            "{} is a Feature variant but is missing from ALL_FEATURES",
            feature.as_str()
        );
    }

    assert_eq!(
        ALL_FEATURES.len(),
        every.len(),
        "ALL_FEATURES has an entry that is not a Feature variant"
    );
}

#[tokio::test]
async fn hosted_admin_routes_use_the_request_tenant_not_the_boot_tier() {
    let app = mount_gated_for_request(
        Router::new().route("/admin/orders", get(|| async { "orders" })),
        vec![GatedGroup::new(
            Feature::PaymentGateway,
            Router::new().route("/admin/gateways", get(|| async { "gateways" })),
        )],
    );

    let mut free = Request::builder()
        .uri("/admin/gateways")
        .body(Body::empty())
        .unwrap();
    free.extensions_mut()
        .insert(std::sync::Arc::new(entitlements(Tier::Free)));
    assert_eq!(
        app.clone().oneshot(free).await.unwrap().status(),
        StatusCode::NOT_FOUND,
        "a disabled feature must look unmounted for Free"
    );

    for tier in [Tier::Trial, Tier::Pro] {
        let mut request = Request::builder()
            .uri("/admin/gateways")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(std::sync::Arc::new(entitlements(tier)));
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::OK,
            "an enabled feature must mount for {tier:?}"
        );
    }

    for tier in [Tier::Trial, Tier::Free, Tier::Pro] {
        let mut request = Request::builder()
            .uri("/admin/orders")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(std::sync::Arc::new(entitlements(tier)));
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::OK
        );
    }
}

// ---------------------------------------------------------------------------
// Buyer-account archetype (PRODUCT.md 2). The merchant picks this on the store
// request; no tier grants it, so it always arrives as a per-tenant override.
// ---------------------------------------------------------------------------

fn buyer_auth_app() -> Router {
    gate_buyer_auth(
        Router::new()
            .route(
                "/auth/{actor_type}/providers",
                get(|| async { "providers" }),
            )
            .route("/auth/token/refresh", get(|| async { "refresh" })),
    )
}

fn with_buyer_accounts(uri: &str, enabled: bool) -> Request<Body> {
    let mut entitlements = entitlements(Tier::Free);
    entitlements.set_feature(Feature::BuyerAccounts, enabled);
    let mut request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(std::sync::Arc::new(entitlements));
    request
}

#[test]
fn no_tier_grants_buyer_accounts() {
    // The archetype is a store property, not something a plan sells. If a tier ever
    // starts granting it, a merchant who chose "no accounts" would silently get them
    // by upgrading, which is a product change disguised as a pricing change.
    for tier in [Tier::Trial, Tier::Free, Tier::Pro] {
        assert!(
            !entitlements(tier).has(Feature::BuyerAccounts),
            "{} must not grant buyer_accounts from config/tiers.toml",
            tier.as_str()
        );
    }
}

#[tokio::test]
async fn buyer_auth_404s_without_the_entitlement() {
    let response = buyer_auth_app()
        .oneshot(with_buyer_accounts("/auth/customer/providers", false))
        .await
        .unwrap();
    // 404, never 403: a 403 confirms the feature exists and invites probing.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn buyer_auth_is_reachable_with_the_entitlement() {
    let response = buyer_auth_app()
        .oneshot(with_buyer_accounts("/auth/customer/providers", true))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn staff_auth_is_never_touched_by_the_buyer_gate() {
    // The whole risk of gating on a path segment instead of a mount point: getting it
    // wrong locks every merchant's staff out of their own dashboard. /auth/user and
    // /auth/token must be unaffected with the archetype off.
    for uri in ["/auth/user/providers", "/auth/token/refresh"] {
        let response = buyer_auth_app()
            .oneshot(with_buyer_accounts(uri, false))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{uri} must not be gated by buyer_accounts"
        );
    }
}

#[tokio::test]
async fn store_customer_routes_follow_the_archetype() {
    let app = mount_gated_for_request(
        Router::new().route("/store/products", get(|| async { "products" })),
        vec![GatedGroup::new(
            Feature::BuyerAccounts,
            Router::new().route("/store/customers/me", get(|| async { "me" })),
        )],
    );

    for (enabled, expected) in [(false, StatusCode::NOT_FOUND), (true, StatusCode::OK)] {
        let response = app
            .clone()
            .oneshot(with_buyer_accounts("/store/customers/me", enabled))
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }

    // Guest checkout is the default path under both archetypes and must never move.
    let response = app
        .oneshot(with_buyer_accounts("/store/products", false))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
