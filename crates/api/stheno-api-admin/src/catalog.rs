//! Admin Catalog API handlers (Medusa v2 byte-compatible).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_product::{
    CreateCategoryInput, CreateCollectionInput, CreateProductInput, CreateVariantInput,
    ProductCategoryDTO, ProductCollectionDTO, ProductDTO, ProductService,
    UpdateProductInput,
};

#[derive(Debug, Deserialize, Default)]
pub struct ListCatalogQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub handle: Option<String>,
    pub id: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductOptionPayload {
    pub title: String,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductVariantPayload {
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub options: Option<serde_json::Value>,
    pub prices: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductAdminRequest {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub discountable: Option<bool>,
    pub status: Option<String>,
    pub options: Option<Vec<CreateProductOptionPayload>>,
    pub variants: Option<Vec<CreateProductVariantPayload>>,
    pub sales_channels: Option<Vec<serde_json::Value>>,
    pub shipping_profile_id: Option<String>,
    pub collection_id: Option<String>,
    pub categories: Option<Vec<serde_json::Value>>,
    pub tags: Option<Vec<serde_json::Value>>,
    pub type_id: Option<String>,
}

pub async fn list_products(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListCatalogQuery>,
) -> Result<impl IntoResponse, Response> {
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
            if let Some(q) = &query.q {
                let q_lower = q.to_lowercase();
                if !p.title.to_lowercase().contains(&q_lower)
                    && !p.handle.to_lowercase().contains(&q_lower)
                {
                    return false;
                }
            }
            true
        })
        .collect();

    let count = filtered.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let page: Vec<ProductDTO> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok((
        StatusCode::OK,
        Json(json!({
            "products": page,
            "data": page,
            "count": count,
            "offset": offset,
            "limit": limit,
        })),
    ))
}

pub async fn get_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_product(&id).await {
        Ok(product) => {
            let variants = service.list_variants(&id).await.unwrap_or_default();
            let options_raw = service.list_options(&id).await.unwrap_or_default();
            let options: Vec<serde_json::Value> = options_raw
                .into_iter()
                .map(|(opt_id, opt_title)| {
                    json!({
                        "id": opt_id,
                        "title": opt_title,
                        "product_id": id,
                        "values": []
                    })
                })
                .collect();

            let mut prod_json = serde_json::to_value(&product).unwrap_or(json!({}));
            if let Some(map) = prod_json.as_object_mut() {
                map.insert("variants".to_string(), json!(variants));
                map.insert("options".to_string(), json!(options));
                map.insert("images".to_string(), json!([]));
                map.insert("categories".to_string(), json!([]));
                map.insert("tags".to_string(), json!([]));
                map.insert("sales_channels".to_string(), json!([]));
            }

            Ok((
                StatusCode::OK,
                Json(json!({
                    "product": prod_json,
                    "data": prod_json,
                })),
            ))
        }
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
    Json(input): Json<CreateProductAdminRequest>,
) -> Result<impl IntoResponse, Response> {
    let product = service
        .create_product(CreateProductInput {
            title: input.title.clone(),
            handle: input.handle.clone(),
            subtitle: input.subtitle.clone(),
            description: input.description.clone(),
            discountable: input.discountable,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;

    let prod_id_str = product.id.to_string();

    // Create options if passed
    if let Some(options_list) = input.options {
        for opt in options_list {
            let opt_id = service
                .create_option(&prod_id_str, &opt.title)
                .await
                .unwrap_or_default();
            if let Some(vals) = opt.values {
                for val in vals {
                    let _ = service.create_option_value(&opt_id, &val).await;
                }
            }
        }
    }

    // Create variants if passed
    if let Some(variants_list) = input.variants {
        for v in variants_list {
            let _ = service
                .create_variant(CreateVariantInput {
                    product_id: prod_id_str.clone(),
                    title: v.title,
                    sku: v.sku,
                    barcode: v.barcode,
                    option_value_ids: vec![],
                })
                .await;
        }
    }

    let variants = service
        .list_variants(&prod_id_str)
        .await
        .unwrap_or_default();
    let options_raw = service
        .list_options(&prod_id_str)
        .await
        .unwrap_or_default();
    let options: Vec<serde_json::Value> = options_raw
        .into_iter()
        .map(|(opt_id, opt_title)| {
            json!({
                "id": opt_id,
                "title": opt_title,
                "product_id": prod_id_str,
                "values": []
            })
        })
        .collect();

    let mut prod_json = serde_json::to_value(&product).unwrap_or(json!({}));
    if let Some(map) = prod_json.as_object_mut() {
        map.insert("variants".to_string(), json!(variants));
        map.insert("options".to_string(), json!(options));
        map.insert("images".to_string(), json!([]));
        map.insert("categories".to_string(), json!([]));
        map.insert("tags".to_string(), json!([]));
        map.insert("sales_channels".to_string(), json!([]));
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "product": prod_json,
            "data": prod_json,
        })),
    ))
}

pub async fn update_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateProductInput>,
) -> Result<impl IntoResponse, Response> {
    let product = service.update_product(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "product": product,
            "data": product,
        })),
    ))
}

