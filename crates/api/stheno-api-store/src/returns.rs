//! Store Returns API handlers (TASKS P6-20, F6).
use axum::extract::{Path, Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_flows::ReturnWorkflowEngine;
use stheno_notification::{NotificationService, SendNotificationInput};
use stheno_order::{OrderFilter, OrderService, ReturnDTO};
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreReturnContext {
    pub order_service: Arc<OrderService>,
    pub return_flows: Arc<ReturnWorkflowEngine>,
    pub auth_service: Arc<AuthService>,
    pub notification_service: Arc<NotificationService>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListReturnsQuery {
    pub order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReturnRequest {
    pub order_id: String,
    pub location_id: Option<String>,
    pub items: Vec<CreateReturnItemRequest>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReturnItemRequest {
    pub id: String, // Line item id
    pub quantity: i32,
    pub reason_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReturnResponse {
    pub r#return: ReturnDTO,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReturnListResponse {
    pub returns: Vec<ReturnDTO>,
    pub count: usize,
}

/// `POST /store/returns`
pub async fn create_return(
    State(ctx): State<StoreReturnContext>,
    headers: HeaderMap,
    Json(payload): Json<CreateReturnRequest>,
) -> Result<(StatusCode, Json<ReturnResponse>), Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    let order = owned_order(&ctx, &customer_id, &payload.order_id).await?;
    let ret = ctx
        .return_flows
        .request_return(
            &payload.order_id,
            payload.location_id.as_deref(),
            None,
            payload.metadata,
        )
        .await
        .map_err(map_error)?;

    for item in payload.items {
        ctx.return_flows
            .add_return_item(
                &ret.id,
                &item.id,
                item.quantity,
                item.reason_id.as_deref(),
                item.note.as_deref(),
            )
            .await
            .map_err(map_error)?;
    }

    let loaded = ctx
        .order_service
        .get_return_dto(&ret.id)
        .await
        .map_err(map_error)?;

    if let Some(email) = order.email.filter(|email| email.contains('@')) {
        let queued = ctx
            .notification_service
            .enqueue_notification_idempotent(
                SendNotificationInput {
                    to: email.clone(),
                    channel: "email".to_string(),
                    template: None,
                    subject: Some(format!(
                        "Return request received for order #{}",
                        order.display_id
                    )),
                    body: Some(format!(
                        "<p>We received your return request for order <strong>#{}</strong>.</p>\
                         <p>The store will review it. This message does not mean a refund has been issued.</p>",
                        order.display_id
                    )),
                    data: None,
                    trigger_type: Some("order.return_requested".to_string()),
                    resource_id: Some(order.id.clone()),
                    resource_type: Some("order".to_string()),
                    receiver_id: Some(email),
                },
                &format!("order-return-requested/{}", loaded.id),
            )
            .await;
        if queued.is_err() {
            tracing::warn!(
                order_id = %order.id,
                return_id = %loaded.id,
                "return request was saved but its customer email could not be queued"
            );
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(ReturnResponse { r#return: loaded }),
    ))
}

/// `GET /store/returns`
pub async fn list_returns(
    State(ctx): State<StoreReturnContext>,
    headers: HeaderMap,
    Query(query): Query<ListReturnsQuery>,
) -> Result<Json<ReturnListResponse>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    let returns = if let Some(order_id) = query.order_id.as_deref() {
        owned_order(&ctx, &customer_id, order_id).await?;
        ctx.order_service
            .list_returns(Some(order_id))
            .await
            .map_err(map_error)?
    } else {
        let mut visible = Vec::new();
        let mut offset = 0;
        let page_size = 100;
        loop {
            let (orders, total) = ctx
                .order_service
                .list_orders(OrderFilter {
                    customer_id: Some(customer_id.clone()),
                    limit: Some(page_size),
                    offset: Some(offset),
                    ..OrderFilter::default()
                })
                .await
                .map_err(map_error)?;
            if orders.is_empty() {
                break;
            }
            for order in orders {
                visible.extend(
                    ctx.order_service
                        .list_returns(Some(&order.id))
                        .await
                        .map_err(map_error)?,
                );
            }
            offset += page_size;
            if offset >= total {
                break;
            }
        }
        visible
    };
    let count = returns.len();

    Ok(Json(ReturnListResponse { returns, count }))
}

/// `GET /store/returns/{id}`
pub async fn get_return(
    State(ctx): State<StoreReturnContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ReturnResponse>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    let ret = ctx
        .order_service
        .get_return_dto(&id)
        .await
        .map_err(map_error)?;
    owned_order(&ctx, &customer_id, &ret.order_id).await?;

    Ok(Json(ReturnResponse { r#return: ret }))
}

async fn current_customer_id(
    ctx: &StoreReturnContext,
    headers: &HeaderMap,
) -> Result<String, Response> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;
    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;
    claims
        .actor_id
        .ok_or_else(|| unauthorized("Token is not bound to a customer"))
}

async fn owned_order(
    ctx: &StoreReturnContext,
    customer_id: &str,
    order_id: &str,
) -> Result<stheno_order::OrderDTO, Response> {
    let order = ctx
        .order_service
        .get_order(order_id)
        .await
        .map_err(|_| not_found())?;
    if order.customer_id.as_deref() != Some(customer_id) {
        return Err(not_found());
    }
    Ok(order)
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            message: message.to_string(),
            r#type: "unauthorized".to_string(),
        }),
    )
        .into_response()
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            message: "Return or order was not found".to_string(),
            r#type: "not_found".to_string(),
        }),
    )
        .into_response()
}

fn map_error(e: Error) -> Response {
    let (status, msg) = match &e {
        Error::NotFound { entity, id } => (
            StatusCode::NOT_FOUND,
            format!("{entity} with id {id} not found"),
        ),
        Error::Conflict { code: _, message } => (StatusCode::CONFLICT, message.clone()),
        Error::Invalid { message, .. } => (StatusCode::BAD_REQUEST, message.clone()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    (
        status,
        Json(ErrorResponse {
            message: msg,
            r#type: "invalid_request_error".to_string(),
        }),
    )
        .into_response()
}
