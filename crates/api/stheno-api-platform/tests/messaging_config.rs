//! P35-07: Email / SMS provider config and templates.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;

const BOOTSTRAP: &str = "messaging-bootstrap-secret";
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
async fn messaging_config_get_and_update() {
    let (_db, router) = app().await;

    // 1. Get messaging config
    let (status, body) = call(&router, "GET", "/platform/config/messaging", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messaging"]["email_provider"], "smtp");

    // 2. Update messaging config
    let (status, body) = call(
        &router,
        "PUT",
        "/platform/config/messaging",
        Some(json!({
            "email_provider": "resend",
            "sms_provider": "infobip",
            "from_email": "orders@stheno.store",
            "order_confirmation_template": "Order #{{order_id}} confirmed."
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messaging"]["email_provider"], "resend");
    assert_eq!(body["messaging"]["sms_provider"], "infobip");
}
