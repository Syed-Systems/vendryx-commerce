//! Admin Store Settings API — key-value configuration for COD rules, store policies, and merchant settings.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_settings::dto::{SetStoreSettingInput, StoreSettingDTO};
use stheno_settings::SettingsService;

#[derive(Clone)]
pub struct SettingsContext {
    pub settings_service: Arc<SettingsService>,
}

fn failed(status: StatusCode, code: &'static str, message: String) -> Response {
    (status, Json(ErrorResponse::new(code, &message))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct SetSettingPayload {
    pub value: serde_json::Value,
    pub category: Option<String>,
    pub is_secret: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SettingsQuery {
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicSettingResponse {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub updated_at: DateTime<Utc>,
}

impl From<StoreSettingDTO> for PublicSettingResponse {
    fn from(s: StoreSettingDTO) -> Self {
        Self {
            id: s.id,
            key: s.key,
            value: s.value,
            category: s.category,
            updated_at: s.updated_at,
        }
    }
}

/// `GET /admin/settings/{key}`
pub async fn get_setting(
    State(ctx): State<SettingsContext>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let setting = ctx.settings_service.get_setting(&key).await.map_err(|_| {
        failed(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("Setting '{key}' not found"),
        )
    })?;

    let res: PublicSettingResponse = setting.into();
    Ok((StatusCode::OK, Json(json!({ "setting": res, "data": res }))))
}

/// `POST /admin/settings/{key}`
pub async fn set_setting(
    State(ctx): State<SettingsContext>,
    Path(key): Path<String>,
    Json(payload): Json<SetSettingPayload>,
) -> Result<impl IntoResponse, Response> {
    // Edge case 1: Validate payload size (reject payloads > 64KB)
    let payload_str = serde_json::to_string(&payload.value).unwrap_or_default();
    if payload_str.len() > 65_536 {
        return Err(failed(
            StatusCode::BAD_REQUEST,
            "payload_too_large",
            "Setting value exceeds maximum allowed size of 64KB".to_string(),
        ));
    }

    // Edge case 2: COD rules domain validation
    if key == "cod_rules" {
        if let Some(max_amount) = payload.value.get("maxOrderAmount").and_then(|v| v.as_f64()) {
            if max_amount < 0.0 {
                return Err(failed(
                    StatusCode::BAD_REQUEST,
                    "invalid_cod_rules",
                    "maxOrderAmount cannot be negative".to_string(),
                ));
            }
        }
    }

    let category = payload.category.unwrap_or_else(|| match key.as_str() {
        "cod_rules" => "payment".to_string(),
        "store_policies" => "policies".to_string(),
        _ => "general".to_string(),
    });

    let setting = ctx
        .settings_service
        .set_setting(SetStoreSettingInput {
            key,
            value: payload.value,
            category,
            is_secret: payload.is_secret,
        })
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unexpected_state",
                e.to_string(),
            )
        })?;

    let res: PublicSettingResponse = setting.into();
    Ok((StatusCode::OK, Json(json!({ "setting": res, "data": res }))))
}

/// `GET /admin/settings`
pub async fn list_settings(
    State(ctx): State<SettingsContext>,
    Query(query): Query<SettingsQuery>,
) -> Result<impl IntoResponse, Response> {
    let category = query.category.as_deref().unwrap_or("general");
    let settings = ctx
        .settings_service
        .list_settings_by_category(category)
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unexpected_state",
                e.to_string(),
            )
        })?;

    let res: Vec<PublicSettingResponse> = settings.into_iter().map(Into::into).collect();
    let count = res.len();

    Ok((
        StatusCode::OK,
        Json(json!({
            "settings": res,
            "data": res,
            "count": count,
        })),
    ))
}

/// `POST /admin/settings`
pub async fn set_setting_body(
    State(ctx): State<SettingsContext>,
    Json(input): Json<SetStoreSettingInput>,
) -> Result<impl IntoResponse, Response> {
    let setting = ctx
        .settings_service
        .set_setting(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e.to_string()))?;

    let res: PublicSettingResponse = setting.into();
    Ok((StatusCode::OK, Json(json!({ "setting": res, "data": res }))))
}
