//! Admin Store, Price Preferences & Notifications API Handlers (P12-12).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_http::HttpError;
use stheno_store::dto::{CreateStoreInput, UpdateStoreInput};
use stheno_store::StoreService;

#[derive(Clone)]
pub struct AdminStoreContext {
    pub store_service: Arc<StoreService>,
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

/// GET /admin/price-preferences
pub async fn list_price_preferences() -> Result<impl IntoResponse, HttpError> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "price_preferences": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
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
