//! Admin Orders, Fulfillments, Payments, and Returns API Handlers.

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_auth::JwtClaims;
use stheno_fulfillment::{CreateFulfillmentInput, FulfillmentService};
use stheno_order::{
    CreateClaimInput, CreateExchangeInput, CreateOrderChangeInput, CreateOrderInput,
    CreateReturnInput, CreateReturnReasonInput, OrderFilter, OrderService,
    ReviewOrderPaymentProofInput, UpdateOrderInput,
};
use stheno_payment::{CapturePaymentInput, PaymentService, RefundPaymentInput};

#[derive(Clone)]
pub struct AdminOrderContext {
    pub order_service: Arc<OrderService>,
    pub fulfillment_service: Arc<FulfillmentService>,
    pub payment_service: Arc<PaymentService>,
    pub links: Option<Arc<stheno_links::PgLinkStore>>,
    pub notification_service: Option<Arc<stheno_notification::NotificationService>>,
    pub social_flows: Arc<stheno_flows_order::SocialDraftWorkflow>,
    pub settings_service: Option<Arc<stheno_settings::SettingsService>>,
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
    pub is_social: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewOrderPaymentProofRequest {
    pub decision: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconcileSocialCourierDispatchRequest {
    pub outcome: String,
    pub evidence_reference: String,
    pub courier_reference: Option<String>,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub label_url: Option<String>,
}

/// Reconciles an unknown social courier response after staff verifies the courier's record.
pub async fn reconcile_social_courier_dispatch(
    State(ctx): State<AdminOrderContext>,
    Extension(claims): Extension<JwtClaims>,
    Path(order_id): Path<String>,
    Json(request): Json<ReconcileSocialCourierDispatchRequest>,
) -> Result<impl IntoResponse, Response> {
    if claims.actor_type.as_deref() != Some("user") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "not_allowed",
                "Courier reconciliation requires an authenticated staff identity.",
            )),
        )
            .into_response());
    }
    let actor_id = claims
        .actor_id
        .filter(|id| !id.trim().is_empty())
        .or_else(|| (!claims.sub.trim().is_empty()).then(|| claims.sub.clone()))
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse::new(
                    "not_allowed",
                    "Courier reconciliation requires an authenticated staff identity.",
                )),
            )
                .into_response()
        })?;
    let order = ctx
        .order_service
        .get_order(&order_id)
        .await
        .map_err(not_found)?;
    let is_social = order
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("source"))
        .and_then(serde_json::Value::as_str)
        == Some(stheno_flows_order::SOCIAL_SOURCE);
    if !is_social || order.is_draft_order {
        return Err(bad_request(stheno_types::Error::Conflict {
            code: "social_dispatch_order_not_reconcilable",
            message: "Only a confirmed social order can have courier dispatch reconciled."
                .to_string(),
        }));
    }

    let resolution = match request.outcome.as_str() {
        "accepted" => stheno_fulfillment::SocialDispatchResolution::Accepted {
            courier_reference: request.courier_reference.ok_or_else(|| {
                bad_request(stheno_types::Error::Invalid {
                    code: "social_dispatch_reconciliation_reference_missing",
                    message: "A courier reference is required for an accepted consignment."
                        .to_string(),
                    field: Some("courier_reference".to_string()),
                })
            })?,
            tracking_number: request.tracking_number.ok_or_else(|| {
                bad_request(stheno_types::Error::Invalid {
                    code: "social_dispatch_reconciliation_tracking_missing",
                    message: "A tracking number is required for an accepted consignment."
                        .to_string(),
                    field: Some("tracking_number".to_string()),
                })
            })?,
            tracking_url: request.tracking_url,
            label_url: request.label_url,
        },
        "not_created" => {
            if request.courier_reference.is_some()
                || request.tracking_number.is_some()
                || request.tracking_url.is_some()
                || request.label_url.is_some()
            {
                return Err(bad_request(stheno_types::Error::Invalid {
                    code: "social_dispatch_reconciliation_fields_invalid",
                    message:
                        "A not-created resolution cannot include consignment or tracking fields."
                            .to_string(),
                    field: Some("outcome".to_string()),
                }));
            }
            stheno_fulfillment::SocialDispatchResolution::NotCreated
        }
        _ => {
            return Err(bad_request(stheno_types::Error::Invalid {
                code: "social_dispatch_reconciliation_outcome_invalid",
                message: "Outcome must be accepted or not_created.".to_string(),
                field: Some("outcome".to_string()),
            }));
        }
    };
    let stable_reference = format!("social-order:{}", order.id);
    ctx.fulfillment_service
        .reconcile_unknown_social_dispatch(
            &stable_reference,
            &actor_id,
            &request.evidence_reference,
            resolution,
        )
        .await
        .map_err(bad_request)?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "order_id": order.id,
            "outcome": request.outcome,
            "reconciled": true,
            "next_action": "retry_staff_confirmation",
        })),
    ))
}

pub async fn get_order_payment_proof(
    State(ctx): State<AdminOrderContext>,
    Path(order_id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let proof = ctx
        .order_service
        .get_manual_payment_proof_for_order(&order_id)
        .await
        .map_err(|error| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    "payment_proof_not_found",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    Ok((StatusCode::OK, Json(json!({ "payment_proof": proof }))))
}

/// GET /admin/notifications/usage — current tenant's UTC-day allowance and usage.
pub async fn get_notification_usage(
    State(ctx): State<AdminOrderContext>,
) -> Result<impl IntoResponse, Response> {
    let Some(notification_service) = &ctx.notification_service else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "notification_usage_unavailable",
                "Notification usage reporting is not configured",
            )),
        )
            .into_response());
    };
    let usage = notification_service
        .get_daily_usage()
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "notification usage could not be loaded");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new(
                    "notification_usage_unavailable",
                    "Notification usage could not be loaded",
                )),
            )
                .into_response()
        })?;
    Ok((StatusCode::OK, Json(json!({ "usage": usage }))))
}

