//! Admin Orders, Fulfillments, Payments, and Returns API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use stheno_api_core::{ErrorResponse, GetResponse, ListResponse};
use stheno_fulfillment::{CreateFulfillmentInput, FulfillmentDTO, FulfillmentService};
use stheno_order::{
    CreateOrderInput, CreateReturnInput, CreateReturnReasonInput, OrderDTO, OrderFilter,
    OrderService, ReturnDTO, ReturnReasonDTO, UpdateOrderInput,
};
use stheno_payment::{
    CaptureDTO, CapturePaymentInput, PaymentService, RefundDTO, RefundPaymentInput,
};

#[derive(Clone)]
pub struct AdminOrderContext {
    pub order_service: Arc<OrderService>,
    pub fulfillment_service: Arc<FulfillmentService>,
    pub payment_service: Arc<PaymentService>,
}

// 1. Order Handlers
pub async fn list_orders(
    State(ctx): State<AdminOrderContext>,
) -> Result<Json<ListResponse<Vec<OrderDTO>>>, Response> {
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
    Ok(Json(ListResponse {
        data: all,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_order(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateOrderInput>,
) -> Result<(StatusCode, Json<GetResponse<OrderDTO>>), Response> {
    let order = ctx.order_service.create_order(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: order })))
}

pub async fn get_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<OrderDTO>>, Response> {
    match ctx.order_service.get_order(&id).await {
        Ok(order) => Ok(Json(GetResponse { data: order })),
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
) -> Result<Json<GetResponse<OrderDTO>>, Response> {
    let order = ctx.order_service.cancel_order(&id).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_action", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: order }))
}

pub async fn archive_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<OrderDTO>>, Response> {
    let order = ctx.order_service.archive_order(&id).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_action", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: order }))
}

// 2. Fulfillment Handlers
pub async fn create_fulfillment(
    State(ctx): State<AdminOrderContext>,
    Path(order_id): Path<String>,
    Json(input): Json<CreateFulfillmentInput>,
) -> Result<(StatusCode, Json<GetResponse<FulfillmentDTO>>), Response> {
    // 1. Create fulfillment
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

    // 2. Transition order fulfillment status to "fulfilled"
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

    Ok((StatusCode::CREATED, Json(GetResponse { data: ful })))
}

// 3. Payment Handlers
pub async fn capture_payment(
    State(ctx): State<AdminOrderContext>,
    Path(_order_id): Path<String>,
    Json(input): Json<CapturePaymentInput>,
) -> Result<(StatusCode, Json<GetResponse<CaptureDTO>>), Response> {
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
    Ok((StatusCode::CREATED, Json(GetResponse { data: cap })))
}

pub async fn refund_payment(
    State(ctx): State<AdminOrderContext>,
    Path(_order_id): Path<String>,
    Json(input): Json<RefundPaymentInput>,
) -> Result<(StatusCode, Json<GetResponse<RefundDTO>>), Response> {
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
    Ok((StatusCode::CREATED, Json(GetResponse { data: ref_dto })))
}

// 4. Return Handlers
pub async fn list_returns(
    State(ctx): State<AdminOrderContext>,
) -> Result<Json<ListResponse<Vec<ReturnDTO>>>, Response> {
    let returns = ctx.order_service.list_returns(None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = returns.len();
    Ok(Json(ListResponse {
        data: returns,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_return(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateReturnInput>,
) -> Result<(StatusCode, Json<GetResponse<ReturnDTO>>), Response> {
    let ret = ctx.order_service.create_return(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_return_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: ret })))
}

pub async fn receive_return(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<ReturnDTO>>, Response> {
    // receive the return
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
    Ok(Json(GetResponse { data: ret }))
}

// 5. Return Reasons Handlers
pub async fn list_return_reasons(
    State(ctx): State<AdminOrderContext>,
) -> Result<Json<ListResponse<Vec<ReturnReasonDTO>>>, Response> {
    let reasons = ctx.order_service.list_return_reasons().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = reasons.len();
    Ok(Json(ListResponse {
        data: reasons,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_return_reason(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateReturnReasonInput>,
) -> Result<(StatusCode, Json<GetResponse<ReturnReasonDTO>>), Response> {
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
    Ok((StatusCode::CREATED, Json(GetResponse { data: reason })))
}
