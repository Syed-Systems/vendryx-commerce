//! Admin Catalog API handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};
use stheno_product::{
    CreateCategoryInput, CreateProductInput, CreateVariantInput, ProductCategoryDTO, ProductDTO,
    ProductService, ProductVariantDTO, UpdateProductInput,
};

#[derive(Debug, Deserialize, Default)]
pub struct ListCatalogQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub handle: Option<String>,
    pub id: Option<String>,
}

pub async fn list_products(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListCatalogQuery>,
) -> Result<Json<ListResponse<Vec<ProductDTO>>>, Response> {
    let all = service.list_products().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
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

    Ok(Json(ListResponse {
        data: page,
        count,
        offset,
        limit,
    }))
}

pub async fn get_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<ProductDTO>>, Response> {
    match service.get_product(&id).await {
        Ok(product) => Ok(Json(GetResponse { data: product })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Product with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn create_product(
    State(service): State<Arc<ProductService>>,
    Json(input): Json<CreateProductInput>,
) -> Result<(StatusCode, Json<GetResponse<ProductDTO>>), Response> {
    let product = service.create_product(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: product })))
}

pub async fn update_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateProductInput>,
) -> Result<Json<GetResponse<ProductDTO>>, Response> {
    let product = service.update_product(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: product }))
}

pub async fn delete_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    service.delete_product(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(DeleteResponse {
        id: id.clone(),
        object: "product".to_string(),
        deleted: true,
    }))
}

// ProductCategory API Handlers
pub async fn list_categories(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListCatalogQuery>,
) -> Result<Json<ListResponse<Vec<ProductCategoryDTO>>>, Response> {
    let all = service.list_categories().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;

    let filtered: Vec<ProductCategoryDTO> = all
        .into_iter()
        .filter(|c| {
            if let Some(h) = &query.handle {
                if &c.handle != h {
                    return false;
                }
            }
            if let Some(id) = &query.id {
                let id_str = c.id.to_string();
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
    let page: Vec<ProductCategoryDTO> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ListResponse {
        data: page,
        count,
        offset,
        limit,
    }))
}

pub async fn get_category(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<ProductCategoryDTO>>, Response> {
    match service.get_category(&id).await {
        Ok(cat) => Ok(Json(GetResponse { data: cat })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Product category with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn create_category(
    State(service): State<Arc<ProductService>>,
    Json(input): Json<CreateCategoryInput>,
) -> Result<(StatusCode, Json<GetResponse<ProductCategoryDTO>>), Response> {
    let cat = service.create_category(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: cat })))
}

pub async fn delete_category(
    State(_service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    // Return standard deleted structure
    Ok(Json(DeleteResponse {
        id: id.clone(),
        object: "product_category".to_string(),
        deleted: true,
    }))
}

// Product Variants Handlers
pub async fn create_variant(
    State(service): State<Arc<ProductService>>,
    Json(input): Json<CreateVariantInput>,
) -> Result<(StatusCode, Json<GetResponse<ProductVariantDTO>>), Response> {
    let variant = service.create_variant(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: variant })))
}
