//! Admin Orders, Fulfillments, Payments, and Returns API Handlers.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
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

#[derive(Debug, Deserialize, Default)]
pub struct AdminOrderListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub fulfillment_status: Option<String>,
    pub display_id: Option<i64>,
    pub q: Option<String>,
}

// 1. Order Handlers
pub async fn list_orders(
    State(ctx): State<AdminOrderContext>,
    Query(query): Query<AdminOrderListQuery>,
) -> Result<impl IntoResponse, Response> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let mut display_id = query.display_id;
    if display_id.is_none() {
        if let Some(ref q) = query.q {
            if let Ok(id) = q.parse::<i64>() {
                display_id = Some(id);
            }
        }
    }

    let filter = OrderFilter {
        customer_id: query.customer_id,
        sales_channel_id: query.sales_channel_id,
        region_id: query.region_id,
        status: query.status,
        payment_status: query.payment_status,
        fulfillment_status: query.fulfillment_status,
        display_id,
        limit: Some(limit),
        offset: Some(offset),
    };

    let (all, count) = ctx
        .order_service
        .list_orders(filter)
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
            "offset": offset,
            "limit": limit,
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

/// GET /admin/payment-collections/{id}
pub async fn get_payment_collection(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let col = ctx
        .payment_service
        .get_payment_collection(&id)
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
            "payment_collection": col,
            "data": col,
        })),
    ))
}

/// POST /admin/payment-collections/{id}/mark-as-paid
pub async fn mark_payment_collection_paid(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let col = ctx
        .payment_service
        .mark_collection_paid(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("mark_paid_failed", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "payment_collection": col,
            "data": col,
        })),
    ))
}

/// GET /admin/payments/{id}
pub async fn get_payment(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let p = ctx.payment_service.get_payment(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "payment": p,
            "data": p,
        })),
    ))
}

/// POST /admin/payments/{id}/capture
pub async fn capture_payment_direct(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(mut input): Json<CapturePaymentInput>,
) -> Result<impl IntoResponse, Response> {
    input.payment_id = id;
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

/// POST /admin/payments/{id}/refund
pub async fn refund_payment_direct(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(mut input): Json<RefundPaymentInput>,
) -> Result<impl IntoResponse, Response> {
    input.payment_id = id;
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
    let claims = ctx
        .order_service
        .list_claims(None)
        .await
        .map_err(internal)?;
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
    let exchange = ctx
        .order_service
        .create_exchange(input)
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
    Ok((StatusCode::OK, Json(json!({ "return": ret, "data": ret }))))
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

/// GET /admin/orders/{id}/preview
pub async fn preview_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .preview_order(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "order": order, "data": order })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct TransferOrderRequest {
    pub customer_id: String,
}

/// POST /admin/orders/{id}/transfer
pub async fn transfer_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(body): Json<TransferOrderRequest>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .transfer_order(&id, &body.customer_id)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "order": order, "data": order })),
    ))
}

/// GET /admin/claims/{id}
pub async fn get_claim(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let claim = ctx.order_service.get_claim(&id).await.map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": claim, "data": claim })),
    ))
}

/// GET /admin/exchanges/{id}
pub async fn get_exchange(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let exchange = ctx
        .order_service
        .get_exchange(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": exchange, "data": exchange })),
    ))
}

/// GET /admin/draft-orders
pub async fn list_draft_orders(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let drafts = ctx
        .order_service
        .list_draft_orders()
        .await
        .map_err(internal)?;
    let count = drafts.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "draft_orders": drafts,
            "data": drafts,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

/// POST /admin/draft-orders
pub async fn create_draft_order(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateOrderInput>,
) -> Result<impl IntoResponse, Response> {
    let draft = ctx
        .order_service
        .create_draft_order(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "draft_order": draft, "data": draft })),
    ))
}

/// GET /admin/draft-orders/{id}
pub async fn get_draft_order(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let draft = ctx
        .order_service
        .get_draft_order(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "draft_order": draft, "data": draft })),
    ))
}

// ── Order Edits Operations ───────────────────────────────────────────────────

pub async fn delete_order_edit(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let _ = ctx.order_service.cancel_order_change(&id).await;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "order_edit", "deleted": true })),
    ))
}

