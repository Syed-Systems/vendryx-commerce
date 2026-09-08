//! P31-11: Merchant health column: orders in the last 7 days, GMV, last activity.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewTenant, PlatformService};

const BOOTSTRAP: &str = "health-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router, Arc<PlatformService>) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    platform
        .create_tenant(NewTenant {
            id: "ten_health_check".to_string(),
            name: "Health Checked Merchant".to_string(),
            tier: "free".to_string(),
            domain: Some("health.localhost".to_string()),
        })
        .await
        .expect("tenant created");

    let router = platform_routes(PlatformApiContext {
        platform: Arc::clone(&platform),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
        fulfillment_service: None,
    });

    (db, router, platform)
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
async fn tenant_health_metrics_returned() {
    let (_db, router, _platform) = app().await;

    let (status, body) = call(
        &router,
        "GET",
        "/platform/tenants/ten_health_check/health",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["health"]["tenant_id"], "ten_health_check");
    assert!(body["health"]["health_verdict"].as_str().is_some());
    assert!(body["health"]["orders_last_7d"].as_i64().is_some());
    assert!(body["health"]["gmv_last_7d"].is_object());
}
