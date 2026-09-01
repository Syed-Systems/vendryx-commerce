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
    ProductCategoryDTO, ProductCollectionDTO, ProductDTO, ProductService, UpdateProductInput,
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

/// State for the product-detail routes.
///
/// Products alone are not enough to answer `GET /admin/products/{id}`: the dashboard's
/// variant table reads inventory off each variant, so the handler needs the inventory
/// module too. Kept to these three routes rather than widening the whole catalog router.
#[derive(Clone)]
pub struct ProductDetailContext {
    pub product_service: Arc<ProductService>,
    pub inventory_service: Arc<stheno_inventory::InventoryService>,
}

/// Attaches what the dashboard's variant table reads to each variant.
///
/// `inventory_items` is not optional on the client: it does
/// `variant.inventory_items?.map(..).filter(..)` and then reads `.length` on the result,
/// and optional chaining short-circuits the whole chain — so an absent field became
/// `undefined.length` and took the page down. An empty array is the honest answer for a
/// variant with no inventory item, and it renders.
async fn variants_with_inventory(
    inventory_service: &stheno_inventory::InventoryService,
    variants: &[stheno_product::ProductVariantDTO],
) -> Vec<serde_json::Value> {
    let variant_ids: Vec<String> = variants.iter().map(|v| v.id.to_string()).collect();
    let items = inventory_service
        .items_for_variants(&variant_ids)
        .await
        .unwrap_or_default();
    let availability = inventory_service
        .availability_for_variants(&variant_ids)
        .await
        .unwrap_or_default();

    variants
        .iter()
        .map(|variant| {
            let variant_id = variant.id.to_string();
            let mut value = serde_json::to_value(variant).unwrap_or(json!({}));
            if let Some(map) = value.as_object_mut() {
                let inventory_items: Vec<serde_json::Value> = items
                    .get(&variant_id)
                    .map(|list| {
                        list.iter()
                            .map(|(item, levels)| {
                                let mut inventory =
                                    serde_json::to_value(item).unwrap_or(json!({}));
                                if let Some(inv) = inventory.as_object_mut() {
                                    inv.insert("location_levels".to_string(), json!(levels));
                                }
                                json!({
                                    "inventory": inventory,
                                    "inventory_item_id": item.id.to_string(),
                                    "variant_id": variant_id,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                map.insert("inventory_items".to_string(), json!(inventory_items));
                map.insert(
                    "inventory_quantity".to_string(),
                    json!(availability.get(&variant_id).copied().unwrap_or(0)),
                );
            }
            value
        })
        .collect()
}

pub async fn get_product(
    State(ctx): State<ProductDetailContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let service = &ctx.product_service;
    match service.get_product(&id).await {
        Ok(product) => {
            let variant_dtos = service.list_variants(&id).await.unwrap_or_default();
            let variants = variants_with_inventory(&ctx.inventory_service, &variant_dtos).await;
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
    let options_raw = service.list_options(&prod_id_str).await.unwrap_or_default();
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
    State(ctx): State<ProductDetailContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateProductInput>,
) -> Result<impl IntoResponse, Response> {
    let service = &ctx.product_service;
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
    State(ctx): State<ProductDetailContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.product_service.delete_product(&id).await.map_err(|e| {
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

/// Shared error mappers for this module. Each maps a domain error to the status the
/// dashboard expects, without discarding the message.
fn internal(e: stheno_types::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("unexpected_state", &e.to_string())),
    )
        .into_response()
}

fn not_found(e: stheno_types::Error) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("not_found", &e.to_string())),
    )
        .into_response()
}

fn bad_request(e: stheno_types::Error) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("invalid_data", &e.to_string())),
    )
        .into_response()
}

// ── Product tags and types ────────────────────────────────────────────────────
//
// These four screens in the admin had nothing behind them: the handlers here returned
// hardcoded empty arrays because `ProductService` had no tag or type methods, even
// though `product.product_tag` and `product.product_type` have existed since the first
// product migration.

#[derive(serde::Deserialize)]
pub struct TaxonomyValue {
    pub value: String,
}

pub async fn list_product_types(
    State(service): State<Arc<ProductService>>,
) -> Result<impl IntoResponse, Response> {
    let types = service.list_types().await.map_err(internal)?;
    let count = types.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "product_types": types, "data": types,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_product_type(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let ty = service.get_type(&id).await.map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "product_type": ty, "data": ty })),
    ))
}

