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
    CreateInventoryItemInput, CreateInventoryLevelInput, CreateReservationInput, InventoryService,
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
fn internal(e: &stheno_types::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("unexpected_state", &e.to_string())),
    )
        .into_response()
}

fn not_found(e: &stheno_types::Error) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("not_found", &e.to_string())),
    )
        .into_response()
}

fn bad_request(e: &stheno_types::Error) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("invalid_data", &e.to_string())),
    )
        .into_response()
}

pub async fn list_inventory_items(
    State(ctx): State<InventoryContext>,
) -> Result<impl IntoResponse, Response> {
    let items = ctx
        .inventory_service
        .list_items()
        .await
        .map_err(|e| internal(&e))?;
    let count = items.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "inventory_items": items, "data": items,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_inventory_item(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let item = ctx
        .inventory_service
        .get_item(&id)
        .await
        .map_err(|e| not_found(&e))?;
    let levels = ctx
        .inventory_service
        .list_levels(&id)
        .await
        .map_err(|e| internal(&e))?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "inventory_item": item, "data": item, "location_levels": levels,
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
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
    Json(payload): Json<CreateLocationLevelPayload>,
) -> Result<impl IntoResponse, Response> {
    // This previously built a plausible-looking level in JSON — with its own invented id
    // prefix — and stored nothing. The admin showed stock that did not exist, and the
    // very next read returned an empty list. `create_level` was there the whole time.
    let level = ctx
        .inventory_service
        .create_level(CreateInventoryLevelInput {
            inventory_item_id: id.clone(),
            location_id: payload.location_id,
            stocked_quantity: payload.stocked_quantity.unwrap_or(0) as i32,
        })
        .await
        .map_err(|e| bad_request(&e))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "inventory_level": level,
            "data": level,
            "inventory_item": { "id": id, "location_levels": [level] },
        })),
    ))
}

// 2. Reservations Handlers
#[derive(serde::Deserialize)]
pub struct ReservationQuery {
    pub inventory_item_id: Option<String>,
}

pub async fn list_reservations(
    State(ctx): State<InventoryContext>,
    axum::extract::Query(query): axum::extract::Query<ReservationQuery>,
) -> Result<impl IntoResponse, Response> {
    let reservations = ctx
        .inventory_service
        .list_reservations(query.inventory_item_id.as_deref())
        .await
        .map_err(|e| internal(&e))?;
    let count = reservations.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "reservations": reservations, "data": reservations,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_reservation(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let reservation = ctx
        .inventory_service
        .get_reservation(&id)
        .await
        .map_err(|e| not_found(&e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "reservation": reservation, "data": reservation })),
    ))
}

pub async fn delete_reservation(
    State(ctx): State<InventoryContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    // Releasing returns the quantity to available stock; deleting the row without
    // releasing would strand the reservation and permanently lose the stock.
    ctx.inventory_service
        .release_reservation(&id)
        .await
        .map_err(|e| not_found(&e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "reservation", "deleted": true })),
    ))
}

#[derive(serde::Deserialize)]
pub struct SetStockRequest {
    pub stocked_quantity: i32,
}

/// `POST /admin/inventory-items/{id}/location-levels/{location_id}` — stock adjustment.
///
/// A receipt, a stock count, or shrinkage. There was previously no way to change stock
/// after a level was created, which left the admin's inventory screens read-only against
/// a number nothing could move.
pub async fn set_stock_level(
    State(ctx): State<InventoryContext>,
    Path((id, location_id)): Path<(String, String)>,
    Json(body): Json<SetStockRequest>,
) -> Result<impl IntoResponse, Response> {
    let level = ctx
        .inventory_service
        .set_stocked_quantity(&id, &location_id, body.stocked_quantity)
        .await
        // Refusing to drop stock below what is reserved is a 400, not a missing record.
        .map_err(|e| bad_request(&e))?;
    Ok((
        StatusCode::OK,
        Json(json!({ "inventory_level": level, "data": level })),
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
