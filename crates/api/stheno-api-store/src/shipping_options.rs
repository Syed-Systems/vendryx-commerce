//! Store Shipping Options API handlers (TASKS P5-22).
use axum::extract::{Query, State};
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
