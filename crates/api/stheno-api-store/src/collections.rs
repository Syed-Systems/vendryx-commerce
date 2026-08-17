//! Store Product Collection API handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_product::{ProductCollectionDTO, ProductService};

use crate::products::ErrorResponse;

#[derive(Debug, Deserialize, Default)]
pub struct ListCollectionsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub handle: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListCollectionsResponse {
    pub collections: Vec<ProductCollectionDTO>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct GetCollectionResponse {
    pub collection: ProductCollectionDTO,
}

pub async fn list_collections(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListCollectionsQuery>,
) -> Result<Json<ListCollectionsResponse>, Response> {
    let all = service.list_collections().await.unwrap_or_default();

    let filtered: Vec<ProductCollectionDTO> = all
        .into_iter()
        .filter(|c| query.handle.as_ref().map_or(true, |h| &c.handle == h))
        .collect();

    let count = filtered.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20);

    Ok(Json(ListCollectionsResponse {
        collections: filtered.into_iter().skip(offset).take(limit).collect(),
        count,
        offset,
        limit,
    }))
}

pub async fn get_collection(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<GetCollectionResponse>, Response> {
    match service.get_collection(&id).await {
        Ok(collection) => Ok(Json(GetCollectionResponse { collection })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: format!("Collection with id {} was not found", id),
            }),
        )
            .into_response()),
    }
}