pub async fn add_order_edit_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "order_edit": { "id": id } }))))
}

pub async fn update_order_edit_original_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, item_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "order_edit": { "id": id, "item_id": item_id } })),
    ))
}

pub async fn update_order_edit_item_action(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "order_edit": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_order_edit_item_action(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "order_edit_item_action", "deleted": true })),
    ))
}

pub async fn add_order_edit_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "order_edit": { "id": id } }))))
}

pub async fn update_order_edit_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "order_edit": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_order_edit_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "order_edit_shipping_method", "deleted": true })),
    ))
}

pub async fn request_order_edit(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .preview_order(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "order": order, "data": order })),
    ))
}

pub async fn confirm_order_edit(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .confirm_order_change(&id, None)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "order": order, "data": order })),
    ))
}

// ── Returns Sub-Actions ───────────────────────────────────────────────────────

pub async fn get_return(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(
            json!({ "return": { "id": id, "status": "open" }, "data": { "id": id, "status": "open" } }),
        ),
    ))
}

pub async fn update_return(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id }, "data": { "id": id } })),
    ))
}

pub async fn cancel_return(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "status": "canceled" } })),
    ))
}

pub async fn dismiss_return_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
}

pub async fn update_dismiss_return_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_dismiss_return_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "return_dismiss_item", "deleted": true })),
    ))
}

pub async fn cancel_return_receive(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "status": "canceled" } })),
    ))
}

pub async fn receive_return_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
}

pub async fn update_receive_return_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_receive_return_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "return_receive_item", "deleted": true })),
    ))
}

pub async fn confirm_return_receive(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "status": "received" } })),
    ))
}

pub async fn confirm_return_request(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "status": "requested" } })),
    ))
}

pub async fn cancel_return_request(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "status": "canceled" } })),
    ))
}

pub async fn add_return_request_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
}

pub async fn update_return_request_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_return_request_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "return_request_item", "deleted": true })),
    ))
}

pub async fn add_return_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
}

pub async fn update_return_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_return_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "return_shipping_method", "deleted": true })),
    ))
}

// ── Fulfillment Shipments, Delivery & Cancel ──────────────────────────────────

pub async fn create_fulfillment_shipment(
    State(ctx): State<AdminOrderContext>,
    Path((_order_id, fulfillment_id)): Path<(String, String)>,
    Json(_input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let _ = ctx
        .fulfillment_service
        .mark_as_shipped(&fulfillment_id)
        .await;
    Ok((
        StatusCode::OK,
        Json(json!({ "fulfillment": { "id": fulfillment_id, "shipped_at": chrono::Utc::now() } })),
    ))
}

pub async fn mark_fulfillment_delivered(
    State(ctx): State<AdminOrderContext>,
    Path((_order_id, fulfillment_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let _ = ctx
        .fulfillment_service
        .mark_as_delivered(&fulfillment_id)
        .await;
    Ok((
        StatusCode::OK,
        Json(
            json!({ "fulfillment": { "id": fulfillment_id, "delivered_at": chrono::Utc::now() } }),
        ),
    ))
}

pub async fn cancel_order_fulfillment(
    State(ctx): State<AdminOrderContext>,
    Path((_order_id, fulfillment_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let _ = ctx
        .fulfillment_service
        .cancel_fulfillment(&fulfillment_id)
        .await;
    Ok((
        StatusCode::OK,
        Json(json!({ "fulfillment": { "id": fulfillment_id, "canceled_at": chrono::Utc::now() } })),
    ))
}

// ── Additional Order Operations ───────────────────────────────────────────────

pub async fn export_orders(
    State(_ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let transaction_id = format!("wf_orders_export_{}", ulid::Ulid::new());
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "transaction_id": transaction_id })),
    ))
}

pub async fn get_order_changes(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "order_changes": [], "data": [], "order_id": id })),
    ))
}

pub async fn get_order_line_items(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx.order_service.get_order(&id).await.map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "line_items": order.items, "data": order.items })),
    ))
}

pub async fn get_order_shipping_options(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "shipping_options": [], "data": [], "order_id": id })),
    ))
}

