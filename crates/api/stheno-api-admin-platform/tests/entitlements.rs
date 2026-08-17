//! P9-14b / P9-14c: `GET /admin/feature-flags` in Medusa's shape, and the additive
//! `GET /admin/entitlements` reporting features, quotas, usage and grace state.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use stheno_api_admin_platform::{admin_platform_routes, AdminPlatformContext, UsageSnapshot};
use stheno_api_core::{QuotaKey, Tier, TierConfig};
use stheno_workflow::WorkflowStore;
use tower::ServiceExt;

const TIERS: &str = include_str!("../../../../config/tiers.toml");

fn app(tier: Tier, usage: UsageSnapshot) -> axum::Router {
    let entitlements = TierConfig::from_toml(TIERS)
        .expect("config/tiers.toml must parse")
        .resolve(tier, &BTreeMap::new())
        .expect("tier must resolve");

    admin_platform_routes(AdminPlatformContext {
        entitlements: Arc::new(entitlements),
        usage: Arc::new(usage),
        workflows: Arc::new(WorkflowStore::new()),
    })
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn feature_flags_use_medusas_exact_shape() {
    let (status, json) = get_json(
        app(Tier::Free, UsageSnapshot::new()),
        "/admin/feature-flags",
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Medusa returns { "feature_flags": { name: bool } } — the dashboard reads this
    // directly, so the wrapper key and the flat bool map both matter.
    let flags = json["feature_flags"]
        .as_object()
        .expect("feature_flags must be an object");
    assert_eq!(flags.len(), 8, "the map must be total, not sparse");
    assert_eq!(flags["web_push"], Value::Bool(true));
    assert_eq!(flags["payment_gateway"], Value::Bool(false));
}

#[tokio::test]
async fn feature_flags_flip_on_pro() {
    let (_, json) = get_json(app(Tier::Pro, UsageSnapshot::new()), "/admin/feature-flags").await;
    let flags = json["feature_flags"].as_object().unwrap();
    assert_eq!(flags["payment_gateway"], Value::Bool(true));
    assert_eq!(flags["courier_dispatch"], Value::Bool(true));
    assert_eq!(flags["custom_domain"], Value::Bool(true));
}

#[tokio::test]
async fn entitlements_reports_tier_features_quotas_and_windows() {
    let usage = UsageSnapshot::new()
        .with(QuotaKey::MaxProducts, 42)
        .with(QuotaKey::MaxStaffSeats, 2);
    let (status, json) = get_json(app(Tier::Free, usage), "/admin/entitlements").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(json["tier"], "free");
    assert_eq!(json["trial_ends_at"], Value::Null);

    let features = json["features"].as_array().unwrap();
    assert!(features.contains(&Value::String("web_push".into())));
    assert!(!features.contains(&Value::String("payment_gateway".into())));

    let quotas = json["quotas"].as_array().unwrap();
    let products = quotas
        .iter()
        .find(|q| q["key"] == "max_products")
        .expect("max_products must be reported");
    assert_eq!(products["limit"], 100);
    assert_eq!(products["used"], 42);
    assert_eq!(products["state"], "ok");

    let windows = json["windows"].as_array().unwrap();
    assert_eq!(windows[0]["key"], "analytics_window_days");
    assert_eq!(windows[0]["limit"], 30);
}

#[tokio::test]
async fn an_over_limit_quota_reports_grace_not_failure() {
    // ENTITLEMENTS.md 6.4: 412 products on a limit of 100 after a downgrade.
    // All 412 stay live and sellable; only creation is blocked.
    let usage = UsageSnapshot::new().with(QuotaKey::MaxProducts, 412);
    let (status, json) = get_json(app(Tier::Free, usage), "/admin/entitlements").await;
    assert_eq!(status, StatusCode::OK, "grace is a state, not an error");

    let quotas = json["quotas"].as_array().unwrap();
    let products = quotas.iter().find(|q| q["key"] == "max_products").unwrap();
    assert_eq!(products["used"], 412);
    assert_eq!(products["limit"], 100);
    assert_eq!(products["state"], "grace");
}

#[tokio::test]
async fn unlimited_quotas_omit_the_limit_rather_than_reporting_zero() {
    // Reporting `limit: 0` would read as "nothing allowed" to any client that does not
    // know the sentinel. Absent is unambiguous.
    let (_, json) = get_json(app(Tier::Pro, UsageSnapshot::new()), "/admin/entitlements").await;
    let quotas = json["quotas"].as_array().unwrap();
    let products = quotas.iter().find(|q| q["key"] == "max_products").unwrap();
    assert!(
        products.get("limit").is_none(),
        "unlimited must omit the field, got {products}"
    );
    assert_eq!(products["state"], "ok");
}