pub async fn review_order_payment_proof(
    State(ctx): State<AdminOrderContext>,
    Extension(claims): Extension<JwtClaims>,
    Path(proof_id): Path<String>,
    Json(body): Json<ReviewOrderPaymentProofRequest>,
) -> Result<impl IntoResponse, Response> {
    let proof = ctx
        .order_service
        .review_manual_payment_proof(ReviewOrderPaymentProofInput {
            proof_id,
            reviewer_id: claims.sub,
            decision: body.decision,
            note: body.note,
        })
        .await
        .map_err(|error| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse::new(
                    "payment_proof_review_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;

    if let Some(notification_service) = &ctx.notification_service {
        if let Ok(order) = ctx.order_service.get_order(&proof.order_id).await {
            if let Some(email) = order.email.filter(|email| email.contains('@')) {
                let (subject, body) = if proof.status == "verified" {
                    (
                        format!("Payment verified for order #{}", order.display_id),
                        format!(
                            "<p>The merchant verified your payment for order <strong>#{}</strong>.</p>\
                             <p>The payment is now recorded as captured.</p>",
                            order.display_id
                        ),
                    )
                } else {
                    (
                        format!(
                            "Payment reference needs attention for order #{}",
                            order.display_id
                        ),
                        format!(
                            "<p>The merchant could not verify the payment reference for order <strong>#{}</strong>.</p>\
                             <p>Your order remains unpaid. Please contact the store for help.</p>",
                            order.display_id
                        ),
                    )
                };
                let queued = notification_service
                    .enqueue_notification_idempotent(
                        stheno_notification::SendNotificationInput {
                            to: email.clone(),
                            channel: "email".to_string(),
                            template: None,
                            subject: Some(subject),
                            body: Some(body),
                            data: None,
                            trigger_type: Some(format!("order.payment_proof_{}", proof.status)),
                            resource_id: Some(proof.order_id.clone()),
                            resource_type: Some("order".to_string()),
                            receiver_id: Some(email),
                        },
                        &format!("order-payment-proof/{}/{}", proof.id, proof.status),
                    )
                    .await;
                if queued.is_err() {
                    tracing::warn!(
                        order_id = %proof.order_id,
                        proof_id = %proof.id,
                        "payment proof review was saved but its customer email could not be queued"
                    );
                }
            }
        }
    }
    Ok((StatusCode::OK, Json(json!({ "payment_proof": proof }))))
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

    let listing = if query.is_social == Some(true) {
        ctx.order_service.list_social_drafts(limit, offset).await
    } else {
        ctx.order_service.list_orders(filter).await
    };
    let (all, count) = listing.map_err(|e| {
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
                &format!("Order with id {id} was not found"),
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
    if let (Some(notification_service), Some(email)) =
        (&ctx.notification_service, order.email.as_deref())
    {
        if email.contains('@') {
            let queued = notification_service
                .enqueue_notification_idempotent(
                    stheno_notification::SendNotificationInput {
                        to: email.to_string(),
                        channel: "email".to_string(),
                        template: None,
                        subject: Some(format!("Order #{} was canceled", order.display_id)),
                        body: Some(format!(
                            "<p>Your order <strong>#{}</strong> was canceled by the store.</p>\
                             <p>If you have questions about any payment or refund, contact the store.</p>",
                            order.display_id
                        )),
                        data: None,
                        trigger_type: Some("order.canceled".to_string()),
                        resource_id: Some(order.id.clone()),
                        resource_type: Some("order".to_string()),
                        receiver_id: Some(email.to_string()),
                    },
                    &format!("order-canceled/{}", order.id),
                )
                .await;
            if queued.is_err() {
                tracing::warn!(
                    order_id = %order.id,
                    "order was canceled but its customer email could not be queued"
                );
            }
        }
    }
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

    let order_status_updated = ctx
        .order_service
        .update_order(
            &order_id,
            UpdateOrderInput {
                fulfillment_status: Some("fulfilled".to_string()),
                ..Default::default()
            },
        )
        .await
        .is_ok();

    // A committed fulfillment can exist even if the denormalized order status write
    // fails. Do not tell the customer it is being prepared until both writes succeeded.
    if order_status_updated {
        queue_fulfillment_status_email(&ctx, &order_id, &ful, "prepared").await;
    } else {
        tracing::warn!(
            order_id,
            fulfillment_id = %ful.id,
            "fulfillment was created but order status could not be updated; customer notice skipped"
        );
    }

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
    Path(order_id): Path<String>,
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
    queue_payment_lifecycle_email(
        &ctx,
        &cap.payment_id,
        Some(&order_id),
        "captured",
        cap.amount,
        &cap.id,
    )
    .await;
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
    Path(order_id): Path<String>,
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
    queue_payment_lifecycle_email(
        &ctx,
        &ref_dto.payment_id,
        Some(&order_id),
        "refunded",
        ref_dto.amount,
        &ref_dto.id,
    )
    .await;
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
    queue_payment_collection_paid_email(&ctx, &col).await;
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
    queue_payment_lifecycle_email(&ctx, &cap.payment_id, None, "captured", cap.amount, &cap.id)
        .await;
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
    queue_payment_lifecycle_email(
        &ctx,
        &ref_dto.payment_id,
        None,
        "refunded",
        ref_dto.amount,
        &ref_dto.id,
    )
    .await;
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
    ctx.order_service
        .start_return_receipt(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("receive_failed", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
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
                &format!("Return reason {id} not found"),
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
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx
        .order_service
        .get_return_dto(&id)
        .await
        .map_err(return_lookup_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "return": { "id": ret.id, "status": ret.status },
            "data": { "id": ret.id, "status": ret.status }
        })),
    ))
}

fn return_lookup_error(error: stheno_types::Error) -> Response {
    let (status, code, message) = match error {
        stheno_types::Error::NotFound { .. } => (
            StatusCode::NOT_FOUND,
            "return_not_found",
            "Return was not found",
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected_state",
            "Return could not be loaded",
        ),
    };
    (status, Json(ErrorResponse::new(code, message))).into_response()
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
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let canceled = ctx
        .order_service
        .cancel_return(&id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "return_cancel_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    queue_return_lifecycle_email(&ctx, &canceled, "canceled").await;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": canceled.id, "status": canceled.status } })),
    ))
}

