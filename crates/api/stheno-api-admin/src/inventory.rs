//! Admin Inventory, Stock Locations and Reservations API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_inventory::{
    CreateInventoryItemInput, CreateReservationInput, InventoryService,
};
use stheno_stock_location::{CreateStockLocationInput, StockLocationService};

#[derive(Clone)]
pub struct InventoryContext {
    pub inventory_service: Arc<InventoryService>,
    pub stock_location_service: Arc<StockLocationService>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLocationLevelPayload {
    pub location_id: String,
    pub stocked_quantity: Option<i64>,
    pub incoming_quantity: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStockLocationPayload {
    pub name: Option<String>,
    pub address: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

// 1. Inventory Items Handlers
pub async fn list_inventory_items(
    State(_ctx): State<InventoryContext>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "inventory_items": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn get_inventory_item(
    State(_ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let mock_item = json!({
        "id": id,
        "sku": "SKU-DEFAULT",
        "origin_country": null,
        "hs_code": null,
        "requires_shipping": true,
        "location_levels": []
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "inventory_item": mock_item,
            "data": mock_item,
        })),
    ))
}

pub async fn create_inventory_item(
    State(ctx): State<InventoryContext>,
    Json(input): Json<CreateInventoryItemInput>,
) -> Result<impl IntoResponse, Response> {
    let item = ctx
        .inventory_service
        .create_item(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "inventory_item": item,
            "data": item,
        })),
    ))
}

pub async fn create_location_level(
    State(_ctx): State<InventoryContext>,
    Path(id): Path<String>,
    Json(payload): Json<CreateLocationLevelPayload>,
) -> Result<impl IntoResponse, Response> {
    let location_level = json!({
        "id": format!("i_lvl_{}", ulid::Ulid::new()),
        "inventory_item_id": id,
        "location_id": payload.location_id,
        "stocked_quantity": payload.stocked_quantity.unwrap_or(0),
        "reserved_quantity": 0,
        "incoming_quantity": payload.incoming_quantity.unwrap_or(0),
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "inventory_item": {
                "id": id,
                "location_levels": [location_level]
            }
        })),
    ))
}

// 2. Reservations Handlers
pub async fn list_reservations(
    State(_ctx): State<InventoryContext>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "reservations": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn create_reservation(
    State(ctx): State<InventoryContext>,
    Json(input): Json<CreateReservationInput>,
) -> Result<impl IntoResponse, Response> {
    let res = ctx.inventory_service.reserve(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "reservation": res,
            "data": res,
        })),
    ))
}

// 3. Stock Locations Handlers
pub async fn list_stock_locations(
    State(ctx): State<InventoryContext>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx
        .stock_location_service
        .list_locations()
        .await
        .map_err(|e| {
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
            "stock_locations": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn get_stock_location(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx
        .stock_location_service
        .list_locations()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("unexpected_state", &e.to_string())),
            )
                .into_response()
        })?;

    if let Some(loc) = all.into_iter().find(|l| l.id.to_string() == id) {
        Ok((
            StatusCode::OK,
            Json(json!({
                "stock_location": loc,
                "data": loc,
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Stock location {} not found", id),
            )),
        )
            .into_response())
    }
}

pub async fn create_stock_location(
    State(ctx): State<InventoryContext>,
    Json(input): Json<CreateStockLocationInput>,
) -> Result<impl IntoResponse, Response> {
    let loc = ctx
        .stock_location_service
        .create_location(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "stock_location": loc,
            "data": loc,
        })),
    ))
}

pub async fn update_stock_location(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStockLocationPayload>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx
        .stock_location_service
        .list_locations()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("unexpected_state", &e.to_string())),
            )
                .into_response()
        })?;

    if let Some(mut loc) = all.into_iter().find(|l| l.id.to_string() == id) {
        if let Some(name) = payload.name {
            loc.name = name;
        }
        Ok((
            StatusCode::OK,
            Json(json!({
                "stock_location": loc,
                "data": loc,
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Stock location {} not found", id),
            )),
        )
            .into_response())
    }
}

pub async fn delete_stock_location(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.stock_location_service
        .delete_location(&id)
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
            "object": "stock_location",
            "deleted": true
        })),
    ))
}
