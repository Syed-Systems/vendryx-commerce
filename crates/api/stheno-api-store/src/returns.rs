//! Store Returns API handlers (TASKS P6-20, F6).
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_flows::ReturnWorkflowEngine;
use stheno_order::{OrderService, ReturnDTO};
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreReturnContext {
    pub order_service: Arc<OrderService>,
    pub return_flows: Arc<ReturnWorkflowEngine>,
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
    Json(payload): Json<CreateReturnRequest>,
) -> Result<(StatusCode, Json<ReturnResponse>), Response> {
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

    Ok((
        StatusCode::CREATED,
        Json(ReturnResponse { r#return: loaded }),
    ))
}

/// `GET /store/returns`
pub async fn list_returns(
    State(ctx): State<StoreReturnContext>,
    Query(query): Query<ListReturnsQuery>,
) -> Result<Json<ReturnListResponse>, Response> {
    let returns = ctx
        .order_service
        .list_returns(query.order_id.as_deref())
        .await
        .map_err(map_error)?;
    let count = returns.len();

    Ok(Json(ReturnListResponse { returns, count }))
}

/// `GET /store/returns/{id}`
pub async fn get_return(
    State(ctx): State<StoreReturnContext>,
    Path(id): Path<String>,
) -> Result<Json<ReturnResponse>, Response> {
    let ret = ctx
        .order_service
        .get_return_dto(&id)
        .await
        .map_err(map_error)?;

    Ok(Json(ReturnResponse { r#return: ret }))
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