pub async fn dismiss_return_items(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(body): Json<ReturnReceiptItemsRequest>,
) -> Result<impl IntoResponse, Response> {
    for item in body.items {
        ctx.order_service
            .stage_return_receipt_quantity(&id, &item.id, item.quantity, true)
            .await
            .map_err(bad_request)?;
    }
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
}

pub async fn update_dismiss_return_item(
    State(ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(body): Json<ReturnReceiptQuantityRequest>,
) -> Result<impl IntoResponse, Response> {
    ctx.order_service
        .update_staged_return_receipt_action(&id, &action_id, body.quantity, true)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_dismiss_return_item(
    State(ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    ctx.order_service
        .remove_staged_return_receipt_quantity(&id, &action_id, true)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "return_dismiss_item", "deleted": true })),
    ))
}

pub async fn cancel_return_receive(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx
        .order_service
        .cancel_return_receipt(&id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "return_receipt_cancel_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": ret.id, "status": ret.status } })),
    ))
}

pub async fn receive_return_items(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(body): Json<ReturnReceiptItemsRequest>,
) -> Result<impl IntoResponse, Response> {
    for item in body.items {
        ctx.order_service
            .stage_return_receipt_quantity(&id, &item.id, item.quantity, false)
            .await
            .map_err(bad_request)?;
    }
    Ok((StatusCode::OK, Json(json!({ "return": { "id": id } }))))
}

#[derive(Debug, Deserialize)]
pub struct ReturnReceiptItemsRequest {
    pub items: Vec<ReturnReceiptItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct ReturnReceiptItemRequest {
    pub id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReturnReceiptQuantityRequest {
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfirmReturnReceiptRequest {
    pub no_notification: Option<bool>,
}

pub async fn update_receive_return_item(
    State(ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
    Json(body): Json<ReturnReceiptQuantityRequest>,
) -> Result<impl IntoResponse, Response> {
    ctx.order_service
        .update_staged_return_receipt_action(&id, &action_id, body.quantity, false)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": id, "action_id": action_id } })),
    ))
}

pub async fn delete_receive_return_item(
    State(ctx): State<AdminOrderContext>,
    Path((id, action_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    ctx.order_service
        .remove_staged_return_receipt_quantity(&id, &action_id, false)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": action_id, "object": "return_receive_item", "deleted": true })),
    ))
}

pub async fn confirm_return_receive(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
    Json(body): Json<ConfirmReturnReceiptRequest>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx
        .order_service
        .confirm_staged_return_receipt(&id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "return_confirmation_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    if !body.no_notification.unwrap_or(false) {
        queue_return_lifecycle_email(&ctx, &ret, "received").await;
    }
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": ret.id, "status": ret.status } })),
    ))
}

pub async fn confirm_return_request(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let ret = ctx
        .order_service
        .get_return_dto(&id)
        .await
        .map_err(|error| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("return_not_found", &error.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": ret.id, "status": ret.status } })),
    ))
}

