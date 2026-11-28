use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use std::ffi::OsString;
use stheno_db::{create_pool, testing::TestDb};
use stheno_server::TestApp;
use stheno_store::dto::UpsertBrandingInput;
use stheno_store::StoreService;
use tower::ServiceExt;

const STOREFRONT_TEST_HOST: &str = "storefront-test.example.test";
const STOREFRONT_FLAG: &str = "STHENO_ENGINE_STOREFRONT";

static STOREFRONT_ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Holds the process-wide env lock until the request finishes and restores the prior value
/// even if setup or an assertion unwinds.
struct StorefrontFlagGuard {
    previous: Option<OsString>,
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

impl StorefrontFlagGuard {
    async fn set(enabled: bool) -> Self {
        let lock = STOREFRONT_ENV_LOCK.lock().await;
        let previous = std::env::var_os(STOREFRONT_FLAG);
        if enabled {
            std::env::set_var(STOREFRONT_FLAG, "1");
        } else {
            std::env::remove_var(STOREFRONT_FLAG);
        }
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for StorefrontFlagGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => std::env::set_var(STOREFRONT_FLAG, value),
            None => std::env::remove_var(STOREFRONT_FLAG),
        }
    }
}

async fn register_test_host(app: &TestApp) {
    app.db()
        .execute(&format!(
            "INSERT INTO platform.tenant_domain (domain, tenant_id, is_primary) \
             VALUES ('{STOREFRONT_TEST_HOST}', '{tenant}', true)",
            tenant = TestDb::TEST_TENANT
        ))
        .await;
}

fn country_request() -> Request<Body> {
    Request::builder()
        .uri("/bd")
        .header(header::HOST, STOREFRONT_TEST_HOST)
        .body(Body::empty())
        .expect("request is valid")
}

#[tokio::test]
async fn renders_the_tenant_store_name() {
    let _flag = StorefrontFlagGuard::set(true).await;
    let app = TestApp::spawn().await;
    register_test_host(&app).await;

    let store_service = StoreService::new(create_pool(&app.database_url, 5).expect("store pool"));
    store_service
        .upsert_branding(UpsertBrandingInput {
            store_name: Some("Test <Store>".to_string()),
            ..Default::default()
        })
        .await
        .expect("test branding is saved");
    drop(store_service);

    let response = app
        .app
        .clone()
        .oneshot(country_request())
        .await
        .expect("storefront request returns a response");
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read storefront HTML");
    let html = String::from_utf8(body.to_vec()).expect("HTML is UTF-8");

    app.teardown().await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        content_type.starts_with("text/html"),
        "expected HTML content type, got {content_type:?}"
    );
    assert!(
        html.contains("Test &lt;Store&gt;"),
        "the tenant store name should be present and HTML-escaped: {html}"
    );
    assert!(
        !html.contains("Test <Store>"),
        "the tenant store name must not be emitted as raw HTML: {html}"
    );
}

#[tokio::test]
async fn flag_off_leaves_routes_unchanged() {
    let _flag = StorefrontFlagGuard::set(false).await;
    let app = TestApp::spawn().await;
    register_test_host(&app).await;

    let response = app
        .app
        .clone()
        .oneshot(country_request())
        .await
        .expect("storefront request returns a response");
    let status = response.status();
    app.teardown().await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
