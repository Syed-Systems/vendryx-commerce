//! Store Shipping Options API handlers (TASKS P5-22).
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_fulfillment::{FulfillmentService, ShippingOptionDTO, ShippingOptionFilter};
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreShippingOptionContext {
    pub fulfillment_service: Arc<FulfillmentService>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListShippingOptionsQuery {
    pub cart_id: Option<String>,
    pub service_zone_id: Option<String>,
    pub shipping_profile_id: Option<String>,
    pub is_return: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShippingOptionListResponse {
    pub shipping_options: Vec<ShippingOptionDTO>,
    pub count: usize,
    pub limit: usize,
    pub offset: usize,
}

/// `GET /store/shipping-options`
pub async fn list_shipping_options(
    State(ctx): State<StoreShippingOptionContext>,
    Query(query): Query<ListShippingOptionsQuery>,
) -> Result<Json<ShippingOptionListResponse>, Response> {
    let filter = ShippingOptionFilter {
        service_zone_id: query.service_zone_id,
        shipping_profile_id: query.shipping_profile_id,
        is_return: query.is_return,
        limit: query.limit,
        offset: query.offset,
    };

    let (options, count) = ctx
        .fulfillment_service
        .list_shipping_options(filter)
        .await
        .map_err(map_error)?;

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    Ok(Json(ShippingOptionListResponse {
        shipping_options: options,
        count,
        limit,
        offset,
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

/// Body of `POST /store/shipping-options/{id}/calculate`.
///
/// Medusa sends the cart the quote is for; `data` carries provider-specific input a
/// calculated-price option needs (a pickup point, a service level).
#[derive(Debug, Deserialize)]
pub struct CalculateShippingRequest {
    pub cart_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// POST /store/shipping-options/{id}/calculate
///
/// A `calculated` shipping option has no fixed price — the amount depends on the cart, so
/// the storefront has to ask before it can show a total. Without this route the checkout
/// could list such an option and never quote it.
pub async fn calculate_shipping_option(
    State(ctx): State<StoreShippingOptionContext>,
    Path(id): Path<String>,
    Json(_body): Json<CalculateShippingRequest>,
) -> Result<impl IntoResponse, Response> {
    let amount = ctx
        .fulfillment_service
        .calculate_shipping_price(&id, None)
        .await
        .map_err(map_error)?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "shipping_option": { "id": id, "amount": amount, "is_calculated": true },
            "data": { "id": id, "amount": amount },
        })),
    ))
}
