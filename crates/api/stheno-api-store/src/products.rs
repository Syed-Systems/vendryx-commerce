//! Store Product API handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_product::{ProductDTO, ProductService};

#[derive(Debug, Deserialize, Default)]
pub struct ListProductsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub handle: Option<String>,
    pub id: Option<String>,
    pub fields: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListProductsResponse {
    pub products: Vec<ProductDTO>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct GetProductResponse {
    pub product: ProductDTO,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub r#type: String,
    pub message: String,
}

pub async fn list_products(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<ListProductsResponse>, Response> {
    let all = service.list_products().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                r#type: "unexpected_state".to_string(),
                message: e.to_string(),
            }),
        )
            .into_response()
    })?;

    let filtered: Vec<ProductDTO> = all
        .into_iter()
        .filter(|p| {
            if let Some(h) = &query.handle {
                if &p.handle != h {
                    return false;
                }
            }
            if let Some(id) = &query.id {
                let id_str = p.id.to_string();
                if &id_str != id && !id_str.ends_with(id) {
                    return false;
                }
            }
            true
        })
        .collect();

    let count = filtered.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20);

    let page: Vec<ProductDTO> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ListProductsResponse {
        products: page,
        count,
        offset,
        limit,
    }))
}

pub async fn get_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<GetProductResponse>, Response> {
    match service.get_product(&id).await {
        Ok(product) => Ok(Json(GetProductResponse { product })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: format!("Product with id {} was not found", id),
            }),
        )
            .into_response()),
    }
}
