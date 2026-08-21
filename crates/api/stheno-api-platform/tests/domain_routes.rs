//! P31-07 & P31-09: Domain management routes and verification state.

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

const BOOTSTRAP: &str = "domain-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router, Arc<PlatformService>) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    platform
        .create_tenant(NewTenant {
            id: "ten_domain_test".to_string(),
            name: "Domain Test Merchant".to_string(),
            tier: "free".to_string(),
            domain: Some("default.localhost".to_string()),
        })
        .await
        .expect("tenant created");

    let router = platform_routes(PlatformApiContext {
        platform: Arc::clone(&platform),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
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
async fn domain_management_routes_and_verification() {
    let (_db, router, _platform) = app().await;

    // 1. List domains
    let (status, body) = call(
        &router,
        "GET",
        "/platform/tenants/ten_domain_test/domains",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);
    assert_eq!(body["domains"][0]["domain"], "default.localhost");

    // 2. Add custom domain
    let (status, body) = call(
        &router,
        "POST",
        "/platform/tenants/ten_domain_test/domains",
        Some(json!({
            "domain": "custom.brand.com",
            "is_primary": true
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "add domain: {body}");
    assert_eq!(body["domain"]["domain"], "custom.brand.com");
    assert_eq!(body["domain"]["is_primary"], true);

    // 3. Check domain verification state (P31-09)
    let (status, body) = call(
        &router,
        "GET",
        "/platform/tenants/ten_domain_test/domains/custom.brand.com/verification",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["verification"]["dns_pointed"], true);
    assert_eq!(body["verification"]["tls_issued"], true);

    // 4. Remove a domain
    let (status, _) = call(
        &router,
        "DELETE",
        "/platform/tenants/ten_domain_test/domains/default.localhost",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 5. List domains again -> only custom domain remains
    let (status, body) = call(
        &router,
        "GET",
        "/platform/tenants/ten_domain_test/domains",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);
    assert_eq!(body["domains"][0]["domain"], "custom.brand.com");
}
