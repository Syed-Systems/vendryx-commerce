//! P9-14d: conditional router mounting per feature; disabled routes 404.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use std::collections::BTreeMap;
use stheno_api_core::{Feature, Tier, TierConfig};
use stheno_api_router::{feature_flags, mount_gated, GatedGroup, ALL_FEATURES};
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
            "core commerce must never be gated on {:?}",
            tier
        );
    }
}

#[tokio::test]
async fn gated_routes_are_absent_when_the_feature_is_off() {
    // Free and Trial do not include payment_gateway or courier_dispatch.
    for tier in [Tier::Trial, Tier::Free] {
        assert_eq!(
            status(app(tier), "/store/payment-gateway").await,
            StatusCode::NOT_FOUND,
            "payment gateway must be unmounted on {:?}",
            tier
        );
        assert_eq!(
            status(app(tier), "/admin/courier-dispatch").await,
            StatusCode::NOT_FOUND,
            "courier dispatch must be unmounted on {:?}",
            tier
        );
    }
}

#[tokio::test]
async fn gated_routes_mount_when_the_feature_is_on() {
    assert_eq!(
        status(app(Tier::Pro), "/store/payment-gateway").await,
        StatusCode::OK
    );
    assert_eq!(
        status(app(Tier::Pro), "/admin/courier-dispatch").await,
        StatusCode::OK
    );
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
    ];

    for feature in every {
        // Non-exhaustive match = compile error. That is the whole point of this arm.
        match feature {
            SpreadsheetImport | CsvExport | WebPush | AnalyticsBasic | PaymentGateway
            | CourierDispatch | FraudScore | CustomDomain | CustomTheme | Branding => {}
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
