//! Admin authentication guard.
//!
//! Every `/admin/*` route routes through this layer, so the credential check lives here
//! once rather than in eighty-five handlers. A missing or invalid token is a **401**; a
//! valid token belonging to a customer rather than a staff user is a **403** — the caller
//! proved who they are and is simply not staff.
//!
//! Medusa's admin surface accepts a staff JWT (`POST /auth/user/emailpass` issues one),
//! which is what the forked dashboard sends. Secret API keys are a separate credential
//! and are not accepted here yet.

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_auth::AuthService;

/// The `actor_type` a staff identity carries. Set at registration by
/// `POST /auth/{actor_type}/{provider_id}/register`.
pub const ADMIN_ACTOR_TYPE: &str = "user";

fn deny(status: StatusCode, r#type: &str, message: &str) -> Response {
    (status, Json(json!({ "type": r#type, "message": message }))).into_response()
}

/// Rejects any `/admin` request that does not carry a valid staff bearer token.
pub async fn require_admin(
    State(auth_service): State<Arc<AuthService>>,
    request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned);

    let Some(token) = token else {
        return deny(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Missing Authorization bearer token",
        );
    };

    let claims = match auth_service.verify_jwt(&token).await {
        Ok(claims) => claims,
        Err(_) => {
            return deny(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Invalid or expired token",
            )
        }
    };

    if claims.actor_type.as_deref() != Some(ADMIN_ACTOR_TYPE) {
        return deny(
            StatusCode::FORBIDDEN,
            "not_allowed",
            "Token is not bound to a staff user",
        );
    }

    next.run(request).await
}