pub async fn create_product_type(
    State(service): State<Arc<ProductService>>,
    Json(body): Json<TaxonomyValue>,
) -> Result<impl IntoResponse, Response> {
    let ty = service.create_type(body.value).await.map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "product_type": ty, "data": ty })),
    ))
}

pub async fn update_product_type(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
    Json(body): Json<TaxonomyValue>,
) -> Result<impl IntoResponse, Response> {
    let ty = service
        .update_type(&id, body.value)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "product_type": ty, "data": ty })),
    ))
}

pub async fn delete_product_type(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_type(&id).await.map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "product_type", "deleted": true })),
    ))
}

pub async fn list_product_tags(
    State(service): State<Arc<ProductService>>,
) -> Result<impl IntoResponse, Response> {
    let tags = service.list_tags().await.map_err(internal)?;
    let count = tags.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "product_tags": tags, "data": tags,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_product_tag(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let tag = service.get_tag(&id).await.map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "product_tag": tag, "data": tag })),
    ))
}

pub async fn create_product_tag(
    State(service): State<Arc<ProductService>>,
    Json(body): Json<TaxonomyValue>,
) -> Result<impl IntoResponse, Response> {
    let tag = service.create_tag(body.value).await.map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "product_tag": tag, "data": tag })),
    ))
}

pub async fn update_product_tag(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
    Json(body): Json<TaxonomyValue>,
) -> Result<impl IntoResponse, Response> {
    let tag = service
        .update_tag(&id, body.value)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "product_tag": tag, "data": tag })),
    ))
}

pub async fn delete_product_tag(
    State(service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_tag(&id).await.map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "product_tag", "deleted": true })),
    ))
}

// Shipping profiles moved to `fulfillment_settings.rs`, backed by FulfillmentService.
// The handler here returned a single fabricated profile that was never stored.

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
    pub prices: Option<serde_json::Value>,
}

/// One deduplicated global product option: its id, title, owning product, and values.
type GlobalOption = (String, String, String, Vec<(String, String)>);

