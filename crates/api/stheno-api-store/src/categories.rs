//! Store Product Category API handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use stheno_product::{ProductCategoryDTO, ProductService};

use crate::products::ErrorResponse;

#[derive(Debug, Serialize)]
pub struct ListCategoriesResponse {
    pub product_categories: Vec<ProductCategoryDTO>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct GetCategoryResponse {
    pub product_category: ProductCategoryDTO,
}

pub async fn list_categories(
    State(service): State<Arc<ProductService>>,
) -> Result<(axum::http::HeaderMap, Json<ListCategoriesResponse>), Response> {
    let cats = service.list_categories().await.unwrap_or_default();
    let count = cats.len();
    let tenant_id = stheno_db::tenant::current().unwrap_or_default();
    let entities: Vec<_> = cats
        .iter()
        .map(|category| ("category", category.handle.as_str()))
        .collect();
    let headers = crate::cache::public_cache_headers(&tenant_id, &entities, true);
    Ok((
        headers,
        Json(ListCategoriesResponse {
            product_categories: cats,
            count,
            offset: 0,
            limit: 20,
        }),
    ))
}

pub async fn get_category(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<(axum::http::HeaderMap, Json<GetCategoryResponse>), Response> {
    match service.get_category(&id).await {
        Ok(cat) => {
            let tenant_id = stheno_db::tenant::current().unwrap_or_default();
            let headers = crate::cache::public_cache_headers(
                &tenant_id,
                &[("category", cat.handle.as_str())],
                false,
            );
            Ok((
                headers,
                Json(GetCategoryResponse {
                    product_category: cat,
                }),
            ))
        }
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: format!("Product category with id {id} was not found"),
            }),
        )
            .into_response()),
    }
}
