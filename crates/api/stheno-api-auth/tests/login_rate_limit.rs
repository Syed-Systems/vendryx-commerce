//! The login endpoint is throttled.
//!
//! `POST /auth/{actor_type}/{provider_id}` had no rate limit of any kind. The shared
//! `stheno_http::rate_limit::RateLimiter` existed, was correct, and was referenced from
//! nowhere — three modules had each hand-rolled their own in-memory counter instead, and
//! none of those was on the login path.
//!
//! Keyed on the submitted account, because `axum::serve` runs without
//! `into_make_service_with_connect_info` so no handler can read a peer address.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use stheno_api_auth::{auth_routes, AuthApiContext};
use stheno_auth::AuthService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows::AuthWorkflows;
use stheno_http::rate_limit::RateLimiter;
use tower::ServiceExt;

const ATTEMPTS: usize = 3;

fn login_request() -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/auth/user/emailpass")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"victim@example.com","password":"wrong-password"}"#,
        ))
        .unwrap()
}

#[tokio::test]
async fn repeated_failures_are_throttled_then_a_correct_password_is_too() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let auth_service = Arc::new(AuthService::new(pool.clone(), None));
    let app = auth_routes(AuthApiContext {
        auth_flows: Arc::new(AuthWorkflows::new(auth_service.clone())),
        login_limiter: Arc::new(RateLimiter::new(ATTEMPTS, 60)),
    });

    // A real account, so the only thing separating success from failure is the password.
    auth_service
        .create_identity(stheno_auth::CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "victim@example.com".to_string(),
            password: Some("CorrectHorse1!".to_string()),
            actor_id: Some("cus_victim".to_string()),
            actor_type: Some("customer".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await;

    // Burn the budget on wrong passwords.
    for attempt in 1..=ATTEMPTS {
        let resp = app
            .clone()
            .oneshot(login_request())
            .await
            .expect("response");
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "attempt {attempt} of {ATTEMPTS} should still reach the password check"
        );
    }

    // The next one is throttled, even though the password is now correct.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user/emailpass")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"victim@example.com","password":"CorrectHorse1!"}"#,
                ))
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "the budget is spent, so even the correct password must not get through"
    );
    assert_eq!(
        resp.headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok()),
        Some("60"),
        "a throttled client must be told when to retry"
    );

    // A different account is not throttled by the first one's failures.
    let other = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/user/emailpass")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"someone-else@example.com","password":"wrong-password"}"#,
                ))
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(
        other.status(),
        StatusCode::UNAUTHORIZED,
        "one account being attacked must not lock out every other account"
    );

    db.drop().await;
}
