//! Admin API Keys & Publishable Keys API Handlers.
//!
//! `stheno-api-key` has had a complete service since the identity/access schema landed —
//! create with a hashed token, retrieve, list, update, revoke, soft-delete and validate,
//! all against `api_key.api_key`. None of it was ever called. Every handler in this file
//! returned a literal: `create_api_key` minted an id and a `sk_test_…` token in JSON and
//! stored nothing, `list` answered with an empty array, `revoke` and `delete` reported
//! success and did nothing.
//!
//! The visible consequence was worse than an empty screen. A merchant could create a
//! secret key, be shown a token that looked real, copy it into a deployment — and then
//! authenticate with nothing, because the key had never existed. Revoking a compromised
//! key reported success and left it working, which is the same class of bug pointed at
//! security rather than convenience.
//!
//! Medusa's wire shape is not the DTO's shape: the dashboard reads `type`, `redacted` and
//! `token`, while the domain calls those `key_type` and `redacted_key`. `to_medusa`
//! translates rather than leaking the internal names, and — this is the part worth being
//! careful about — the plain token exists in exactly one response, the create. Every read
//! after that carries the redacted form, because the service only ever stores a hash.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use stheno_api_key::{ApiKeyDTO, ApiKeyService, CreateApiKeyInput, UpdateApiKeyInput};
use stheno_http::HttpError;

#[derive(Clone)]
pub struct AdminApiKeyContext {
    pub api_key_service: Arc<ApiKeyService>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub title: String,
    pub r#type: Option<String>,
    pub sales_channel_ids: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub title: Option<String>,
    pub sales_channel_ids: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Domain DTO to Medusa's `AdminApiKey`.
///
/// `token` is the redacted form on every read. The plain token is returned by the create
/// route alone and is not recoverable afterwards — the service stores only a hash.
fn to_medusa(key: &ApiKeyDTO) -> Value {
    json!({
        "id": key.id,
        "title": key.title,
        "type": key.key_type,
        "redacted": key.redacted_key,
        "token": key.redacted_key,
        "sales_channels": key.sales_channel_ids,
        "permissions": key.permissions,
        "last_used_at": key.last_used_at,
        "revoked_at": key.revoked_at,
        "revoked_by": Value::Null,
        "created_at": key.created_at,
        "updated_at": key.updated_at,
        "deleted_at": Value::Null,
        "metadata": key.metadata,
    })
}

fn envelope(field: &str, key: &ApiKeyDTO) -> Value {
    let body = to_medusa(key);
    json!({ field: body, "data": body })
}

fn listing(field: &str, keys: &[ApiKeyDTO]) -> Value {
    let rows: Vec<Value> = keys.iter().map(to_medusa).collect();
    json!({
        field: rows,
        "data": rows,
        "count": rows.len(),
        "offset": 0,
        "limit": rows.len(),
    })
}

/// Keys of one kind.
///
/// `list_api_keys` returns both kinds because the table holds both; the split into two
/// routes is Medusa's, not the domain's. Filtering here keeps that presentation detail out
/// of the service.
async fn list_of_type(ctx: &AdminApiKeyContext, key_type: &str) -> Result<Vec<ApiKeyDTO>, HttpError> {
    let keys = ctx
        .api_key_service
        .list_api_keys()
        .await?;
    Ok(keys.into_iter().filter(|k| k.key_type == key_type).collect())
}

async fn create_of_type(
    ctx: &AdminApiKeyContext,
    key_type: &str,
    payload: CreateApiKeyRequest,
    field: &str,
) -> Result<(StatusCode, Json<Value>), HttpError> {
    let created = ctx
        .api_key_service
        .create_api_key(CreateApiKeyInput {
            key_type: key_type.to_string(),
            title: payload.title,
            sales_channel_ids: payload.sales_channel_ids,
            permissions: payload.permissions,
            created_by: None,
            metadata: payload.metadata,
        })
        .await?;

    // The one response that carries the plain token. It is not stored and cannot be read
    // back, so a client that does not keep it here has to rotate the key.
    let mut body = to_medusa(&created.api_key);
    body["token"] = json!(created.token);

    Ok((
        StatusCode::CREATED,
        Json(json!({ field: body, "data": body })),
    ))
}

/// GET /admin/api-keys
pub async fn list_api_keys(
    State(ctx): State<AdminApiKeyContext>,
) -> Result<impl IntoResponse, HttpError> {
    let keys = list_of_type(&ctx, "secret").await?;
    Ok((StatusCode::OK, Json(listing("api_keys", &keys))))
}

/// POST /admin/api-keys
pub async fn create_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let key_type = payload
        .r#type
        .clone()
        .unwrap_or_else(|| "secret".to_string());
    create_of_type(&ctx, &key_type, payload, "api_key").await
}

/// GET /admin/api-keys/{id}
pub async fn get_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let key = ctx
        .api_key_service
        .get_api_key(&id)
        .await?;
    Ok((StatusCode::OK, Json(envelope("api_key", &key))))
}

/// POST /admin/api-keys/{id}
pub async fn update_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let key = ctx
        .api_key_service
        .update_key(
            &id,
            UpdateApiKeyInput {
                title: payload.title,
                sales_channel_ids: payload.sales_channel_ids,
                permissions: payload.permissions,
                metadata: payload.metadata,
            },
        )
        .await?;
    Ok((StatusCode::OK, Json(envelope("api_key", &key))))
}

/// DELETE /admin/api-keys/{id}
pub async fn delete_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    ctx.api_key_service
        .delete_key(&id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "api_key", "deleted": true })),
    ))
}

/// POST /admin/api-keys/{id}/revoke
///
/// Revocation is what makes a leaked key safe, so it has to reach the database. This
/// previously answered with `{ id, revoked_at: now }` and left the key working.
pub async fn revoke_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let key = ctx
        .api_key_service
        .revoke_key(&id, None)
        .await?;
    Ok((StatusCode::OK, Json(envelope("api_key", &key))))
}

/// GET /admin/publishable-api-keys
pub async fn list_publishable_api_keys(
    State(ctx): State<AdminApiKeyContext>,
) -> Result<impl IntoResponse, HttpError> {
    let keys = list_of_type(&ctx, "publishable").await?;
    Ok((
        StatusCode::OK,
        Json(listing("publishable_api_keys", &keys)),
    ))
}

/// POST /admin/publishable-api-keys
pub async fn create_publishable_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    create_of_type(&ctx, "publishable", payload, "publishable_api_key").await
}

/// GET /admin/publishable-api-keys/{id}
pub async fn get_publishable_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let key = ctx
        .api_key_service
        .get_api_key(&id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(envelope("publishable_api_key", &key)),
    ))
}

/// POST /admin/publishable-api-keys/{id}
pub async fn update_publishable_api_key(
    State(ctx): State<AdminApiKeyContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let key = ctx
        .api_key_service
        .update_key(
            &id,
            UpdateApiKeyInput {
                title: payload.title,
                sales_channel_ids: payload.sales_channel_ids,
                permissions: payload.permissions,
                metadata: payload.metadata,
            },
        )
        .await?;
    Ok((
        StatusCode::OK,
        Json(envelope("publishable_api_key", &key)),
    ))
}
