//! Admin locale, translation, property-label and store-setting routes.
//!
//! `stheno-translation` and `stheno-settings` both had complete services and no HTTP
//! surface. `/admin/locales` answered with five hardcoded languages that were never
//! stored — so a locale created through the API was invisible, and one deleted from the
//! database still appeared — and `/admin/translations` always returned an empty array.
//! `/admin/property-labels` did not exist at all despite `settings.property_label` being
//! a real table.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use stheno_api_core::ErrorResponse;
use stheno_settings::{SetPropertyLabelInput, SetStoreSettingInput, SettingsService};
use stheno_translation::{CreateLocaleInput, SetTranslationInput, TranslationService};

#[derive(Clone)]
pub struct LocalisationContext {
    pub translation_service: Arc<TranslationService>,
    pub settings_service: Arc<SettingsService>,
}

fn failed(status: StatusCode, code: &'static str, e: stheno_types::Error) -> Response {
    (status, Json(ErrorResponse::new(code, &e.to_string()))).into_response()
}

// ── Locales ──────────────────────────────────────────────────────────────────

pub async fn list_locales(
    State(ctx): State<LocalisationContext>,
) -> Result<impl IntoResponse, Response> {
    let locales = ctx
        .translation_service
        .list_locales()
        .await
        .map_err(|e| failed(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state", e))?;
    let count = locales.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "locales": locales, "data": locales,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_locale(
    State(ctx): State<LocalisationContext>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let locale = ctx
        .translation_service
        .get_locale(&code)
        .await
        .map_err(|e| failed(StatusCode::NOT_FOUND, "not_found", e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "locale": locale, "data": locale })),
    ))
}

pub async fn create_locale(
    State(ctx): State<LocalisationContext>,
    Json(input): Json<CreateLocaleInput>,
) -> Result<impl IntoResponse, Response> {
    let locale = ctx
        .translation_service
        .create_locale(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e))?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "locale": locale, "data": locale })),
    ))
}

// ── Translations ─────────────────────────────────────────────────────────────

pub async fn set_translation(
    State(ctx): State<LocalisationContext>,
    Json(input): Json<SetTranslationInput>,
) -> Result<impl IntoResponse, Response> {
    let translation = ctx
        .translation_service
        .set_translation(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "translation": translation, "data": translation })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct ResolveQuery {
    pub resource_type: String,
    pub resource_id: String,
    pub field: String,
    pub locale: String,
    /// Scope narrowing. A translation may differ per sales channel or region, which is
    /// what makes this a resolution rather than a lookup.
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    /// Returned when nothing is translated, so the caller always gets a usable string.
    pub fallback: Option<String>,
}

/// `GET /admin/translations/resolve` — the value a given locale actually sees.
///
/// Resolution order (override, then translation, then the base field) lives in the
/// service; this only exposes it so the translations screen can show what a shopper in
/// that locale would read.
pub async fn resolve_translation(
    State(ctx): State<LocalisationContext>,
    Query(query): Query<ResolveQuery>,
) -> Result<impl IntoResponse, Response> {
    // Infallible by design: it falls back rather than erroring, so a missing translation
    // shows the base value instead of a broken screen.
    let value = ctx
        .translation_service
        .resolve_scoped_field(
            &query.locale,
            &query.resource_type,
            &query.resource_id,
            &query.field,
            query.sales_channel_id.as_deref(),
            query.region_id.as_deref(),
            query.fallback.as_deref().unwrap_or(""),
        )
        .await;
    Ok((StatusCode::OK, Json(json!({ "value": value }))))
}

// ── Property labels ──────────────────────────────────────────────────────────

pub async fn set_property_label(
    State(ctx): State<LocalisationContext>,
    Json(input): Json<SetPropertyLabelInput>,
) -> Result<impl IntoResponse, Response> {
    let label = ctx
        .settings_service
        .set_property_label(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "property_label": label, "data": label })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct PropertyLabelQuery {
    pub resource: String,
    pub property: String,
}

pub async fn get_property_label(
    State(ctx): State<LocalisationContext>,
    Query(query): Query<PropertyLabelQuery>,
) -> Result<impl IntoResponse, Response> {
    let label = ctx
        .settings_service
        .get_property_label(&query.resource, &query.property)
        .await
        .map_err(|e| failed(StatusCode::NOT_FOUND, "not_found", e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "property_label": label, "data": label })),
    ))
}

// ── Store settings ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SettingsQuery {
    pub category: Option<String>,
}

pub async fn list_settings(
    State(ctx): State<LocalisationContext>,
    Query(query): Query<SettingsQuery>,
) -> Result<impl IntoResponse, Response> {
    let category = query.category.as_deref().unwrap_or("general");
    let settings = ctx
        .settings_service
        .list_settings_by_category(category)
        .await
        .map_err(|e| failed(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state", e))?;
    let count = settings.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "settings": settings, "data": settings, "count": count,
        })),
    ))
}

pub async fn set_setting(
    State(ctx): State<LocalisationContext>,
    Json(input): Json<SetStoreSettingInput>,
) -> Result<impl IntoResponse, Response> {
    let setting = ctx
        .settings_service
        .set_setting(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "setting": setting, "data": setting })),
    ))
}