pub async fn cancel_return_request(
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let canceled = ctx
        .order_service
        .cancel_return(&id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "return_cancel_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    queue_return_lifecycle_email(&ctx, &canceled, "canceled").await;
    Ok((
        StatusCode::OK,
        Json(json!({ "return": { "id": canceled.id, "status": canceled.status } })),
    ))
}

async fn queue_return_lifecycle_email(
    ctx: &AdminOrderContext,
    ret: &stheno_order::ReturnDTO,
    status: &str,
) {
    let Some(notification_service) = &ctx.notification_service else {
        return;
    };
    let Ok(order) = ctx.order_service.get_order(&ret.order_id).await else {
        return;
    };
    let Some(email) = order.email.filter(|email| email.contains('@')) else {
        return;
    };
    let Some((subject, body)) = return_notification_content(status, order.display_id) else {
        return;
    };
    let queued = notification_service
        .enqueue_notification_idempotent(
            stheno_notification::SendNotificationInput {
                to: email.clone(),
                channel: "email".to_string(),
                template: None,
                subject: Some(subject),
                body: Some(body),
                data: None,
                trigger_type: Some(format!("order.return_{status}")),
                resource_id: Some(order.id.clone()),
                resource_type: Some("order".to_string()),
                receiver_id: Some(email),
            },
            &format!("order-return/{}/{status}", ret.id),
        )
        .await;
    if queued.is_err() {
        tracing::warn!(
            order_id = %order.id,
            return_id = %ret.id,
            status,
            "return lifecycle changed but its customer email could not be queued"
        );
    }
}

fn return_notification_content(status: &str, display_id: i64) -> Option<(String, String)> {
    Some(match status {
        "received" => (
            format!("Return items received for order #{display_id}"),
            format!(
                "<p>The store recorded receipt of the items for return on order <strong>#{display_id}</strong>.</p>\
                 <p>The return is still subject to review. This notice does not mean a refund was issued.</p>"
            ),
        ),
        "canceled" => (
            format!("Return request for order #{display_id} was canceled"),
            format!(
                "<p>The return request for order <strong>#{display_id}</strong> was canceled by the store.</p>\
                 <p>Contact the store if you have questions about your order or any refund.</p>"
            ),
        ),
        _ => return None,
    })
}

async fn queue_payment_lifecycle_email(
    ctx: &AdminOrderContext,
    payment_id: &str,
    expected_order_id: Option<&str>,
    event: &str,
    amount: rust_decimal::Decimal,
    event_id: &str,
) {
    let (Some(notification_service), Some(links)) = (&ctx.notification_service, &ctx.links) else {
        return;
    };
    let Ok(payment) = ctx.payment_service.get_payment(payment_id).await else {
        return;
    };
    let order_ids = if let Some(order_id) = expected_order_id {
        let Ok(collection_ids) = links
            .get_by_left("order_payment_collection", order_id)
            .await
        else {
            return;
        };
        if !collection_ids.contains(&payment.payment_collection_id) {
            tracing::warn!(
                order_id,
                payment_id,
                "payment lifecycle event did not match the order payment collection"
            );
            return;
        }
        let Ok(linked_order_ids) = links
            .get_by_right("order_payment_collection", &payment.payment_collection_id)
            .await
        else {
            return;
        };
        if linked_order_ids.len() != 1 || linked_order_ids[0].as_str() != order_id {
            tracing::warn!(
                order_id,
                payment_id,
                count = linked_order_ids.len(),
                "payment lifecycle notice skipped because the collection has multiple order links"
            );
            return;
        }
        vec![order_id.to_string()]
    } else {
        let Ok(order_ids) = links
            .get_by_right("order_payment_collection", &payment.payment_collection_id)
            .await
        else {
            return;
        };
        order_ids
    };
    if order_ids.len() != 1 {
        tracing::warn!(
            payment_id,
            count = order_ids.len(),
            "payment lifecycle notice skipped because its order association is ambiguous"
        );
        return;
    }
    let Ok(order) = ctx.order_service.get_order(&order_ids[0]).await else {
        return;
    };
    let Some(email) = order.email.filter(|email| email.contains('@')) else {
        return;
    };

    let (subject, body) = if event == "captured" {
        (
            format!("Payment received for order #{}", order.display_id),
            format!(
                "<p>A payment of <strong>{amount} {currency}</strong> was captured for order <strong>#{display_id}</strong>.</p>",
                currency = payment.currency_code.to_ascii_uppercase(),
                display_id = order.display_id,
            ),
        )
    } else {
        (
            format!("Refund issued for order #{}", order.display_id),
            format!(
                "<p>A refund of <strong>{amount} {currency}</strong> was issued for order <strong>#{display_id}</strong>. Your bank or payment provider may take additional time to show it.</p>",
                currency = payment.currency_code.to_ascii_uppercase(),
                display_id = order.display_id,
            ),
        )
    };
    let queued = notification_service
        .enqueue_notification_idempotent(
            stheno_notification::SendNotificationInput {
                to: email.clone(),
                channel: "email".to_string(),
                template: None,
                subject: Some(subject),
                body: Some(body),
                data: None,
                trigger_type: Some(format!("order.payment_{event}")),
                resource_id: Some(order.id.clone()),
                resource_type: Some("order".to_string()),
                receiver_id: Some(email),
            },
            &format!("order-payment-{event}/{event_id}"),
        )
        .await;
    if queued.is_err() {
        tracing::warn!(
            order_id = %order.id,
            payment_id,
            event,
            "payment lifecycle event persisted but its customer email could not be queued"
        );
    }
}

async fn queue_payment_collection_paid_email(
    ctx: &AdminOrderContext,
    collection: &stheno_payment::PaymentCollectionDTO,
) {
    let (Some(notification_service), Some(links)) = (&ctx.notification_service, &ctx.links) else {
        return;
    };
    let Ok(order_ids) = links
        .get_by_right("order_payment_collection", &collection.id)
        .await
    else {
        return;
    };
    if order_ids.len() != 1 {
        tracing::warn!(
            collection_id = %collection.id,
            count = order_ids.len(),
            "payment status notice skipped because its order association is ambiguous"
        );
        return;
    }
    let Ok(order) = ctx.order_service.get_order(&order_ids[0]).await else {
        return;
    };
    let Some(email) = order.email.filter(|email| email.contains('@')) else {
        return;
    };
    let queued = notification_service
        .enqueue_notification_idempotent(
            stheno_notification::SendNotificationInput {
                to: email.clone(),
                channel: "email".to_string(),
                template: None,
                subject: Some(format!("Payment recorded for order #{}", order.display_id)),
                body: Some(format!(
                    "<p>The store recorded <strong>{} {}</strong> as paid for order <strong>#{}</strong>.</p>",
                    collection.amount,
                    collection.currency_code.to_ascii_uppercase(),
                    order.display_id
                )),
                data: None,
                trigger_type: Some("order.payment_collection_paid".to_string()),
                resource_id: Some(order.id.clone()),
                resource_type: Some("order".to_string()),
                receiver_id: Some(email),
            },
            &format!("order-payment-collection-paid/{}", collection.id),
        )
        .await;
    if queued.is_err() {
        tracing::warn!(
            order_id = %order.id,
            collection_id = %collection.id,
            "payment status was persisted but its customer email could not be queued"
        );
    }
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
    Path((order_id, fulfillment_id)): Path<(String, String)>,
    Json(_input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let fulfillment = ctx
        .fulfillment_service
        .mark_as_shipped(&fulfillment_id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "fulfillment_ship_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    queue_fulfillment_status_email(&ctx, &order_id, &fulfillment, "shipped").await;
    Ok((
        StatusCode::OK,
        Json(json!({ "fulfillment": { "id": fulfillment_id, "shipped_at": chrono::Utc::now() } })),
    ))
}

pub async fn mark_fulfillment_delivered(
    State(ctx): State<AdminOrderContext>,
    Path((order_id, fulfillment_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let fulfillment = ctx
        .fulfillment_service
        .mark_as_delivered(&fulfillment_id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "fulfillment_delivery_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    queue_fulfillment_status_email(&ctx, &order_id, &fulfillment, "delivered").await;
    Ok((
        StatusCode::OK,
        Json(
            json!({ "fulfillment": { "id": fulfillment_id, "delivered_at": chrono::Utc::now() } }),
        ),
    ))
}

async fn queue_fulfillment_status_email(
    ctx: &AdminOrderContext,
    order_id: &str,
    fulfillment: &stheno_fulfillment::FulfillmentDTO,
    status: &str,
) {
    let Some(notification_service) = &ctx.notification_service else {
        return;
    };
    let Ok(order) = ctx.order_service.get_order(order_id).await else {
        return;
    };
    if fulfillment.items.is_empty()
        || !fulfillment.items.iter().all(|item| {
            item.quantity > 0 && order.items.iter().any(|line| line.id == item.line_item_id)
        })
    {
        return;
    }
    let Some(email) = order.email.filter(|email| email.contains('@')) else {
        return;
    };
    let Some((subject, body)) = fulfillment_notification_content(status, order.display_id) else {
        return;
    };
    let queued = notification_service
        .enqueue_notification_idempotent(
            stheno_notification::SendNotificationInput {
                to: email.clone(),
                channel: "email".to_string(),
                template: None,
                subject: Some(subject),
                body: Some(body),
                data: None,
                trigger_type: Some(format!("fulfillment.{status}")),
                resource_id: Some(order.id.clone()),
                resource_type: Some("order".to_string()),
                receiver_id: Some(email),
            },
            &format!("fulfillment/{}/{status}", fulfillment.id),
        )
        .await;
    if queued.is_err() {
        tracing::warn!(
            order_id = %order.id,
            fulfillment_id = %fulfillment.id,
            status,
            "fulfillment status changed but its customer email could not be queued"
        );
    }
}

fn fulfillment_notification_content(status: &str, display_id: i64) -> Option<(String, String)> {
    Some(match status {
        "prepared" => (
            format!("Fulfillment started for order #{display_id}"),
            format!(
                "<p>Fulfillment for order <strong>#{display_id}</strong> has started. We will notify you when it ships.</p>"
            ),
        ),
        "shipped" => (
            format!("Order #{display_id} has shipped"),
            format!("<p>Your order <strong>#{display_id}</strong> has been marked shipped.</p>"),
        ),
        "delivered" => (
            format!("Order #{display_id} was delivered"),
            format!("<p>Your order <strong>#{display_id}</strong> has been marked delivered.</p>"),
        ),
        "canceled" => (
            format!("Shipment for order #{display_id} was canceled"),
            format!(
                "<p>The shipment for order <strong>#{display_id}</strong> was canceled by the store. Contact the store if you need help with your order.</p>"
            ),
        ),
        _ => return None,
    })
}

pub async fn cancel_order_fulfillment(
    State(ctx): State<AdminOrderContext>,
    Path((order_id, fulfillment_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let fulfillment = ctx
        .fulfillment_service
        .cancel_fulfillment(&fulfillment_id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "fulfillment_cancel_failed",
                    &error.to_string(),
                )),
            )
                .into_response()
        })?;
    queue_fulfillment_status_email(&ctx, &order_id, &fulfillment, "canceled").await;
    Ok((
        StatusCode::OK,
        Json(json!({
            "fulfillment": { "id": fulfillment_id, "canceled_at": chrono::Utc::now() }
        })),
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
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let changes = ctx
        .order_service
        .list_order_changes(Some(&id))
        .await
        .map_err(not_found)?;
    let mut changes_with_actions = Vec::with_capacity(changes.len());
    for change in changes {
        changes_with_actions.push(
            ctx.order_service
                .get_order_change_with_actions(&change.id)
                .await
                .map_err(not_found)?,
        );
    }
    Ok((
        StatusCode::OK,
        Json(
            json!({ "order_changes": changes_with_actions, "data": changes_with_actions, "order_id": id }),
        ),
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
    State(ctx): State<AdminOrderContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let draft = ctx.order_service.get_order(&id).await.map_err(not_found)?;
    let converted = if draft
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("source"))
        .and_then(serde_json::Value::as_str)
        == Some(stheno_flows_order::SOCIAL_SOURCE)
    {
        let settings = ctx.settings_service.as_ref().ok_or_else(|| {
            bad_request(stheno_types::Error::Conflict {
                code: "social_dispatch_settings_unavailable",
                message: "Courier dispatch is unavailable; the social draft was not confirmed."
                    .to_string(),
            })
        })?;
        // Validate courier configuration before conversion reserves inventory. The draft id
        // remains the order id, so the provider idempotency reference is already stable.
        let dispatch = prepare_social_dispatch(&ctx, settings, &draft).await?;
        let converted = ctx
            .social_flows
            .convert_to_order(&id)
            .await
            .map_err(bad_request)?;
        dispatch_confirmed_social_order(&ctx, dispatch).await?;
        converted
    } else {
        ctx.order_service
            .convert_draft_order(&id)
            .await
            .map_err(bad_request)?
    };
    Ok((
        StatusCode::OK,
        Json(json!({ "order": converted, "draft_order_id": id })),
    ))
}

struct SocialDispatchPlan {
    input: stheno_fulfillment::CreateFulfillmentInput,
    stable_reference: String,
    provider_data: serde_json::Value,
}

async fn prepare_social_dispatch(
    ctx: &AdminOrderContext,
    settings: &stheno_settings::SettingsService,
    order: &stheno_order::OrderDTO,
) -> Result<SocialDispatchPlan, Response> {
    if order.shipping_methods.len() != 1 {
        return Err(bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_shipping_method_invalid",
            message: "Confirmed social order must have exactly one validated shipping method."
                .to_string(),
            field: Some("shipping_methods".to_string()),
        }));
    }
    let method = &order.shipping_methods[0];
    let option_id = method
        .shipping_option_id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| {
            bad_request(stheno_types::Error::Invalid {
                code: "social_dispatch_option_missing",
                message: "Confirmed social order has no courier shipping option.".to_string(),
                field: Some("shipping_option_id".to_string()),
            })
        })?;
    let option = ctx
        .fulfillment_service
        .get_shipping_option(option_id)
        .await
        .map_err(bad_request)?;
    let provider_id = option.provider_id.as_str();
    if !matches!(provider_id, "pathao" | "steadfast" | "redx") {
        return Err(bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_provider_unsupported",
            message: "The selected social order courier does not support automated dispatch."
                .to_string(),
            field: Some("provider_id".to_string()),
        }));
    }
    let credentials = crate::couriers::load_courier_credentials(settings, provider_id)
        .await
        .map_err(bad_request)?;
    if credentials
        .get("is_enabled")
        .and_then(serde_json::Value::as_bool)
        == Some(false)
    {
        return Err(bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_provider_disabled",
            message: "The selected courier is disabled for this store.".to_string(),
            field: Some("provider_id".to_string()),
        }));
    }
    if matches!(provider_id, "pathao" | "redx")
        && !crate::couriers::destination_mappings_valid(provider_id, &credentials)
    {
        return Err(bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_mapping_invalid",
            message: "Courier destination mappings are missing, malformed, or ambiguous."
                .to_string(),
            field: Some("destination_mappings".to_string()),
        }));
    }
    let address = order.shipping_address.as_ref().ok_or_else(|| {
        bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_address_missing",
            message: "The confirmed social order has no delivery address.".to_string(),
            field: Some("shipping_address".to_string()),
        })
    })?;
    let required = |value: Option<&str>, field: &'static str| -> Result<String, Response> {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                bad_request(stheno_types::Error::Invalid {
                    code: "social_dispatch_recipient_missing",
                    message: format!("Delivery {field} is required before courier dispatch."),
                    field: Some(field.to_string()),
                })
            })
    };
    let recipient_name = format!(
        "{} {}",
        address.first_name.as_deref().unwrap_or_default().trim(),
        address.last_name.as_deref().unwrap_or_default().trim()
    )
    .trim()
    .to_string();
    let recipient_name = required(Some(&recipient_name), "recipient name")?;
    let recipient_phone = required(address.phone.as_deref(), "phone")?;
    let mut address_parts = Vec::new();
    for part in [
        address.address_1.as_deref(),
        address.address_2.as_deref(),
        address.city.as_deref(),
        address.province.as_deref(),
        address.postal_code.as_deref(),
    ] {
        if let Some(part) = part.map(str::trim).filter(|part| !part.is_empty()) {
            address_parts.push(part);
        }
    }
    let recipient_address = required(Some(&address_parts.join(", ")), "street address")?;
    let country = required(address.country_code.as_deref(), "country code")?;
    if !country.eq_ignore_ascii_case("BD") {
        return Err(bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_country_unsupported",
            message: "Automated social dispatch currently supports Bangladesh destinations only."
                .to_string(),
            field: Some("country_code".to_string()),
        }));
    }
    let city = required(address.city.as_deref(), "city")?;
    let province = address.province.as_deref().unwrap_or_default().trim();
    let mapping_value = credentials
        .get("destination_mappings")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            bad_request(stheno_types::Error::Invalid {
                code: "social_dispatch_mapping_missing",
                message:
                    "Configure this courier's destination mapping before confirming the order."
                        .to_string(),
                field: Some("destination_mappings".to_string()),
            })
        })?;
    let mappings: serde_json::Value = serde_json::from_str(mapping_value).map_err(|_| {
        bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_mapping_invalid",
            message: "Courier destination mappings must contain valid JSON.".to_string(),
            field: Some("destination_mappings".to_string()),
        })
    })?;
    let canonical_mapping_parts = [
        country.trim().to_lowercase(),
        province.to_lowercase(),
        city.to_lowercase(),
    ];
    let mapping_key = canonical_mapping_parts.join("|");
    let mapping = mappings
        .as_object()
        .and_then(|entries| {
            entries.iter().find(|(key, _)| {
                key.split('|')
                    .map(|part| part.trim().to_lowercase())
                    .collect::<Vec<_>>()
                    == canonical_mapping_parts
            })
        })
        .map(|(_, value)| value)
        .ok_or_else(|| {
            bad_request(stheno_types::Error::Invalid {
                code: "social_dispatch_mapping_missing",
                message: format!("No courier destination mapping is configured for {mapping_key}."),
                field: Some("destination_mappings".to_string()),
            })
        })?;
    let mut request_data = serde_json::Map::new();
    let reference = format!("social-order:{}", order.id);
    request_data.insert("merchant_order_id".to_string(), reference.clone().into());
    request_data.insert("invoice".to_string(), reference.clone().into());
    request_data.insert("merchant_invoice_id".to_string(), reference.clone().into());
    request_data.insert("reference".to_string(), reference.into());
    request_data.insert("recipient_name".to_string(), recipient_name.clone().into());
    request_data.insert("customer_name".to_string(), recipient_name.into());
    request_data.insert(
        "recipient_phone".to_string(),
        recipient_phone.clone().into(),
    );
    request_data.insert("customer_phone".to_string(), recipient_phone.into());
    request_data.insert(
        "recipient_address".to_string(),
        recipient_address.clone().into(),
    );
    request_data.insert("customer_address".to_string(), recipient_address.into());
    for key in [
        "client_id",
        "client_secret",
        "username",
        "password",
        "store_id",
        "api_key",
        "secret_key",
        "sandbox_mode",
    ] {
        if let Some(value) = credentials.get(key) {
            request_data.insert(key.to_string(), value.clone());
        }
    }
    if provider_id == "pathao" {
        for (source, target) in [("city_id", "recipient_city"), ("zone_id", "recipient_zone")] {
            let value = mapping
                .get(source)
                .and_then(serde_json::Value::as_u64)
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    bad_request(stheno_types::Error::Invalid {
                        code: "social_dispatch_mapping_invalid",
                        message: format!(
                            "Courier destination mapping for {mapping_key} requires a positive {source}."
                        ),
                        field: Some("destination_mappings".to_string()),
                    })
                })?;
            request_data.insert(target.to_string(), value.into());
        }
    } else if provider_id == "redx" {
        let area_id = mapping
            .get("area_id")
            .and_then(serde_json::Value::as_u64)
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                bad_request(stheno_types::Error::Invalid {
                    code: "social_dispatch_mapping_invalid",
                    message: format!(
                        "Courier destination mapping for {mapping_key} requires a positive area_id."
                    ),
                    field: Some("destination_mappings".to_string()),
                })
            })?;
        request_data.insert("delivery_area_id".to_string(), area_id.into());
        let area_name = mapping
            .get("area_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                bad_request(stheno_types::Error::Invalid {
                    code: "social_dispatch_mapping_invalid",
                    message: format!(
                        "Courier destination mapping for {mapping_key} requires area_name."
                    ),
                    field: Some("destination_mappings".to_string()),
                })
            })?;
        request_data.insert("delivery_area".to_string(), area_name.to_string().into());
        let weight = mapping.get("parcel_weight").and_then(serde_json::Value::as_u64).filter(|value| *value > 0).ok_or_else(|| bad_request(stheno_types::Error::Invalid { code: "social_dispatch_mapping_invalid", message: format!("Courier destination mapping for {mapping_key} requires a positive parcel_weight in grams."), field: Some("destination_mappings".to_string()) }))?;
        request_data.insert("parcel_weight".to_string(), weight.into());
    }
    let collect = order
        .cod_amounts
        .as_ref()
        .map(|amounts| amounts.collected_amount)
        .unwrap_or(rust_decimal::Decimal::ZERO);
    let minor = order
        .currency_code
        .parse::<stheno_types::CurrencyCode>()
        .ok()
        .and_then(|currency| {
            stheno_types::Money::new(collect, currency)
                .to_minor_units()
                .ok()
        })
        .and_then(|minor| u64::try_from(minor).ok())
        .ok_or_else(|| {
            bad_request(stheno_types::Error::Invalid {
                code: "social_dispatch_cod_invalid",
                message:
                    "Cash collection amount cannot be represented in courier minor currency units."
                        .to_string(),
                field: Some("cod_amounts".to_string()),
            })
        })?;
    request_data.insert("amount_to_collect".to_string(), minor.into());
    request_data.insert("cod_amount".to_string(), minor.into());
    request_data.insert("cash_collection_amount".to_string(), minor.into());
    let items = order
        .items
        .iter()
        .filter(|item| item.requires_shipping)
        .map(|item| stheno_fulfillment::CreateFulfillmentItemInput {
            title: item.title.clone(),
            sku: item.variant_sku.clone(),
            barcode: item.variant_barcode.clone(),
            line_item_id: item.id.clone(),
            inventory_item_id: None,
            quantity: item.quantity,
            metadata: None,
        })
        .collect::<Vec<_>>();
    if items.is_empty() || items.iter().any(|item| item.quantity <= 0) {
        return Err(bad_request(stheno_types::Error::Invalid {
            code: "social_dispatch_items_invalid",
            message:
                "Courier dispatch requires at least one shippable item with a positive quantity."
                    .to_string(),
            field: Some("items".to_string()),
        }));
    }
    let input = stheno_fulfillment::CreateFulfillmentInput {
        location_id: None,
        provider_id: provider_id.to_string(),
        shipping_option_id: Some(option_id.to_string()),
        delivery_address: Some(stheno_fulfillment::CreateAddressInput {
            customer_id: order.customer_id.clone(),
            first_name: address.first_name.clone(),
            last_name: address.last_name.clone(),
            address_1: address.address_1.clone(),
            address_2: address.address_2.clone(),
            city: address.city.clone(),
            country_code: address.country_code.clone(),
            province: address.province.clone(),
            postal_code: address.postal_code.clone(),
            phone: address.phone.clone(),
            ..Default::default()
        }),
        items,
        labels: None,
        data: Some(serde_json::json!({"reference": format!("social-order:{}", order.id)})),
        metadata: Some(serde_json::json!({"order_id": order.id, "source": "social"})),
    };
    let stable_reference = format!("social-order:{}", order.id);
    if let Some(existing) = ctx
        .fulfillment_service
        .lookup_by_reference(&stable_reference)
        .await
        .map_err(bad_request)?
    {
        if existing.provider_id != provider_id
            || existing.shipping_option_id.as_deref() != Some(option_id)
        {
            return Err(bad_request(stheno_types::Error::Conflict {
                code: "social_order_already_dispatched",
                message: "This social order already has a courier fulfillment; changing couriers requires staff reconciliation first.".to_string(),
            }));
        }
    }
    Ok(SocialDispatchPlan {
        input,
        stable_reference,
        provider_data: serde_json::Value::Object(request_data),
    })
}