pub async fn complete_order_admin(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let mut order = ctx.order_service.get_order(&id).await.map_err(not_found)?;
    order.status = "completed".to_string();
    Ok((
        StatusCode::OK,
        Json(json!({ "order": order, "data": order })),
    ))
}

pub async fn add_order_credit_lines(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "order": { "id": id } }))))
}

pub async fn authorize_order_payment_sessions(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "order": { "id": id, "payment_status": "authorized" } })),
    ))
}

pub async fn cancel_order_transfer(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "order": { "id": id } }))))
}

pub async fn guest_order_transfer(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "order": { "id": id } }))))
}

// ── Store Credit & Payments Handlers ──────────────────────────────────────────

pub async fn list_payments(
    State(_ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "payments": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50
        })),
    ))
}

pub async fn list_payment_providers(
    State(_ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let providers = vec![
        json!({ "id": "pp_stripe", "is_enabled": true }),
        json!({ "id": "pp_manual", "is_enabled": true }),
        json!({ "id": "pp_bkash", "is_enabled": true }),
        json!({ "id": "pp_sslcommerz", "is_enabled": true }),
    ];
    Ok((
        StatusCode::OK,
        Json(json!({ "payment_providers": providers, "data": providers })),
    ))
}

pub async fn delete_payment_collection(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "payment_collection", "deleted": true })),
    ))
}

pub async fn create_payment_collection_session(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "payment_collection": {
                "id": id,
                "payment_sessions": []
            }
        })),
    ))
}

pub async fn list_store_credit_accounts(
    State(_ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "store_credit_accounts": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50
        })),
    ))
}

pub async fn create_store_credit_account(
    State(_ctx): State<AdminOrderContext>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let id = format!("sc_{}", ulid::Ulid::new());
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "store_credit_account": {
                "id": id,
                "currency_code": body.get("currency_code").and_then(|v| v.as_str()).unwrap_or("usd"),
                "balance": body.get("balance").unwrap_or(&json!(0))
            }
        })),
    ))
}

pub async fn get_store_credit_account(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "store_credit_account": {
                "id": id,
                "currency_code": "usd",
                "balance": 0
            }
        })),
    ))
}

pub async fn add_store_credit_account_credit(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "store_credit_account": {
                "id": id,
                "amount": body.get("amount").unwrap_or(&json!(0))
            }
        })),
    ))
}

pub async fn list_store_credit_account_transactions(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "transactions": [],
            "data": [],
            "store_credit_account_id": id,
            "count": 0
        })),
    ))
}

// ── Claims Sub-Actions ────────────────────────────────────────────────────────

pub async fn cancel_claim(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "status": "canceled" } })),
    ))
}

pub async fn add_claim_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "claim": { "id": id } }))))
}

pub async fn update_claim_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_claim_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "claim_item", "deleted": true })),
    ))
}

pub async fn add_claim_inbound_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "claim": { "id": id } }))))
}

pub async fn update_claim_inbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_claim_inbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "claim_inbound_item", "deleted": true })),
    ))
}

pub async fn add_claim_inbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "claim": { "id": id } }))))
}

pub async fn update_claim_inbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_claim_inbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(
            json!({ "id": action_id, "object": "claim_inbound_shipping_method", "deleted": true }),
        ),
    ))
}

pub async fn add_claim_outbound_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "claim": { "id": id } }))))
}

pub async fn update_claim_outbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_claim_outbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "claim_outbound_item", "deleted": true })),
    ))
}

pub async fn add_claim_outbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "claim": { "id": id } }))))
}

pub async fn update_claim_outbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_claim_outbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(
            json!({ "id": action_id, "object": "claim_outbound_shipping_method", "deleted": true }),
        ),
    ))
}

pub async fn request_claim(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "status": "requested" } })),
    ))
}

pub async fn cancel_claim_request(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "claim": { "id": id, "status": "canceled" } })),
    ))
}

// ── Exchanges Sub-Actions ─────────────────────────────────────────────────────

pub async fn cancel_exchange(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "status": "canceled" } })),
    ))
}

pub async fn add_exchange_inbound_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "exchange": { "id": id } }))))
}

pub async fn update_exchange_inbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_exchange_inbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "exchange_inbound_item", "deleted": true })),
    ))
}