pub async fn delete_product(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_product(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "product",
            "deleted": true
        })),
    ))
}

// ── Product Categories ─────────────────────────────────────────────────────────

pub async fn list_categories(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListCatalogQuery>,
) -> Result<impl IntoResponse, Response> {
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
    let limit = query.limit.unwrap_or(50);
    let page: Vec<ProductCategoryDTO> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok((
        StatusCode::OK,
        Json(json!({
            "product_categories": page,
            "data": page,
            "count": count,
            "offset": offset,
            "limit": limit,
        })),
    ))
}

pub async fn get_category(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_category(&id).await {
        Ok(cat) => Ok((
            StatusCode::OK,
            Json(json!({
                "product_category": cat,
                "data": cat,
            })),
        )),
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
) -> Result<impl IntoResponse, Response> {
    let cat = service.create_category(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "product_category": cat,
            "data": cat,
        })),
    ))
}

pub async fn delete_category(
    State(_service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "product_category",
            "deleted": true
        })),
    ))
}

// ── Product Collections ───────────────────────────────────────────────────────

pub async fn list_collections(
    State(service): State<Arc<ProductService>>,
    Query(query): Query<ListCatalogQuery>,
) -> Result<impl IntoResponse, Response> {
    let all = service.list_collections().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;

    let count = all.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let page: Vec<ProductCollectionDTO> = all.into_iter().skip(offset).take(limit).collect();

    Ok((
        StatusCode::OK,
        Json(json!({
            "collections": page,
            "data": page,
            "count": count,
            "offset": offset,
            "limit": limit,
        })),
    ))
}

pub async fn get_collection(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_collection(&id).await {
        Ok(col) => Ok((
            StatusCode::OK,
            Json(json!({
                "collection": col,
                "data": col,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Collection {} not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn create_collection(
    State(service): State<Arc<ProductService>>,
    Json(input): Json<CreateCollectionInput>,
) -> Result<impl IntoResponse, Response> {
    let col = service.create_collection(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "collection": col,
            "data": col,
        })),
    ))
}

pub async fn update_collection(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
    Json(input): Json<stheno_product::UpdateCollectionInput>,
) -> Result<impl IntoResponse, Response> {
    let col = service.update_collection(&id, input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "collection": col,
            "data": col,
        })),
    ))
}

pub async fn delete_collection(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_collection(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "collection",
            "deleted": true
        })),
    ))
}

// ── Product Types & Tags ───────────────────────────────────────────────────────

pub async fn list_product_types() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "product_types": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn list_product_tags() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "product_tags": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

// ── Shipping Profiles ─────────────────────────────────────────────────────────

