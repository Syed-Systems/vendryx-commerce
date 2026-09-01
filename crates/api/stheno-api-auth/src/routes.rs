//! Medusa v2 wire-compatible Auth API handlers and route definitions (P7-22).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use stheno_auth::{
    AuthenticateInput, CreateIdentityInput, PasswordResetConfirmInput, PasswordResetRequestInput,
};
use stheno_flows::AuthWorkflows;
use stheno_http::HttpError;

#[derive(Clone)]
pub struct AuthApiContext {
    pub auth_flows: Arc<AuthWorkflows>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthPayload {
    pub email: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
    pub actor_id: Option<String>,
    pub user_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponseWrapper {
    pub token: String,
}

/// POST /auth/{actor_type}/{provider_id}
pub async fn authenticate(
    State(ctx): State<AuthApiContext>,
    Path((actor_type, provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let (_, tokens) = ctx
        .auth_flows
        .authenticate(AuthenticateInput {
            provider_id,
            entity_id: email,
            password: payload.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "actor_type": actor_type,
        })),
    ))
}

/// POST /auth/{actor_type}/{provider_id}/register
pub async fn register(
    State(ctx): State<AuthApiContext>,
    Path((actor_type, provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let identity = ctx
        .auth_flows
        .create_identity(CreateIdentityInput {
            provider_id: provider_id.clone(),
            entity_id: email.clone(),
            password: payload.password.clone(),
            actor_id: payload.actor_id,
            actor_type: Some(actor_type.clone()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: payload.user_metadata,
        })
        .await?;

    let (_, tokens) = ctx
        .auth_flows
        .authenticate(AuthenticateInput {
            provider_id,
            entity_id: email,
            password: payload.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "identity": identity,
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "actor_type": actor_type,
        })),
    ))
}

/// POST /auth/token/refresh
pub async fn refresh_token(
    State(ctx): State<AuthApiContext>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let raw_refresh = payload
        .refresh_token
        .ok_or_else(|| stheno_types::Error::Invalid {
            code: "missing_refresh_token",
            message: "Missing 'refresh_token' parameter".to_string(),
            field: Some("refresh_token".to_string()),
        })?;

    let tokens = ctx.auth_flows.refresh_tokens(&raw_refresh).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
        })),
    ))
}

/// POST /auth/{actor_type}/{provider_id}/reset-password
pub async fn reset_password(
    State(ctx): State<AuthApiContext>,
    Path((_actor_type, _provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let email = payload.email.unwrap_or_default();
    let token_opt = ctx
        .auth_flows
        .request_password_reset(PasswordResetRequestInput { entity_id: email })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "reset_token": token_opt,
        })),
    ))
}

/// POST /auth/{actor_type}/{provider_id}/confirm-reset-password
pub async fn confirm_reset_password(
    State(ctx): State<AuthApiContext>,
    Path((_actor_type, _provider_id)): Path<(String, String)>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, HttpError> {
    let token = payload.token.ok_or_else(|| stheno_types::Error::Invalid {
        code: "missing_reset_token",
        message: "Missing 'token' in request".to_string(),
        field: Some("token".to_string()),
    })?;

    let new_password = payload
        .new_password
        .ok_or_else(|| stheno_types::Error::Invalid {
            code: "missing_new_password",
            message: "Missing 'new_password' in request".to_string(),
            field: Some("new_password".to_string()),
        })?;

    let updated_identity = ctx
        .auth_flows
        .confirm_password_reset(PasswordResetConfirmInput {
            token,
            new_password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "identity": updated_identity,
        })),
    ))
}

/// GET /auth/{actor_type}/providers
/// GET /auth/{actor_type}/providers
///
/// Reads `auth.auth_provider` rather than naming emailpass in the source. The dashboard
/// renders its login form from this list, so a hardcoded answer meant a merchant who
/// enabled Google or GitHub was shown a screen that never offered it.
pub async fn list_providers(
    State(ctx): State<AuthApiContext>,
    Path(actor_type): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let providers = ctx.auth_flows.list_providers().await?;
    let rendered: Vec<serde_json::Value> = providers
        .into_iter()
        .filter(|p| p.is_enabled)
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "actor_type": actor_type,
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(json!({ "providers": rendered }))))
}

/// Mounts all Medusa v2 compatible Auth API routes.
pub fn auth_routes(ctx: AuthApiContext) -> Router {
    Router::new()
        .route("/auth/{actor_type}/providers", get(list_providers))
        .route("/auth/{actor_type}/{provider_id}", post(authenticate))
        .route("/auth/{actor_type}/{provider_id}/register", post(register))
        .route("/auth/token/refresh", post(refresh_token))
        .route(
            "/auth/{actor_type}/{provider_id}/reset-password",
            post(reset_password),
        )
        .route(
            "/auth/{actor_type}/{provider_id}/confirm-reset-password",
            post(confirm_reset_password),
        )
        .with_state(ctx)
}
