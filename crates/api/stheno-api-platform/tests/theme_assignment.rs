//! P35-03: Per-tenant theme assignment with tier entitlement enforcement.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewTenant, PlatformService};

const BOOTSTRAP: &str = "themeassign-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, Arc<PlatformService>, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    let router = platform_routes(PlatformApiContext {
        platform: Arc::clone(&platform),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
    });

    (db, platform, router)
}

async fn call(
    router: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {BOOTSTRAP}"));
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = router
        .clone()
        .oneshot(builder.body(body).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("bytes");
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

#[tokio::test]
async fn theme_assignment_enforces_tier_feature_gate() {
    let (_db, platform, router) = app().await;

    // 1. Create a free merchant
    platform
        .create_tenant(NewTenant {
            id: "ten_free_theme".to_string(),
            name: "Free Theme Shop".to_string(),
            tier: "free".to_string(),
            domain: None,
        })
        .await
        .expect("free tenant created");

    // 2. Assigning theme_luxury (requires custom_theme) to free merchant is refused with 403 Forbidden
    let (status, _) = call(
        &router,
        "POST",
        "/platform/tenants/ten_free_theme/theme",
        Some(json!({ "theme_id": "theme_luxury" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 3. Assigning theme_default succeeds
    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_free_theme/theme",
        Some(json!({ "theme_id": "theme_default" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["assignment"]["theme_id"], "theme_default");
}
