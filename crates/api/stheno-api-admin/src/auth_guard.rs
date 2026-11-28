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
use axum::http::header::{AUTHORIZATION, COOKIE};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_auth::AuthService;

/// The `actor_type` a staff identity carries. Set at registration by
/// `POST /auth/{actor_type}/{provider_id}/register`.
pub const ADMIN_ACTOR_TYPE: &str = "user";
const ADMIN_SESSION_COOKIE: &str = "__Host-stheno-admin-session";

enum AdminCredential {
    Bearer(String),
    SessionCookie(String),
}

fn admin_credential(headers: &HeaderMap) -> Option<AdminCredential> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned)
        .map(AdminCredential::Bearer)
        .or_else(|| session_cookie_token(headers).map(AdminCredential::SessionCookie))
}

fn session_cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == ADMIN_SESSION_COOKIE && !value.is_empty()).then(|| value.to_string())
            })
        })
}

fn credential_origin_allowed(
    credential: &AdminCredential,
    method: &Method,
    headers: &HeaderMap,
) -> bool {
    match credential {
        AdminCredential::Bearer(_) => true,
        AdminCredential::SessionCookie(_) => {
            stheno_http::csrf::validate_origin_host(method, headers).is_ok()
        }
    }
}

fn deny(status: StatusCode, r#type: &str, message: &str) -> Response {
    (status, Json(json!({ "type": r#type, "message": message }))).into_response()
}

/// Rejects any `/admin` request that does not carry a valid staff bearer token or session cookie.
pub async fn require_admin(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(credential) = admin_credential(request.headers()) else {
        return deny(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Missing Authorization bearer token",
        );
    };
    if !credential_origin_allowed(&credential, request.method(), request.headers()) {
        return deny(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Cookie session Origin does not match request Host",
        );
    }
    let token = match credential {
        AdminCredential::Bearer(token) | AdminCredential::SessionCookie(token) => token,
    };

    let claims = match auth_service.verify_jwt(&token).await {
        Ok(claims) => claims,
        Err(_) => {
            return deny(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Invalid or expired token",
            );
        }
    };

    if claims.actor_type.as_deref() != Some(ADMIN_ACTOR_TYPE) {
        return deny(
            StatusCode::FORBIDDEN,
            "not_allowed",
            "Token is not bound to a staff user",
        );
    }

    request.extensions_mut().insert(claims);
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::credential_origin_allowed;
    use axum::body::Body;
    use axum::http::{HeaderMap, HeaderValue, Method, Request};

    fn request(origin: Option<&str>, authorization: Option<&str>) -> Request<Body> {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("admin.example.test"));
        headers.insert(
            "cookie",
            HeaderValue::from_static("__Host-stheno-admin-session=token"),
        );
        if let Some(origin) = origin {
            headers.insert(
                "origin",
                HeaderValue::from_str(origin).expect("valid test Origin"),
            );
        }
        if let Some(authorization) = authorization {
            headers.insert(
                "authorization",
                HeaderValue::from_str(authorization).expect("valid test Authorization"),
            );
        }
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("/admin/products")
            .body(Body::empty())
            .expect("request is valid");
        *request.headers_mut() = headers;
        request
    }

    fn credential_is_allowed(request: &Request<Body>) -> bool {
        let credential = super::admin_credential(request.headers())
            .expect("test request has an admin credential");
        credential_origin_allowed(&credential, request.method(), request.headers())
    }

    #[test]
    fn cookie_post_requires_a_matching_origin_host() {
        assert!(!credential_is_allowed(&request(None, None)));
        assert!(!credential_is_allowed(&request(
            Some("https://sibling.admin.example.test"),
            None
        )));
        assert!(credential_is_allowed(&request(
            Some("https://admin.example.test"),
            None
        )));
    }

    #[test]
    fn bearer_post_does_not_require_origin() {
        assert!(credential_is_allowed(&request(
            None,
            Some("Bearer signed-token")
        )));
    }
}
