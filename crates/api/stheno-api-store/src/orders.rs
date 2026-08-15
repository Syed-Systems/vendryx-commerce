//! Store Order API handlers (TASKS P5-22, F6, F20).
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_flows::OrderWorkflowEngine;
use stheno_order::{OrderDTO, OrderFilter, OrderService};
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreOrderContext {
    pub order_service: Arc<OrderService>,
    pub order_flows: Arc<OrderWorkflowEngine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListOrdersQuery {
    pub display_id: Option<i64>,
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub order: OrderDTO,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderDTO>,
    pub count: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestTransferRequest {
    pub order_id: String,
    pub target_customer_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransferActionRequest {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferResponse {
    pub order_id: String,
    pub status: String,
}

/// `GET /store/orders`
pub async fn list_orders(
    State(ctx): State<StoreOrderContext>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<OrderListResponse>, Response> {
    let filter = OrderFilter {
        display_id: query.display_id,
        status: query.status,
        limit: query.limit,
        offset: query.offset,
        ..Default::default()
    };

    let (orders, count) = ctx
        .order_service
        .list_orders(filter)
        .await
        .map_err(map_error)?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    Ok(Json(OrderListResponse {
        orders,
        count,
        limit,
        offset,
    }))
}

/// `GET /store/orders/{id}`
pub async fn get_order(
    State(ctx): State<StoreOrderContext>,
    Path(id): Path<String>,
) -> Result<Json<OrderResponse>, Response> {
    let order = ctx.order_service.get_order(&id).await.map_err(map_error)?;
    Ok(Json(OrderResponse { order }))
}

/// `POST /store/orders/transfer`
pub async fn request_transfer(
    State(ctx): State<StoreOrderContext>,
    Json(body): Json<RequestTransferRequest>,
) -> Result<Json<TransferResponse>, Response> {
    let req = ctx
        .order_flows
        .request_order_transfer(&body.order_id, &body.target_customer_id)
        .await
        .map_err(map_error)?;

    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

/// `POST /store/orders/transfer/accept`
pub async fn accept_transfer(
    State(ctx): State<StoreOrderContext>,
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<OrderResponse>, Response> {
    let order = ctx
        .order_flows
        .accept_order_transfer(&body.order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(OrderResponse { order }))
}

/// `POST /store/orders/transfer/decline`
pub async fn decline_transfer(
    State(ctx): State<StoreOrderContext>,
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<TransferResponse>, Response> {
    let req = ctx
        .order_flows
        .decline_order_transfer(&body.order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

/// `POST /store/orders/transfer/cancel`
pub async fn cancel_transfer(
    State(ctx): State<StoreOrderContext>,
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<TransferResponse>, Response> {
    let req = ctx
        .order_flows
        .cancel_order_transfer(&body.order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

fn map_error(e: Error) -> Response {
    let (status, r#type) = match e {
        Error::NotFound { .. } => (StatusCode::NOT_FOUND, "not_found"),
        Error::Conflict { .. } => (StatusCode::CONFLICT, "conflict"),
        Error::Invalid { .. } => (StatusCode::BAD_REQUEST, "invalid_data"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state"),
    };

    (
        status,
        Json(ErrorResponse {
            r#type: r#type.to_string(),
            message: e.to_string(),
        }),
    )
        .into_response()
}
