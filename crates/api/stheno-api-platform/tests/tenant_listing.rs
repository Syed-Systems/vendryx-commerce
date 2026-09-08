//! P31-10: Tenant listing: search, filter by tier/status, sort, cursor pagination.

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

const BOOTSTRAP: &str = "listing-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router, Arc<PlatformService>) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    for i in 1..=5 {
        platform
            .create_tenant(NewTenant {
                id: format!("ten_list_{i}"),
                name: format!("Merchant Store {i}"),
                tier: if i % 2 == 0 {
                    "pro".to_string()
                } else {
                    "free".to_string()
                },
                domain: Some(format!("shop{i}.localhost")),
            })
            .await
            .expect("tenant created");
    }

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
async fn tenant_listing_with_search_and_filter() {
    let (_db, router, _platform) = app().await;

    // 1. List all tenants
    let (status, body) = call(&router, "GET", "/platform/tenants", None).await;
    assert_eq!(status, StatusCode::OK);
    let tenants = body["tenants"].as_array().expect("array");
    assert!(tenants.len() >= 5);

    // 2. Search by merchant name
    let (status, body) = call(&router, "GET", "/platform/tenants?search=Store+3", None).await;
    assert_eq!(status, StatusCode::OK);
    let tenants = body["tenants"].as_array().expect("array");
    assert_eq!(tenants.len(), 1);
    assert_eq!(tenants[0]["id"], "ten_list_3");

    // 3. Filter by tier
    let (status, body) = call(&router, "GET", "/platform/tenants?tier=pro", None).await;
    assert_eq!(status, StatusCode::OK);
    let tenants = body["tenants"].as_array().expect("array");
    assert!(tenants.len() >= 2);
    for t in tenants {
        assert_eq!(t["tier"], "pro");
    }
}
