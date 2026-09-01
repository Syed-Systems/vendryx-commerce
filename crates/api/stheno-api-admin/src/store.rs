//! Admin Store, Price Preferences & Notifications API Handlers (P12-12).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_http::HttpError;
use stheno_store::dto::{CreateStoreInput, UpdateStoreInput};
use stheno_store::StoreService;
use stheno_translation::TranslationService;

#[derive(Clone)]
pub struct AdminStoreContext {
    pub store_service: Arc<StoreService>,
    /// Resolves locale codes to their names for `supported_locales`.
    pub translation_service: Arc<TranslationService>,
}

/// Expands `supported_locales` from bare codes into Medusa's `AdminStoreLocale` objects.
///
/// The store DTO holds locale codes — `["en"]` — and that is the right shape for the
/// domain. Medusa's admin contract is a list of objects carrying a nested locale, and the
/// dashboard reads it as `store.supported_locales.map((l) => l.locale)` before touching
/// `.code`. Serving strings therefore mapped to `[undefined]` and took the translations
/// screen down with `Cannot read properties of undefined (reading 'code')` — a shape
/// mismatch that only ever surfaced as a render crash two components away.
async fn expand_supported_locales(ctx: &AdminStoreContext, store: &Value) -> Value {
    let codes: Vec<String> = store
        .get("supported_locales")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    if codes.is_empty() {
        return json!([]);
    }

    // Names are a nicety, not a requirement: a locale the translation module does not know
    // still has to appear, or the store would silently support fewer locales than it does.
    let known = ctx
        .translation_service
        .list_locales()
        .await
        .unwrap_or_default();
    let store_id = store.get("id").and_then(Value::as_str).unwrap_or_default();

    let expanded: Vec<Value> = codes
        .iter()
        .map(|code| {
            let found = known.iter().find(|l| &l.code == code);
            let locale = match found {
                Some(l) => json!(l),
                None => json!({ "code": code, "name": code }),
            };
            json!({
                "id": format!("stloc_{store_id}_{code}"),
                "store_id": store_id,
                "locale_code": code,
                "locale": locale,
            })
        })
        .collect();

    json!(expanded)
}

/// Serialises a store the way the admin contract describes it.
async fn store_response(ctx: &AdminStoreContext, store: impl serde::Serialize) -> Value {
    let mut value = serde_json::to_value(store).unwrap_or_else(|_| json!({}));
    let locales = expand_supported_locales(ctx, &value).await;
    if let Some(map) = value.as_object_mut() {
        map.insert("supported_locales".to_string(), locales);
    }
    value
}

/// GET /admin/stores
pub async fn list_stores(
    State(ctx): State<AdminStoreContext>,
) -> Result<impl IntoResponse, HttpError> {
    let store_res = ctx.store_service.get_default_store().await;
    let store = match store_res {
        Ok(s) => s,
        Err(_) => {
            ctx.store_service
                .create_store(CreateStoreInput {
                    name: "Stheno Store".to_string(),
                    default_sales_channel_id: Some("sc_default".to_string()),
                    default_region_id: Some("reg_01".to_string()),
                    default_location_id: Some("loc_main".to_string()),
                    supported_currencies: Some(vec!["usd".to_string(), "eur".to_string()]),
                    default_currency_code: Some("usd".to_string()),
                    supported_locales: Some(vec!["en".to_string()]),
                    default_locale: Some("en".to_string()),
                })
                .await?
        }
    };

    let store = store_response(&ctx, store).await;
    Ok((
        StatusCode::OK,
        Json(json!({
            "stores": [store],
            "data": [store],
            "count": 1,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

/// GET /admin/stores/{id}
pub async fn get_store(
    State(ctx): State<AdminStoreContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let store = ctx.store_service.get_store(&id).await?;
    let store = store_response(&ctx, store).await;
    Ok((
        StatusCode::OK,
        Json(json!({
            "store": store,
            "data": store,
        })),
    ))
}

/// POST /admin/stores/{id}
pub async fn update_store(
    State(ctx): State<AdminStoreContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStoreInput>,
) -> Result<impl IntoResponse, HttpError> {
    let store = ctx.store_service.update_store(&id, payload).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "store": store,
            "data": store,
        })),
    ))
}

/// GET /admin/notifications
pub async fn list_notifications() -> Result<impl IntoResponse, HttpError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "notifications": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}
