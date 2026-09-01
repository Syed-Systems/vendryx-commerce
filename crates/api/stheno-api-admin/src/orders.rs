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
    CreateClaimInput, CreateExchangeInput, CreateOrderChangeInput, CreateOrderInput,
    CreateReturnInput, CreateReturnReasonInput, OrderFilter, OrderService, UpdateOrderInput,
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
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.order_service
        .delete_return_reason(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "return_reason", "deleted": true })),
    ))
}

// 6. Refund Reasons Handlers
pub async fn list_refund_reasons(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let reasons = ctx
        .payment_service
        .list_refund_reasons()
        .await
        .map_err(internal)?;
    let count = reasons.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "refund_reasons": reasons,
            "data": reasons,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

pub async fn get_refund_reason(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let reason = ctx
        .payment_service
        .get_refund_reason(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "refund_reason": reason, "data": reason })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateRefundReasonRequest {
    pub label: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn create_refund_reason(
    State(ctx): State<AdminOrderContext>,
    Json(request): Json<CreateRefundReasonRequest>,
) -> Result<impl IntoResponse, Response> {
    let id = ctx
        .payment_service
        .create_refund_reason(&request.label, request.description, request.metadata)
        .await
        .map_err(bad_request)?;
    let reason = ctx
        .payment_service
        .get_refund_reason(&id)
        .await
        .map_err(internal)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "refund_reason": reason, "data": reason })),
    ))
}

/// POST /admin/refund-reasons/{id}
pub async fn update_refund_reason(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(input): Json<stheno_payment::UpdateRefundReasonInput>,
) -> Result<impl IntoResponse, Response> {
    let reason = ctx
        .payment_service
        .update_refund_reason(&id, input)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "refund_reason": reason, "data": reason })),
    ))
}

pub async fn delete_refund_reason(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.payment_service
        .delete_refund_reason_by_id(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "refund_reason", "deleted": true })),
    ))
}

pub async fn list_order_edits(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let edits = ctx
        .order_service
        .list_order_changes(None)
        .await
        .map_err(internal)?;
    let count = edits.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "order_edits": edits, "data": edits, "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn list_claims(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let claims = ctx.order_service.list_claims(None).await.map_err(internal)?;
    let count = claims.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "claims": claims, "data": claims, "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn list_exchanges(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let exchanges = ctx
        .order_service
        .list_exchanges(None)
        .await
        .map_err(internal)?;
    let count = exchanges.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "exchanges": exchanges, "data": exchanges, "count": count, "offset": 0, "limit": count,
        })),
    ))
}

/// POST /admin/orders/{id}
///
/// `UpdateOrderInput` and `OrderService::update_order` have both existed since P13; the
/// route was mounted `get`-only, so every save from the order detail screen — email,
/// addresses, metadata — was rejected by the method router before a handler ran.
pub async fn update_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateOrderInput>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .update_order(&id, input)
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
            "order": order,
            "data": order,
        })),
    ))
}

/// POST /admin/claims
pub async fn create_claim(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateClaimInput>,
) -> Result<impl IntoResponse, Response> {
    let claim = ctx.order_service.create_claim(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "claim": claim, "data": claim })),
    ))
}

/// POST /admin/exchanges
pub async fn create_exchange(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateExchangeInput>,
) -> Result<impl IntoResponse, Response> {
    let exchange = ctx.order_service.create_exchange(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "exchange": exchange, "data": exchange })),
    ))
}

/// The dashboard's `POST /admin/order-edits` body.
///
/// Medusa sends `{ order_id, description?, internal_note? }` and no discriminator, because
/// the resource *is* the edit. `CreateOrderChangeInput` is the general order-change type
/// and requires one, so the wire shape and the domain shape differ by exactly this struct
/// rather than by a loosened service input.
#[derive(Debug, serde::Deserialize)]
pub struct CreateOrderEditRequest {
    pub order_id: String,
    pub description: Option<String>,
    pub internal_note: Option<String>,
}

/// POST /admin/order-edits
pub async fn create_order_edit(
    State(ctx): State<AdminOrderContext>,
    Json(request): Json<CreateOrderEditRequest>,
) -> Result<impl IntoResponse, Response> {
    let mut metadata = serde_json::Map::new();
    if let Some(description) = request.description {
        metadata.insert("description".to_string(), json!(description));
    }
    if let Some(note) = request.internal_note {
        metadata.insert("internal_note".to_string(), json!(note));
    }

    let change = ctx
        .order_service
        .create_order_change(CreateOrderChangeInput {
            order_id: request.order_id,
            change_type: "edit".to_string(),
            requested_by: None,
            metadata: (!metadata.is_empty()).then_some(serde_json::Value::Object(metadata)),
        })
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
        Json(json!({ "order_change": change, "order_edit": change, "data": change })),
    ))
}

/// POST /admin/return-reasons/{id}
pub async fn update_return_reason(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(input): Json<stheno_order::UpdateReturnReasonInput>,
) -> Result<impl IntoResponse, Response> {
    let reason = ctx
        .order_service
        .update_return_reason(&id, input)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "return_reason": reason, "data": reason })),
    ))
}

/// DELETE /admin/returns/{id}/receive — reverses a receival.
pub async fn cancel_return_receipt(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx
        .order_service
        .cancel_return_receipt(&id)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": ret, "data": ret })),
    ))
}

fn internal(e: stheno_types::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("unexpected_state", &e.to_string())),
    )
        .into_response()
}

fn not_found(e: stheno_types::Error) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("not_found", &e.to_string())),
    )
        .into_response()
}

fn bad_request(e: stheno_types::Error) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("invalid_data", &e.to_string())),
    )
        .into_response()
}

/// The dashboard's `POST /admin/payment-collections` body.
///
/// Medusa sends only `order_id`: a payment collection covers what an order still owes, so
/// the amount is derived rather than supplied. `amount` is accepted for the partial-payment
/// case the order screen offers.
#[derive(Debug, serde::Deserialize)]
pub struct CreatePaymentCollectionRequest {
    pub order_id: String,
    pub amount: Option<rust_decimal::Decimal>,
}

/// GET /admin/payment-collections
pub async fn list_payment_collections(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let collections = ctx
        .payment_service
        .list_payment_collections()
        .await
        .map_err(internal)?;
    let count = collections.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "payment_collections": collections,
            "data": collections,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

/// POST /admin/payment-collections
///
/// Mounted on the orders router rather than the config one: deriving the amount needs the
/// order, and `ConfigContext` holds no `OrderService`. The currency always comes from the
/// order — taking it from the request would let a collection be raised in a currency the
/// order was never priced in.
pub async fn create_payment_collection(
    State(ctx): State<AdminOrderContext>,
    Json(request): Json<CreatePaymentCollectionRequest>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .get_order(&request.order_id)
        .await
        .map_err(not_found)?;

    // What the order still owes, unless the caller asked for a specific amount.
    let amount = match request.amount {
        Some(amount) => amount,
        None => order
            .summary
            .as_ref()
            .map(|s| s.pending_difference)
            .unwrap_or_default(),
    };

    let collection = ctx
        .payment_service
        .create_payment_collection(stheno_payment::CreatePaymentCollectionInput {
            currency_code: order.currency_code.clone(),
            amount,
            metadata: Some(json!({ "order_id": order.id })),
        })
        .await
        .map_err(bad_request)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "payment_collection": collection, "data": collection })),
    ))
}