async fn dispatch_confirmed_social_order(
    ctx: &AdminOrderContext,
    plan: SocialDispatchPlan,
) -> Result<(), Response> {
    ctx.fulfillment_service
        .create_fulfillment_idempotent_with_provider_data(
            plan.input,
            &plan.stable_reference,
            plan.provider_data,
        )
        .await
        .map_err(bad_request)?;
    Ok(())
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

#[cfg(test)]
mod social_confirmation_tests {
    use super::{convert_draft_order_to_order, AdminOrderContext};
    use axum::extract::{Path, State};
    use std::sync::Arc;
    use stheno_db::create_pool;
    use stheno_db::testing::TestDb;
    use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};

    #[tokio::test]
    async fn invalid_courier_preflight_leaves_social_draft_unconverted() {
        let db = TestDb::migrated().await;
        let pool = create_pool(&db.url, 3).expect("pool");
        let order_service = Arc::new(OrderService::new(pool.clone()));
        let draft = order_service
            .create_order(CreateOrderInput {
                sales_channel_id: Some("sc_social_test".to_string()),
                currency_code: "bdt".to_string(),
                items: Some(vec![CreateLineItemInput {
                    title: "Social item".to_string(),
                    quantity: 1,
                    unit_price: rust_decimal::Decimal::ONE,
                    ..Default::default()
                }]),
                shipping_methods: Some(Vec::new()),
                is_draft_order: Some(true),
                metadata: Some(serde_json::json!({
                    "source": stheno_flows_order::SOCIAL_SOURCE
                })),
                ..Default::default()
            })
            .await
            .expect("create social draft without a courier option");

        let context = AdminOrderContext {
            order_service: order_service.clone(),
            fulfillment_service: Arc::new(stheno_fulfillment::FulfillmentService::new(
                pool.clone(),
            )),
            payment_service: Arc::new(stheno_payment::PaymentService::new(pool.clone())),
            links: None,
            social_flows: Arc::new(stheno_flows_order::SocialDraftWorkflow::new(
                order_service.clone(),
            )),
            settings_service: Some(Arc::new(stheno_settings::SettingsService::new(pool))),
            notification_service: None,
        };

        let response =
            match convert_draft_order_to_order(State(context), Path(draft.id.clone())).await {
                Ok(_) => panic!("missing courier option must reject before conversion"),
                Err(response) => response,
            };
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

        let unchanged = order_service
            .get_order(&draft.id)
            .await
            .expect("social draft remains available for correction");
        assert!(unchanged.is_draft_order);

        db.drop().await;
    }
}

