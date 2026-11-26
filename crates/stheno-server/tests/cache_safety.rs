use axum::body::Body;
use axum::http::{header, HeaderMap, Request, StatusCode};
use serde_json::{json, Value};
use stheno_platform::{NewTenant, PlatformService};
use stheno_server::TestApp;
use tower::ServiceExt;

const ADMIN_A: &str = "merchant-a.admin.example.test";
const ADMIN_B: &str = "merchant-b.admin.example.test";
const STORE_A: &str = "shop-a.example.test";

async fn call(
    app: &axum::Router,
    host: &str,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, host);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let body = body.map_or_else(Body::empty, |value| Body::from(value.to_string()));
    let response = app
        .clone()
        .oneshot(builder.body(body).expect("request builds"))
        .await
        .expect("router responds");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body reads");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

async fn add_tenant(platform: &PlatformService, id: &str, domain: &str) {
    platform
        .create_tenant(NewTenant {
            id: id.to_string(),
            name: id.to_string(),
            tier: "free".to_string(),
            domain: Some(domain.to_string()),
        })
        .await
        .expect("test tenant is created");
}

fn assert_not_public(headers: &HeaderMap) {
    let cache_control = headers
        .get(header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(!cache_control.contains("public"));
    assert!(!cache_control.contains("s-maxage"));
}

#[tokio::test]
async fn public_catalog_cache_headers_are_tenant_scoped() {
    let harness = TestApp::spawn().await;
    let platform = PlatformService::new(
        stheno_db::create_pool(&harness.database_url, 5).expect("platform pool builds"),
    );
    add_tenant(&platform, "ten_cache_a", STORE_A).await;

    let (status, headers, _) =
        call(&harness.app, STORE_A, "GET", "/store/products", &[], None).await;
    assert_eq!(status, StatusCode::OK);
    let cache_control = headers
        .get(header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(cache_control.contains("public"));
    assert!(cache_control.contains("s-maxage="));
    assert!(headers.get(header::SET_COOKIE).is_none());
    assert!(!headers
        .get(header::VARY)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("cookie"));

    for tag in headers
        .get("cache-tag")
        .and_then(|value| value.to_str().ok())
        .expect("catalog response carries cache tags")
        .split(',')
    {
        assert!(tag.starts_with("tenant:ten_cache_a:"), "foreign tag: {tag}");
    }

    harness.teardown().await;
}

#[tokio::test]
async fn customer_cart_checkout_auth_and_admin_responses_are_not_public() {
    let harness = TestApp::spawn().await;
    let platform = PlatformService::new(
        stheno_db::create_pool(&harness.database_url, 5).expect("platform pool builds"),
    );
    add_tenant(&platform, "ten_private_a", STORE_A).await;

    for (method, path) in [
        ("GET", "/store/carts/ca_missing"),
        ("POST", "/store/carts/ca_missing/complete"),
        ("POST", "/store/payment-collections"),
        ("GET", "/store/customers/me"),
        ("GET", "/auth/session"),
        ("GET", "/admin/products"),
        ("GET", "/admin/__unknown__"),
    ] {
        let (_, headers, _) = call(&harness.app, STORE_A, method, path, &[], None).await;
        assert_not_public(&headers);
    }
    let (_, admin_headers, _) =
        call(&harness.app, STORE_A, "GET", "/admin/products", &[], None).await;
    assert_eq!(
        admin_headers
            .get("content-security-policy")
            .and_then(|value| value.to_str().ok()),
        Some("script-src 'self'; frame-ancestors 'none'")
    );

    harness.teardown().await;
}

#[tokio::test]
async fn admin_session_cookie_is_rejected_on_another_tenant_host() {
    let harness = TestApp::spawn().await;
    let platform = PlatformService::new(
        stheno_db::create_pool(&harness.database_url, 5).expect("platform pool builds"),
    );
    add_tenant(&platform, "ten_admin_a", ADMIN_A).await;
    add_tenant(&platform, "ten_admin_b", ADMIN_B).await;

    let (status, _, registered) = call(
        &harness.app,
        ADMIN_A,
        "POST",
        "/auth/user/emailpass/register",
        &[],
        Some(json!({
            "email": "owner@example.test",
            "password": "TestPassword-OnlyForThisCase-49!",
            "actor_id": "usr_admin_a"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = registered["token"]
        .as_str()
        .expect("registration issues token");

    let (status, session_headers, _) = call(
        &harness.app,
        ADMIN_A,
        "POST",
        "/auth/session",
        &[("authorization", &format!("Bearer {token}"))],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let cookie = session_headers
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("session response issues a cookie");
    assert!(cookie.starts_with("__Host-stheno-admin-session="));
    assert!(!cookie.contains("Domain="));
    let cookie_pair = cookie
        .split(';')
        .next()
        .expect("cookie value exists")
        .to_string();

    let (status, _, _) = call(
        &harness.app,
        ADMIN_A,
        "GET",
        "/admin/products",
        &[("cookie", &cookie_pair)],
        None,
    )
    .await;
    assert_ne!(status, StatusCode::UNAUTHORIZED);

    let (status, _, _) = call(
        &harness.app,
        ADMIN_B,
        "GET",
        "/admin/products",
        &[("cookie", &cookie_pair)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    harness.teardown().await;
}
