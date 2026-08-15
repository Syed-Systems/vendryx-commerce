//! Admin Inventory, Stock Locations and Reservations API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};
use stheno_inventory::{
    CreateInventoryItemInput, CreateReservationInput, InventoryItemDTO, InventoryService,
    ReservationItemDTO,
};
use stheno_stock_location::{CreateStockLocationInput, StockLocationDTO, StockLocationService};

#[derive(Clone)]
pub struct InventoryContext {
    pub inventory_service: Arc<InventoryService>,
    pub stock_location_service: Arc<StockLocationService>,
}

// 1. Inventory Items Handlers
pub async fn list_inventory_items(
    State(_ctx): State<InventoryContext>,
) -> Result<Json<ListResponse<Vec<InventoryItemDTO>>>, Response> {
    // Return empty list/mock list since no list exists in stheno-inventory, or mock it
    Ok(Json(ListResponse {
        data: vec![],
        count: 0,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_inventory_item(
    State(ctx): State<InventoryContext>,
    Json(input): Json<CreateInventoryItemInput>,
) -> Result<(StatusCode, Json<GetResponse<InventoryItemDTO>>), Response> {
    let item = ctx.inventory_service.create_item(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: item })))
}

// 2. Reservations Handlers
pub async fn list_reservations(
    State(_ctx): State<InventoryContext>,
) -> Result<Json<ListResponse<Vec<ReservationItemDTO>>>, Response> {
    Ok(Json(ListResponse {
        data: vec![],
        count: 0,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_reservation(
    State(ctx): State<InventoryContext>,
    Json(input): Json<CreateReservationInput>,
) -> Result<(StatusCode, Json<GetResponse<ReservationItemDTO>>), Response> {
    let res = ctx.inventory_service.reserve(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: res })))
}

// 3. Stock Locations Handlers
pub async fn list_stock_locations(
    State(ctx): State<InventoryContext>,
) -> Result<Json<ListResponse<Vec<StockLocationDTO>>>, Response> {
    let all = ctx.stock_location_service.list_locations().await.map_err(|e| {
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

pub async fn create_stock_location(
    State(ctx): State<InventoryContext>,
    Json(input): Json<CreateStockLocationInput>,
) -> Result<(StatusCode, Json<GetResponse<StockLocationDTO>>), Response> {
    let loc = ctx.stock_location_service.create_location(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: loc })))
}

pub async fn delete_stock_location(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    ctx.stock_location_service.delete_location(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(DeleteResponse {
        id,
        object: "stock_location".to_string(),
        deleted: true,
    }))
}
