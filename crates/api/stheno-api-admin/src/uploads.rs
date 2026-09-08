//! Admin Uploads and Files API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_file::dto::{CreatePresignedUploadInput, UploadFileInput};
use stheno_file::FileService;

#[derive(Clone)]
pub struct AdminUploadContext {
    pub file_service: Arc<FileService>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUploadRequest {
    pub filename: String,
    pub content_type: Option<String>,
    pub data: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// GET /admin/uploads
pub async fn list_uploads(
    State(ctx): State<AdminUploadContext>,
) -> Result<impl IntoResponse, Response> {
    let files = ctx.file_service.list_files().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = files.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "files": files,
            "uploads": files,
            "data": files,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

/// POST /admin/uploads
pub async fn create_upload(
    State(ctx): State<AdminUploadContext>,
    Json(payload): Json<CreateUploadRequest>,
) -> Result<impl IntoResponse, Response> {
    if let Some(data_str) = payload.data {
        let bytes = data_str.into_bytes();

        let file = ctx
            .file_service
            .upload_file(UploadFileInput {
                filename: payload.filename,
                content_type: payload.content_type,
                bytes,
                metadata: payload.metadata,
            })
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("upload_failed", &e.to_string())),
                )
                    .into_response()
            })?;

        Ok((
            StatusCode::CREATED,
            Json(json!({
                "file": file,
                "upload": file,
                "data": file,
                "files": [file.clone()],
                "uploads": [file],
            })),
        ))
    } else {
        let presigned = ctx
            .file_service
            .create_presigned_upload(CreatePresignedUploadInput {
                filename: payload.filename,
                content_type: payload
                    .content_type
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
                expires_in_seconds: Some(3600),
            })
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("presigned_failed", &e.to_string())),
                )
                    .into_response()
            })?;

        Ok((
            StatusCode::CREATED,
            Json(json!({
                "presigned": presigned,
                "data": presigned,
            })),
        ))
    }
}

/// GET /admin/uploads/{id}
pub async fn get_upload(
    State(ctx): State<AdminUploadContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let file = ctx.file_service.get_file(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "file": file, "upload": file, "data": file })),
    ))
}

/// DELETE /admin/uploads/{id}
pub async fn delete_upload(
    State(ctx): State<AdminUploadContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.file_service.delete_file(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "file", "deleted": true })),
    ))
}

/// POST /admin/uploads/presigned-urls
pub async fn create_presigned_url(
    State(ctx): State<AdminUploadContext>,
    Json(payload): Json<CreateUploadRequest>,
) -> Result<impl IntoResponse, Response> {
    let presigned = ctx
        .file_service
        .create_presigned_upload(CreatePresignedUploadInput {
            filename: payload.filename,
            content_type: payload
                .content_type
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            expires_in_seconds: Some(3600),
        })
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("presigned_failed", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "presigned": presigned,
            "data": presigned,
        })),
    ))
}