pub async fn list_shipping_profiles() -> Result<impl IntoResponse, Response> {
    let default_profile = json!({
        "id": "sp_default_shipping_profile",
        "name": "Default Shipping Profile",
        "type": "default",
        "created_at": "2026-08-17T17:29:39.266111Z",
        "updated_at": "2026-08-17T17:29:39.266111Z"
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "shipping_profiles": [default_profile],
            "data": [default_profile],
            "count": 1,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

// ── Product Variants & Options ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateOptionRequest {
    pub title: String,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariantRequest {
    pub title: Option<String>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
}

pub async fn list_global_product_options(
    State(service): State<Arc<ProductService>>,
) -> Result<impl IntoResponse, Response> {
    let raw = service.list_all_options().await.unwrap_or_default();
    let mut seen: std::collections::HashMap<String, (String, String, String, Vec<(String, String)>)> =
        std::collections::HashMap::new();

    for (opt_id, opt_title, prod_id) in raw {
        let key = opt_title.trim().to_lowercase();
        let vals = service.list_option_values(&opt_id).await.unwrap_or_default();

        let entry = seen.entry(key).or_insert_with(|| {
            (opt_id.clone(), opt_title.clone(), prod_id.clone(), Vec::new())
        });

        for (val_id, val) in vals {
            if !entry.3.iter().any(|(_, v)| v.eq_ignore_ascii_case(&val)) {
                entry.3.push((val_id, val));
            }
        }
    }

    let mut options = Vec::new();
    for (_key, (opt_id, opt_title, prod_id, vals)) in seen {
        let formatted_vals: Vec<serde_json::Value> = vals
            .into_iter()
            .enumerate()
            .map(|(rank, (val_id, val))| {
                json!({
                    "id": val_id,
                    "value": val,
                    "option_id": opt_id,
                    "rank": rank as i32
                })
            })
            .collect();
        options.push(json!({
            "id": opt_id,
            "title": opt_title,
            "product_id": prod_id,
            "values": formatted_vals
        }));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "product_options": options,
            "data": options,
            "count": options.len(),
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn create_global_product_option(
    State(_service): State<Arc<ProductService>>,
    Json(input): Json<CreateOptionRequest>,
) -> Result<impl IntoResponse, Response> {
    let opt_id = format!("opt_{}", ulid::Ulid::new());
    let formatted_vals: Vec<serde_json::Value> = input
        .values
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(rank, v)| {
            json!({
                "id": format!("optval_{}", ulid::Ulid::new()),
                "value": v,
                "option_id": opt_id,
                "rank": rank as i32
            })
        })
        .collect();

    let opt = json!({
        "id": opt_id,
        "title": input.title,
        "product_id": null,
        "values": formatted_vals
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "product_option": opt,
            "data": opt,
        })),
    ))
}

pub async fn list_product_variants(
    State(service): State<Arc<ProductService>>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let variants = service
        .list_variants(&product_id)
        .await
        .unwrap_or_default();

    Ok((
        StatusCode::OK,
        Json(json!({
            "variants": variants,
            "data": variants,
            "count": variants.len(),
            "offset": 0,
            "limit": 50,
        })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateVariantAdminRequest {
    pub title: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub prices: Option<serde_json::Value>,
    pub options: Option<serde_json::Value>,
    pub manage_inventory: Option<bool>,
    pub allow_backorder: Option<bool>,
}

pub async fn create_variant(
    State(service): State<Arc<ProductService>>,
    Path(product_id): Path<String>,
    Json(input): Json<CreateVariantAdminRequest>,
) -> Result<impl IntoResponse, Response> {
    let variant = service
        .create_variant(CreateVariantInput {
            product_id: product_id.clone(),
            title: input.title,
            sku: input.sku,
            barcode: input.barcode,
            option_value_ids: vec![],
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
        Json(json!({
            "variant": variant,
            "data": variant,
        })),
    ))
}

pub async fn get_variant(
    State(service): State<Arc<ProductService>>,
    Path((_product_id, variant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    match service.get_variant(&variant_id).await {
        Ok(v) => Ok((
            StatusCode::OK,
            Json(json!({
                "variant": v,
                "data": v,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Variant {} was not found", variant_id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_variant(
    State(service): State<Arc<ProductService>>,
    Path((_product_id, variant_id)): Path<(String, String)>,
    Json(input): Json<UpdateVariantRequest>,
) -> Result<impl IntoResponse, Response> {
    let v = service
        .update_variant(&variant_id, input.title, input.sku, input.barcode)
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
            "variant": v,
            "data": v,
        })),
    ))
}

pub async fn delete_variant(
    State(service): State<Arc<ProductService>>,
    Path((_product_id, variant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    service.delete_variant(&variant_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": variant_id,
            "object": "variant",
            "deleted": true
        })),
    ))
}

pub async fn list_product_options(
    State(service): State<Arc<ProductService>>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let options_raw = service
        .list_options(&product_id)
        .await
        .unwrap_or_default();
    let mut options = Vec::new();
    for (opt_id, opt_title) in options_raw {
        let vals = service.list_option_values(&opt_id).await.unwrap_or_default();
        let formatted_vals: Vec<serde_json::Value> = vals
            .into_iter()
            .enumerate()
            .map(|(rank, (val_id, val))| {
                json!({
                    "id": val_id,
                    "value": val,
                    "option_id": opt_id,
                    "rank": rank as i32
                })
            })
            .collect();
        options.push(json!({
            "id": opt_id,
            "title": opt_title,
            "product_id": product_id,
            "values": formatted_vals
        }));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "product_options": options,
            "data": options,
            "count": options.len(),
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn create_product_option(
    State(service): State<Arc<ProductService>>,
    Path(product_id): Path<String>,
    Json(input): Json<CreateOptionRequest>,
) -> Result<impl IntoResponse, Response> {
    let opt_id = service
        .create_option(&product_id, &input.title)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;

    let mut formatted_vals = Vec::new();
    if let Some(vals) = input.values {
        for (rank, v) in vals.into_iter().enumerate() {
            if let Ok(vid) = service.create_option_value(&opt_id, &v).await {
                formatted_vals.push(json!({
                    "id": vid,
                    "value": v,
                    "option_id": opt_id,
                    "rank": rank as i32
                }));
            }
        }
    }

    let opt = json!({
        "id": opt_id,
        "title": input.title,
        "product_id": product_id,
        "values": formatted_vals
    });

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "product_option": opt,
            "data": opt,
        })),
    ))
}

pub async fn get_product_option(
    State(service): State<Arc<ProductService>>,
    Path((product_id, option_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let vals = service.list_option_values(&option_id).await.unwrap_or_default();
    let formatted_vals: Vec<serde_json::Value> = vals
        .into_iter()
        .enumerate()
        .map(|(rank, (val_id, val))| {
            json!({
                "id": val_id,
                "value": val,
                "option_id": option_id,
                "rank": rank as i32
            })
        })
        .collect();

    let opt = json!({
        "id": option_id,
        "product_id": product_id,
        "values": formatted_vals
    });

    Ok((
        StatusCode::OK,
        Json(json!({
            "product_option": opt,
            "data": opt,
        })),
    ))
}

pub async fn delete_product_option(
    State(_service): State<Arc<ProductService>>,
    Path((_product_id, option_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": option_id,
            "object": "product_option",
            "deleted": true
        })),
    ))
}
