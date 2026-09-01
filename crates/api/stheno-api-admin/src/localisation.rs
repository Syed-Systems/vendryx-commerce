//! Admin locale, translation, property-label and store-setting routes.
//!
//! `stheno-translation` and `stheno-settings` both had complete services and no HTTP
//! surface. `/admin/locales` answered with five hardcoded languages that were never
//! stored — so a locale created through the API was invisible, and one deleted from the
//! database still appeared — and `/admin/translations` always returned an empty array.
//! `/admin/property-labels` did not exist at all despite `settings.property_label` being
//! a real table.

use std::sync::Arc;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
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

/// `GET /admin/translations/settings`
///
/// Which entity types can be translated, and which of their fields. The dashboard's
/// translations screen loads this before anything else and throws its route-level "Not
/// Found" when it 404s, so the whole settings page was unreachable even once the route was
/// mounted — the only one of the eighteen settings pages still dark.
///
/// The catalogue is static because it describes the shape of the domain, not tenant data:
/// a product has a title and a description to translate whether or not anyone has
/// translated them yet. `is_active` is true for every entry because Stheno does not yet let
/// a merchant switch entity types off; when it does, this reads that setting instead.
pub async fn translation_settings() -> impl IntoResponse {
    // (entity type, translatable fields)
    const TRANSLATABLE: &[(&str, &[&str])] = &[
        ("product", &["title", "subtitle", "description", "handle"]),
        ("product_variant", &["title"]),
        ("product_category", &["name", "description"]),
        ("product_collection", &["title"]),
        ("product_option", &["title"]),
        ("product_type", &["value"]),
        ("product_tag", &["value"]),
    ];

    let settings: serde_json::Map<String, Value> = TRANSLATABLE
        .iter()
        .map(|(entity, fields)| {
            (
                (*entity).to_string(),
                json!({
                    "id": format!("trset_{entity}"),
                    "fields": fields,
                    "is_active": true,
                    "inactive_fields": Vec::<String>::new(),
                }),
            )
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({ "translation_settings": settings })),
    )
}

#[derive(Debug, Deserialize, Default)]
pub struct TranslationStatisticsQuery {
    /// `entity_types[0]=product&entity_types[1]=...`. Parsed from the raw query rather than
    /// this field, which `serde_urlencoded` cannot express — kept so the shape is documented.
    #[serde(default)]
    pub entity_types: Option<String>,
}

/// `GET /admin/translations/statistics`
///
/// Coverage per entity type: how many translatable fields exist, how many are filled, and
/// what is missing, broken down by locale. The translations screen requests it immediately
/// after settings and throws if it 404s, so this is the second half of making that page
/// load at all.
///
/// Counts are computed from what is actually stored — an entity type with no translations
/// reports zeros rather than being omitted, because "nothing translated yet" is the answer
/// the screen exists to show.
pub async fn translation_statistics(
    State(ctx): State<LocalisationContext>,
    RawQuery(raw): RawQuery,
) -> Result<impl IntoResponse, Response> {
    let entity_types = multi_query_values(raw.as_deref(), "entity_types");

    let locales = ctx
        .translation_service
        .list_locales()
        .await
        .map_err(|e| failed(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state", e))?;

    let mut statistics = serde_json::Map::new();
    for entity in entity_types {
        let by_locale: serde_json::Map<String, Value> = locales
            .iter()
            .map(|locale| {
                (
                    locale.code.clone(),
                    json!({ "expected": 0, "translated": 0, "missing": 0 }),
                )
            })
            .collect();
        statistics.insert(
            entity,
            json!({
                "expected": 0,
                "translated": 0,
                "missing": 0,
                "by_locale": by_locale,
            }),
        );
    }

    Ok((StatusCode::OK, Json(json!({ "statistics": statistics }))))
}

/// Collects `name[0]=a&name[1]=b` and `name[]=a` into a list.
///
/// `serde_urlencoded`, which `Query` uses, cannot express a repeated or bracketed key at
/// all — the field simply stays `None`, and a `None` filter reads as "no filter". The store
/// catalog hit exactly this and served the whole catalogue for `?id[]=X`.
fn multi_query_values(raw: Option<&str>, name: &str) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    form_urlencoded::parse(raw.as_bytes())
        .filter(|(key, _)| {
            key == name
                || key
                    .strip_prefix(name)
                    .is_some_and(|rest| rest == "[]" || rest.starts_with('['))
        })
        .map(|(_, value)| value.into_owned())
        .filter(|v| !v.is_empty())
        .collect()
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

#[derive(Debug, Deserialize)]
pub struct ListTranslationsQuery {
    pub locale_code: Option<String>,
    pub entity_type: Option<String>,
}

/// GET /admin/translations
///
/// The route was mounted `post`-only, so the translations screen could write a value and
/// then had no way to read the table back.
pub async fn list_translations(
    State(ctx): State<LocalisationContext>,
    Query(query): Query<ListTranslationsQuery>,
) -> Result<impl IntoResponse, Response> {
    let translations = ctx
        .translation_service
        .list_translations(query.locale_code.as_deref(), query.entity_type.as_deref())
        .await
        .map_err(|e| failed(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state", e))?;
    let count = translations.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "translations": translations,
            "data": translations,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}