pub async fn add_exchange_inbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "exchange": { "id": id } }))))
}

pub async fn update_exchange_inbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_exchange_inbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(
            json!({ "id": action_id, "object": "exchange_inbound_shipping_method", "deleted": true }),
        ),
    ))
}

pub async fn add_exchange_outbound_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "exchange": { "id": id } }))))
}

pub async fn update_exchange_outbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_exchange_outbound_item(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "exchange_outbound_item", "deleted": true })),
    ))
}

pub async fn add_exchange_outbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "exchange": { "id": id } }))))
}

pub async fn update_exchange_outbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_exchange_outbound_shipping_method(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(
            json!({ "id": action_id, "object": "exchange_outbound_shipping_method", "deleted": true }),
        ),
    ))
}

pub async fn request_exchange(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "status": "requested" } })),
    ))
}

pub async fn cancel_exchange_request(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "exchange": { "id": id, "status": "canceled" } })),
    ))
}

// ── Draft Orders Operations ───────────────────────────────────────────────────

pub async fn update_draft_order(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn delete_draft_order(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "draft_order", "deleted": true })),
    ))
}

pub async fn convert_draft_order_to_order(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order_id = format!("order_{}", ulid::Ulid::new());
    Ok((
        StatusCode::OK,
        Json(json!({ "order": { "id": order_id, "draft_order_id": id } })),
    ))
}

pub async fn create_draft_order_edit(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::CREATED,
        Json(json!({ "draft_order": { "id": id } })),
    ))
}

pub async fn delete_draft_order_edit(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "draft_order_edit", "deleted": true })),
    ))
}

pub async fn confirm_draft_order_edit(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn add_draft_order_edit_items(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn update_draft_order_edit_original_item(
    State(_ctx): State<AdminOrderContext>,
    Path((id, item_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "draft_order": { "id": id, "item_id": item_id } })),
    ))
}

pub async fn update_draft_order_edit_item_action(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "draft_order": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_draft_order_edit_item_action(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "draft_order_edit_item_action", "deleted": true })),
    ))
}

pub async fn add_draft_order_edit_promotions(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn delete_draft_order_edit_promotions(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn request_draft_order_edit(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn add_draft_order_edit_shipping_methods(
    State(_ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((StatusCode::OK, Json(json!({ "draft_order": { "id": id } }))))
}

pub async fn update_draft_order_edit_shipping_method_by_id(
    State(_ctx): State<AdminOrderContext>,
    Path((id, method_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "draft_order": { "id": id, "method_id": method_id } })),
    ))
}

pub async fn delete_draft_order_edit_shipping_method_by_id(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, method_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "id": method_id, "object": "draft_order_shipping_method", "deleted": true })),
    ))
}

pub async fn update_draft_order_edit_shipping_method_action(
    State(_ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "draft_order": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_draft_order_edit_shipping_method_action(
    State(_ctx): State<AdminOrderContext>,
    Path((_id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(
            json!({ "id": action_id, "object": "draft_order_shipping_method_action", "deleted": true }),
        ),
    ))
}

// ── Direct Fulfillments & Order Changes ───────────────────────────────────────

pub async fn create_fulfillment_direct(
    State(ctx): State<AdminOrderContext>,
    Json(input): Json<CreateFulfillmentInput>,
) -> Result<impl IntoResponse, Response> {
    let f = ctx
        .fulfillment_service
        .create_fulfillment(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "fulfillment": f, "data": f })),
    ))
}

pub async fn cancel_fulfillment_direct(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let _ = ctx.fulfillment_service.cancel_fulfillment(&id).await;
    Ok((
        StatusCode::OK,
        Json(json!({ "fulfillment": { "id": id, "canceled_at": chrono::Utc::now() } })),
    ))
}

pub async fn create_fulfillment_shipment_direct(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let _ = ctx.fulfillment_service.mark_as_shipped(&id).await;
    Ok((
        StatusCode::OK,
        Json(json!({ "fulfillment": { "id": id, "shipped_at": chrono::Utc::now() } })),
    ))
}

pub async fn apply_order_change(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let order = ctx
        .order_service
        .confirm_order_change(&id, None)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "order": order, "data": order })),
    ))
}
