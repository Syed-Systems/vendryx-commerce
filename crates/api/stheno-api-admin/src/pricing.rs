//! Admin Pricing API Handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};
use stheno_pricing::{CreatePriceListInput, PriceListDTO, PricingService, UpdatePriceListInput};

// Price List Handlers
pub async fn list_price_lists(
    State(service): State<Arc<PricingService>>,
) -> Result<Json<ListResponse<Vec<PriceListDTO>>>, Response> {
    let all = service.list_price_lists().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse {
        data: all,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_price_list(
    State(service): State<Arc<PricingService>>,
    Json(input): Json<CreatePriceListInput>,
) -> Result<(StatusCode, Json<GetResponse<PriceListDTO>>), Response> {
    let list = service.create_price_list(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: list })))
}

pub async fn get_price_list(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<PriceListDTO>>, Response> {
    match service.get_price_list(&id).await {
        Ok(list) => Ok(Json(GetResponse { data: list })),
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
) -> Result<Json<GetResponse<PriceListDTO>>, Response> {
    let list = service.update_price_list(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: list }))
}

pub async fn delete_price_list(
    State(service): State<Arc<PricingService>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    service.delete_price_list(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(DeleteResponse {
        id,
        object: "price_list".to_string(),
        deleted: true,
    }))
}