pub async fn list_global_product_options(
    State(service): State<Arc<ProductService>>,
) -> Result<impl IntoResponse, Response> {
    let raw = service.list_all_options().await.unwrap_or_default();
    let mut seen: std::collections::HashMap<String, GlobalOption> =
        std::collections::HashMap::new();

    for (opt_id, opt_title, prod_id) in raw {
        let key = opt_title.trim().to_lowercase();
        let vals = service
            .list_option_values(&opt_id)
            .await
            .unwrap_or_default();

        let entry = seen.entry(key).or_insert_with(|| {
            (
                opt_id.clone(),
                opt_title.clone(),
                prod_id.clone(),
                Vec::new(),
            )
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
    State(ctx): State<VariantContext>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let service = &ctx.product_service;
    let variant_dtos = service.list_variants(&product_id).await.unwrap_or_default();

    // This, not `GET /admin/products/{id}`, is what the product page's variant table reads:
    // it asks for `*inventory_items.inventory.location_levels,inventory_quantity` by name.
    // Serving bare variants here is what threw `undefined.length` and blanked the page.
    let variants = variants_with_inventory(&ctx.inventory_service, &variant_dtos).await;

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

/// State for the variant routes, which need pricing as well as products.
///
/// The rest of the catalog stays on `Arc<ProductService>`; only the routes that actually
/// write a price are widened.
#[derive(Clone)]
pub struct VariantContext {
    pub product_service: Arc<ProductService>,
    pub pricing_service: Arc<stheno_pricing::PricingService>,
    pub inventory_service: Arc<stheno_inventory::InventoryService>,
}

/// Reads Medusa's `prices: [{ currency_code, amount }]` payload.
///
/// Tolerant about the amount's JSON type because admin clients send it both ways — a
/// number from the form, a string from an import — and rejecting one of them would look
/// like the price simply not saving.
fn parse_prices(raw: Option<&serde_json::Value>) -> Vec<stheno_pricing::CreatePriceInput> {
    let Some(items) = raw.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|price| {
            let currency_code = price.get("currency_code")?.as_str()?.to_lowercase();
            let amount = price.get("amount").and_then(|a| {
                a.as_f64()
                    .and_then(rust_decimal::Decimal::from_f64_retain)
                    .or_else(|| a.as_str().and_then(|s| s.parse().ok()))
            })?;
            Some(stheno_pricing::CreatePriceInput {
                currency_code,
                amount,
                min_quantity: None,
                max_quantity: None,
                price_list_id: None,
                rules: None,
            })
        })
        .collect()
}

/// Reads Medusa's `options: { "Size": "M" }` payload into option-value ids.
///
/// Resolves against the product's existing options by title and value, creating neither:
/// a variant referring to an option the product does not have is a client error, and
/// inventing one here would let a typo silently create a second "Sizee" option.
async fn resolve_option_value_ids(
    service: &Arc<ProductService>,
    product_id: &str,
    options: Option<&serde_json::Value>,
) -> Vec<String> {
    let Some(map) = options.and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let Ok(product_options) = service.list_options(product_id).await else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (title, value) in map {
        let Some(value) = value.as_str() else {
            continue;
        };
        for (option_id, option_title) in &product_options {
            if option_title.eq_ignore_ascii_case(title) {
                if let Ok(values) = service.list_option_values(option_id).await {
                    if let Some((value_id, _)) = values
                        .into_iter()
                        .find(|(_, v)| v.eq_ignore_ascii_case(value))
                    {
                        out.push(value_id);
                    }
                }
            }
        }
    }
    out
}

pub async fn create_variant(
    State(ctx): State<VariantContext>,
    Path(product_id): Path<String>,
    Json(input): Json<CreateVariantAdminRequest>,
) -> Result<impl IntoResponse, Response> {
    let service = &ctx.product_service;
    // Both of these were deserialized and dropped on the floor: a merchant could set a
    // price and pick options in the admin, get a 200, and have neither stored — the
    // storefront then showed an unpriced variant nothing could select.
    let option_value_ids =
        resolve_option_value_ids(service, &product_id, input.options.as_ref()).await;
    let prices = parse_prices(input.prices.as_ref());

    let variant = service
        .create_variant(CreateVariantInput {
            product_id: product_id.clone(),
            title: input.title,
            sku: input.sku,
            barcode: input.barcode,
            option_value_ids,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;

    if !prices.is_empty() {
        ctx.pricing_service
            .set_variant_prices(&variant.id.to_string(), prices)
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("invalid_data", &e.to_string())),
                )
                    .into_response()
            })?;
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "variant": variant,
            "data": variant,
        })),
    ))
}

pub async fn get_variant(
    State(ctx): State<VariantContext>,
    Path((_product_id, variant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    match ctx.product_service.get_variant(&variant_id).await {
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
    State(ctx): State<VariantContext>,
    Path((_product_id, variant_id)): Path<(String, String)>,
    Json(input): Json<UpdateVariantRequest>,
) -> Result<impl IntoResponse, Response> {
    let prices = parse_prices(input.prices.as_ref());
    if !prices.is_empty() {
        ctx.pricing_service
            .set_variant_prices(&variant_id, prices)
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("invalid_data", &e.to_string())),
                )
                    .into_response()
            })?;
    }

    let v = ctx
        .product_service
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
    State(ctx): State<VariantContext>,
    Path((_product_id, variant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let service = &ctx.product_service;
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
    let options_raw = service.list_options(&product_id).await.unwrap_or_default();
    let mut options = Vec::new();
    for (opt_id, opt_title) in options_raw {
        let vals = service
            .list_option_values(&opt_id)
            .await
            .unwrap_or_default();
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
    let vals = service
        .list_option_values(&option_id)
        .await
        .unwrap_or_default();
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
