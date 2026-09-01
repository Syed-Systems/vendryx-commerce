//! Admin Pricing API Handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_http::HttpError;
use stheno_pricing::{CreatePriceListInput, PricingService, UpdatePriceListInput};

// Price List Handlers
pub async fn list_price_lists(
    State(service): State<Arc<PricingService>>,
) -> Result<impl IntoResponse, Response> {
    let all = service.list_price_lists().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "price_lists": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_price_list(
    State(service): State<Arc<PricingService>>,
    Json(input): Json<CreatePriceListInput>,
) -> Result<impl IntoResponse, Response> {
    let list = service.create_price_list(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "price_list": list,
            "data": list,
        })),
    ))
}

pub async fn get_price_list(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_price_list(&id).await {
        Ok(list) => Ok((
            StatusCode::OK,
            Json(json!({
                "price_list": list,
                "data": list,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Price list with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_price_list(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePriceListInput>,
) -> Result<impl IntoResponse, Response> {
    let list = service.update_price_list(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "price_list": list,
            "data": list,
        })),
    ))
}

pub async fn delete_price_list(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_price_list(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "price_list",
            "deleted": true
        })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct SetPricePreferenceRequest {
    pub attribute: String,
    pub value: String,
    pub is_tax_inclusive: Option<bool>,
}

/// GET /admin/price-preferences
///
/// Moved here from `store.rs`, which answered from an empty literal because
/// `AdminStoreContext` carries no `PricingService`. The preference table has been written
/// to since P12 and never read.
pub async fn list_price_preferences(
    State(service): State<Arc<PricingService>>,
) -> Result<impl IntoResponse, HttpError> {
    let preferences = service.list_price_preferences().await?;
    let count = preferences.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "price_preferences": preferences,
            "data": preferences,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

/// POST /admin/price-preferences
///
/// Upsert, matching the service: a preference is identified by `(attribute, value)`, and
/// setting one that exists changes its tax inclusivity rather than adding a second row.
pub async fn set_price_preference(
    State(service): State<Arc<PricingService>>,
    Json(request): Json<SetPricePreferenceRequest>,
) -> Result<impl IntoResponse, HttpError> {
    service
        .set_price_preference(
            &request.attribute,
            &request.value,
            request.is_tax_inclusive.unwrap_or(false),
        )
        .await?;

    let preference = service
        .list_price_preferences()
        .await?
        .into_iter()
        .find(|p| p.attribute == request.attribute && p.value == request.value);

    Ok((
        StatusCode::OK,
        Json(json!({ "price_preference": preference, "data": preference })),
    ))
}

/// GET /admin/price-preferences/{id}
pub async fn get_price_preference(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let preferences = service.list_price_preferences().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let pref = preferences
        .into_iter()
        .find(|p| p.id.to_string() == id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    "not_found",
                    &format!("Price preference {} not found", id),
                )),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "price_preference": pref, "data": pref })),
    ))
}

/// POST /admin/price-preferences/{id}
pub async fn update_price_preference(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
    Json(request): Json<SetPricePreferenceRequest>,
) -> Result<impl IntoResponse, Response> {
    service
        .set_price_preference(
            &request.attribute,
            &request.value,
            request.is_tax_inclusive.unwrap_or(false),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;
    let preferences = service.list_price_preferences().await.unwrap_or_default();
    let pref = preferences.into_iter().find(|p| {
        p.id.to_string() == id
            || (p.attribute == request.attribute && p.value == request.value)
    });
    Ok((
        StatusCode::OK,
        Json(json!({ "price_preference": pref, "data": pref })),
    ))
}

/// DELETE /admin/price-preferences/{id}
pub async fn delete_price_preference(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service
        .delete_price_preference_by_id(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("not_found", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "price_preference",
            "deleted": true
        })),
    ))
}

/// GET /admin/price-lists/{id}/prices
pub async fn list_price_list_prices(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let _list = service.get_price_list(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    let prices = service
        .list_prices_for_price_list(&id)
        .await
        .unwrap_or_default();
    let count = prices.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "prices": prices,
            "data": prices,
            "count": count,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct BatchPriceListPricesRequest {
    pub create: Option<Vec<stheno_pricing::CreatePriceInput>>,
    pub delete: Option<Vec<String>>,
}

/// POST /admin/price-lists/{id}/prices/batch
pub async fn batch_price_list_prices(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
    Json(body): Json<BatchPriceListPricesRequest>,
) -> Result<impl IntoResponse, Response> {
    let _list = service.get_price_list(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    service
        .batch_price_list_prices(
            &id,
            stheno_pricing::BatchPriceListPricesInput {
                create: body.create,
                delete: body.delete,
            },
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true
        })),
    ))
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct BatchPriceListProductsRequest {
    pub remove: Option<Vec<String>>,
}

/// POST /admin/price-lists/{id}/products
pub async fn update_price_list_products(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
    Json(_body): Json<BatchPriceListProductsRequest>,
) -> Result<impl IntoResponse, Response> {
    let list = service.get_price_list(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "price_list": list,
            "data": list,
        })),
    ))
}