#[cfg(test)]
mod fulfillment_notification_tests {
    #[test]
    fn prepared_notice_does_not_claim_the_order_has_shipped() {
        let (subject, body) = super::fulfillment_notification_content("prepared", 42)
            .expect("prepared lifecycle has a customer notice");
        assert_eq!(subject, "Fulfillment started for order #42");
        assert!(body.contains("has started"));
        assert!(body.contains("when it ships"));
        assert!(!body.contains("has shipped"));
    }

    #[test]
    fn unsupported_fulfillment_states_do_not_generate_customer_notices() {
        assert!(super::fulfillment_notification_content("unknown", 42).is_none());
    }

    #[test]
    fn received_return_notice_does_not_claim_a_refund_was_issued() {
        let (subject, body) = super::return_notification_content("received", 42)
            .expect("received return has a customer notice");
        assert_eq!(subject, "Return items received for order #42");
        assert!(body.contains("still subject to review"));
        assert!(body.contains("does not mean a refund was issued"));
    }

    #[test]
    fn unsupported_return_states_do_not_generate_customer_notices() {
        assert!(super::return_notification_content("unknown", 42).is_none());
    }
}

#[cfg(test)]
mod return_lookup_error_tests {
    use super::return_lookup_error;
    use axum::http::StatusCode;

    #[test]
    fn missing_return_maps_to_not_found() {
        let response = return_lookup_error(stheno_types::Error::NotFound {
            entity: "return",
            id: "ret_missing".to_string(),
        });
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn database_failure_is_not_reported_as_a_missing_return() {
        let response = return_lookup_error(stheno_types::Error::Database("offline".to_string()));
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
