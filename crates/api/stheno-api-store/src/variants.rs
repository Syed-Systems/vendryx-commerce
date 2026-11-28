//! Store Product Variant API handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use stheno_product::{ProductService, ProductVariantDTO};

use crate::products::ErrorResponse;

#[derive(Debug, Serialize)]
pub struct GetVariantResponse {
    pub variant: ProductVariantDTO,
}

pub async fn get_variant(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<GetVariantResponse>, Response> {
    match service.get_variant(&id).await {
        Ok(variant) => Ok(Json(GetVariantResponse { variant })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: format!("Product variant with id {id} was not found"),
            }),
        )
            .into_response()),
    }
}
