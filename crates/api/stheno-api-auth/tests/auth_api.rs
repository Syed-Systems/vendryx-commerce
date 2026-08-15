//! Test P7-22: Medusa v2 wire-compatible Auth API endpoints parity.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_auth::{auth_routes, AuthApiContext};
use stheno_auth::AuthService;
use stheno_flows::AuthWorkflows;
use tower::ServiceExt;

#[tokio::test]
async fn test_auth_api_endpoints_parity() {
    let auth_service = Arc::new(AuthService::new(None));
    let auth_flows = Arc::new(AuthWorkflows::new(auth_service));
    let app = auth_routes(AuthApiContext { auth_flows });

    // 1. POST /auth/customer/emailpass/register
    let reg_req = Request::builder()
        .uri("/auth/customer/emailpass/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "medusa_user@example.com",
                "password": "SuperSecretPassword123!",
                "actor_id": "cust_123"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(reg_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let reg_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(reg_json["token"].is_string());
    let refresh_token = reg_json["refresh_token"].as_str().unwrap().to_string();

    // 2. POST /auth/customer/emailpass (Authenticate)
    let auth_req = Request::builder()
        .uri("/auth/customer/emailpass")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "medusa_user@example.com",
                "password": "SuperSecretPassword123!"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(auth_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(auth_json["token"].is_string());

    // 3. POST /auth/token/refresh
    let refresh_req = Request::builder()
        .uri("/auth/token/refresh")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "refresh_token": refresh_token
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(refresh_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let refresh_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(refresh_json["token"].is_string());
    assert!(refresh_json["refresh_token"].is_string());

    // 4. POST /auth/customer/emailpass/reset-password
    let reset_req = Request::builder()
        .uri("/auth/customer/emailpass/reset-password")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "medusa_user@example.com"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(reset_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let reset_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(reset_json["success"].as_bool().unwrap());
    let reset_token = reset_json["reset_token"].as_str().unwrap().to_string();

    // 5. POST /auth/customer/emailpass/confirm-reset-password
    let confirm_req = Request::builder()
        .uri("/auth/customer/emailpass/confirm-reset-password")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "token": reset_token,
                "new_password": "NewBrandPassword789!"
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(confirm_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let confirm_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(confirm_json["success"].as_bool().unwrap());
}
