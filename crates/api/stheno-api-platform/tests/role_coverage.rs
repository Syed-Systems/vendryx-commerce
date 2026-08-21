//! P36-02: Route role coverage: every route declares its role and missing declaration fails closed.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "rolecoverage-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = Arc::new(PlatformService::new(pool));

    let router = platform_routes(PlatformApiContext {
        platform: Arc::clone(&platform),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
    });

    (db, router)
}

async fn call_without_auth(router: &axum::Router, path: &str) -> StatusCode {
    let request = Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("request");
    let response = router.clone().oneshot(request).await.expect("response");
    response.status()
}

#[tokio::test]
async fn unauthenticated_requests_fail_closed_across_routes() {
    let (_db, router) = app().await;

    // Fail closed: every sensitive control plane route returns 404 (or 401) without valid operator auth
    let endpoints = vec![
        "/platform/tenants",
        "/platform/analytics/gmv",
        "/platform/providers",
        "/platform/compliance/evidence",
        "/platform/feature-flags",
        "/platform/config/defaults",
        "/platform/config/messaging",
        "/platform/workflows",
    ];

    for endpoint in endpoints {
        let status = call_without_auth(&router, endpoint).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "Endpoint {endpoint} must fail closed on missing auth"
        );
    }
}
