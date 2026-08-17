//! Admin API Keys & Publishable Keys API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use stheno_http::HttpError;

#[derive(Clone)]
pub struct AdminApiKeyContext;

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub title: String,
    pub r#type: Option<String>,
}

/// GET /admin/api-keys
pub async fn list_api_keys(
    State(_ctx): State<AdminApiKeyContext>,
) -> Result<impl IntoResponse, HttpError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "api_keys": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

/// POST /admin/api-keys
pub async fn create_api_key(
    State(_ctx): State<AdminApiKeyContext>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let now = chrono::Utc::now();
    let key = json!({
        "id": format!("apk_{}", ulid::Ulid::new()),
        "token": format!("sk_test_{}", ulid::Ulid::new()),
        "title": payload.title,
        "type": payload.r#type.unwrap_or_else(|| "secret".to_string()),
        "redacted": "sk_test_••••••••",
        "created_at": now,
        "updated_at": now
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "api_key": key,
            "data": key
        })),
    ))
}

/// DELETE /admin/api-keys/{id}
pub async fn delete_api_key(
    State(_ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "api_key",
            "deleted": true
        })),
    ))
}

/// POST /admin/api-keys/{id}/revoke
pub async fn revoke_api_key(
    State(_ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let key = json!({
        "id": id,
        "revoked_at": chrono::Utc::now()
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "api_key": key,
            "data": key
        })),
    ))
}

/// GET /admin/publishable-api-keys
pub async fn list_publishable_api_keys(
    State(_ctx): State<AdminApiKeyContext>,
) -> Result<impl IntoResponse, HttpError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "publishable_api_keys": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

/// POST /admin/publishable-api-keys
pub async fn create_publishable_api_key(
    State(_ctx): State<AdminApiKeyContext>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let now = chrono::Utc::now();
    let key = json!({
        "id": format!("pk_{}", ulid::Ulid::new()),
        "token": format!("pk_test_{}", ulid::Ulid::new()),
        "title": payload.title,
        "type": "publishable",
        "created_at": now,
        "updated_at": now
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "publishable_api_key": key,
            "data": key
        })),
    ))
}
