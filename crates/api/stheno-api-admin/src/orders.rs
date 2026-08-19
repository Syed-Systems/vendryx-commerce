//! Admin Orders, Fulfillments, Payments, and Returns API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_fulfillment::{CreateFulfillmentInput, FulfillmentService};
use stheno_order::{
    CreateOrderInput, CreateReturnInput, CreateReturnReasonInput, OrderFilter, OrderService,
    UpdateOrderInput,
};
use stheno_payment::{CapturePaymentInput, PaymentService, RefundPaymentInput};

#[derive(Clone)]
pub struct AdminOrderContext {
    pub order_service: Arc<OrderService>,
    pub fulfillment_service: Arc<FulfillmentService>,
    pub payment_service: Arc<PaymentService>,
}

// 1. Order Handlers
pub async fn list_orders(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let (all, count) = ctx
        .order_service
        .list_orders(OrderFilter::default())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("unexpected_state", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "orders": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_order(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateOrderInput>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx.order_service.create_order(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "order": order,
            "data": order,
        })),
    ))
}

pub async fn get_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match ctx.order_service.get_order(&id).await {
        Ok(order) => Ok((
            StatusCode::OK,
            Json(json!({
                "order": order,
                "data": order,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Order with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn cancel_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx.order_service.cancel_order(&id).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_action", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "order": order,
            "data": order,
        })),
    ))
}

pub async fn archive_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx.order_service.archive_order(&id).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_action", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "order": order,
            "data": order,
        })),
    ))
}

// 2. Fulfillment Handlers
pub async fn create_fulfillment(
    State(ctx): State<AdminOrderContext>,
    Path(order_id): Path<String>,
    Json(input): Json<CreateFulfillmentInput>,
) -> Result<impl IntoResponse, Response> {
    let ful = ctx
        .fulfillment_service
        .create_fulfillment(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "invalid_fulfillment_data",
                    &e.to_string(),
                )),
            )
                .into_response()
        })?;

    let _ = ctx
        .order_service
        .update_order(
            &order_id,
            UpdateOrderInput {
                fulfillment_status: Some("fulfilled".to_string()),
                ..Default::default()
            },
        )
        .await;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "fulfillment": ful,
            "data": ful,
        })),
    ))
}

// 3. Payment Handlers
pub async fn capture_payment(
    State(ctx): State<AdminOrderContext>,
    Path(_order_id): Path<String>,
    Json(input): Json<CapturePaymentInput>,
) -> Result<impl IntoResponse, Response> {
    let cap = ctx
        .payment_service
        .capture_payment(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("payment_capture_failed", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "capture": cap,
            "data": cap,
        })),
    ))
}

pub async fn refund_payment(
    State(ctx): State<AdminOrderContext>,
    Path(_order_id): Path<String>,
    Json(input): Json<RefundPaymentInput>,
) -> Result<impl IntoResponse, Response> {
    let ref_dto = ctx
        .payment_service
        .refund_payment(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("payment_refund_failed", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "refund": ref_dto,
            "data": ref_dto,
        })),
    ))
}

// 4. Return Handlers
pub async fn list_returns(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let returns = ctx.order_service.list_returns(None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = returns.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "returns": returns,
            "data": returns,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_return(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateReturnInput>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx.order_service.create_return(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_return_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "return": ret,
            "data": ret,
        })),
    ))
}

pub async fn receive_return(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx
        .order_service
        .receive_return_items(&id, vec![])
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("receive_failed", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "return": ret,
            "data": ret,
        })),
    ))
}

// 5. Return Reasons Handlers
pub async fn list_return_reasons(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let reasons = ctx.order_service.list_return_reasons().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = reasons.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "return_reasons": reasons,
            "data": reasons,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_return_reason(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateReturnReasonInput>,
) -> Result<impl IntoResponse, Response> {
    let reason = ctx
        .order_service
        .create_return_reason(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_reason_data", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "return_reason": reason,
            "data": reason,
        })),
    ))
}

pub async fn get_return_reason(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let reasons = ctx.order_service.list_return_reasons().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    if let Some(r) = reasons.into_iter().find(|item| item.id == id) {
        Ok((
            StatusCode::OK,
            Json(json!({
                "return_reason": r,
                "data": r,
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Return reason {} not found", id),
            )),
        )
            .into_response())
    }
}

pub async fn delete_return_reason(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "return_reason",
            "deleted": true
        })),
    ))
}

// 6. Refund Reasons Handlers
pub async fn list_refund_reasons(
    State(_ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let reasons = vec![
        json!({
            "id": "refr_01default",
            "label": "Customer return",
            "description": "Item returned in acceptable condition"
        }),
        json!({
            "id": "refr_02damaged",
            "label": "Item damaged or defective",
            "description": "Item arrived broken or malfunctioning"
        }),
    ];
    let count = reasons.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "refund_reasons": reasons,
            "data": reasons,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_refund_reason(
    State(_ctx): State<AdminOrderContext>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let mut obj = input;
    if obj.get("id").is_none() {
        if let Some(map) = obj.as_object_mut() {
            map.insert(
                "id".to_string(),
                json!(format!("refr_{}", ulid::Ulid::new())),
            );
        }
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "refund_reason": obj,
            "data": obj,
        })),
    ))
}

pub async fn get_refund_reason(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let reason = json!({
        "id": id,
        "label": "General refund",
        "description": "Standard store refund policy"
    });
    Ok((
        StatusCode::OK,
        Json(json!({
            "refund_reason": reason,
            "data": reason,
        })),
    ))
}

pub async fn delete_refund_reason(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "refund_reason",
            "deleted": true
        })),
    ))
}

pub async fn list_order_edits() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "order_edits": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn list_claims() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "claims": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn list_exchanges() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "exchanges": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}
