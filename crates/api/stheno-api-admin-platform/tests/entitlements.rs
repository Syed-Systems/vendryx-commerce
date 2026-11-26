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

    // The handlers read the caller's entitlements from a request extension, because one
    // process now serves tenants on different tiers (ADR-012). In the server the tenant
    // layer sets it; here the test supplies it directly.
    let entitlements = Arc::new(entitlements);
    admin_platform_routes(AdminPlatformContext {
        entitlements: entitlements.clone(),
        tier_config: Arc::new(TierConfig::from_toml(TIERS).expect("tier config must parse")),
        usage: Arc::new(usage),
        workflows: Arc::new(WorkflowStore::new()),
        platform: None,
    })
    .layer(axum::Extension(entitlements))
}

#[tokio::test]
async fn subscription_reports_deployment_configured_plan_price() {
    let configured = TIERS.replace("price         = \"20.00\"", "price         = \"19.95\"");
    let config = TierConfig::from_toml(&configured).expect("deployment price override parses");
    let entitlements = Arc::new(
        config
            .resolve(Tier::Pro, &BTreeMap::new())
            .expect("Pro tier resolves"),
    );
    let app = admin_platform_routes(AdminPlatformContext {
        entitlements: entitlements.clone(),
        tier_config: Arc::new(config),
        usage: Arc::new(UsageSnapshot::new()),
        workflows: Arc::new(WorkflowStore::new()),
        platform: None,
    })
    .layer(axum::Extension(entitlements));

    let (status, json) = get_json(app, "/admin/billing/subscription").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["subscription"]["plan_amount"], "19.95");
    assert_eq!(json["subscription"]["currency"], "usd");
    assert_eq!(json["subscription"]["billing_interval"], "month");
    assert_eq!(json["subscription"]["pro_plan"]["amount"], "19.95");
    assert_eq!(json["subscription"]["pro_plan"]["currency"], "usd");
    assert_eq!(json["subscription"]["pro_plan"]["interval"], "month");
}

#[tokio::test]
async fn free_subscription_exposes_configured_pro_upgrade_price() {
    let configured = TIERS.replace("price         = \"20.00\"", "price         = \"19.95\"");
    let config = TierConfig::from_toml(&configured).expect("deployment price override parses");
    let entitlements = Arc::new(
        config
            .resolve(Tier::Free, &BTreeMap::new())
            .expect("Free tier resolves"),
    );
    let app = admin_platform_routes(AdminPlatformContext {
        entitlements: entitlements.clone(),
        tier_config: Arc::new(config),
        usage: Arc::new(UsageSnapshot::new()),
        workflows: Arc::new(WorkflowStore::new()),
        platform: None,
    })
    .layer(axum::Extension(entitlements));

    let (status, json) = get_json(app, "/admin/billing/subscription").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["subscription"]["tier"], "free");
    assert_eq!(json["subscription"]["plan_amount"], Value::Null);
    assert_eq!(json["subscription"]["pro_plan"]["amount"], "19.95");
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

    // Totality is the property that matters: the dashboard's hook reads an absent flag
    // as `false`, so a name it asks for and does not get silently hides a screen. This
    // used to be `len() == 8`, which asserted totality only by coincidence — it also
    // forbade any *other* flag from appearing, and so failed the moment the capability
    // flags the fork actually reads were added. Naming both sets keeps the real
    // guarantee and drops the accidental one.
    for name in [
        "spreadsheet_import",
        "csv_export",
        "web_push",
        "analytics_basic",
        "payment_gateway",
        "courier_dispatch",
        "fraud_score",
        "custom_domain",
    ] {
        assert!(
            flags[name].is_boolean(),
            "tier feature {name} must be present"
        );
    }
    for name in ["rbac", "translation", "view_configurations"] {
        assert!(
            flags[name].is_boolean(),
            "capability flag {name} must be present — the dashboard reads it"
        );
    }

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
async fn billing_subscription_reports_unknown_without_a_persisted_subscription() {
    let (status, json) = get_json(
        app(Tier::Free, UsageSnapshot::new()),
        "/admin/billing/subscription",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["subscription"]["tier"], "free");
    assert_eq!(json["subscription"]["status"], "unknown");
    assert_eq!(json["subscription"]["trial_ends_at"], Value::Null);
    assert_eq!(json["subscription"]["term_ends_at"], Value::Null);
    assert_eq!(json["subscription"]["grace_ends_at"], Value::Null);
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
