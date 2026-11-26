//! Store Product API handlers (Medusa v2 byte-compatible).

use axum::Json;
use axum::extract::{Path, Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_pricing::PricingService;
use stheno_product::models::ProductStatus;
use stheno_product::{ProductDTO, ProductService};
use stheno_region::RegionService;

use crate::product_expand::{StoreProduct, expand_products};

/// The services a catalog response needs.
///
/// `store_catalog_routes` used to take `Arc<ProductService>` alone, which is precisely why
/// products came back without prices — the handler had no way to reach the pricing module
/// even though it was fully implemented.
#[derive(Clone)]
pub struct CatalogContext {
    pub product_service: Arc<ProductService>,
    pub pricing_service: Arc<PricingService>,
    pub region_service: Arc<RegionService>,
    pub inventory_service: Arc<stheno_inventory::InventoryService>,
    pub cache_service: Option<Arc<stheno_caching::CacheService>>,
}

impl CatalogContext {
    pub fn new(
        product_service: Arc<ProductService>,
        pricing_service: Arc<PricingService>,
        region_service: Arc<RegionService>,
        inventory_service: Arc<stheno_inventory::InventoryService>,
    ) -> Self {
        Self {
            product_service,
            pricing_service,
            region_service,
            inventory_service,
            cache_service: Some(Arc::new(stheno_caching::CacheService::new(Arc::new(
                stheno_caching::InMemoryCacheProvider::new(),
            )))),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ListProductsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub handle: Option<String>,
    pub id: Option<String>,
    pub fields: Option<String>,
    /// Prices depend on it, so the storefront sends it on every catalog call.
    pub region_id: Option<String>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct GetProductQuery {
    pub region_id: Option<String>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListProductsResponse {
    pub products: Vec<StoreProduct>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetProductResponse {
    pub product: StoreProduct,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub r#type: String,
    pub message: String,
}

fn internal(e: impl std::fmt::Display) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            r#type: "unexpected_state".to_string(),
            message: e.to_string(),
        }),
    )
        .into_response()
}

/// The 404 the store returns for a product it will not serve.
///
/// Shared by "no such id" and "exists but is not published": a shopper must not be able to
/// tell those apart, or the catalog leaks the existence of unreleased products.
fn product_not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            r#type: "not_found".to_string(),
            message: format!("Product with id {id} was not found"),
        }),
    )
        .into_response()
}

/// Only published products are public.
///
/// The store API had no status filter at all, so every draft, proposed and rejected
/// product was served to shoppers the moment it was created — a merchant drafting a
/// product published it by accident. Admin keeps seeing every status; this is the store
/// boundary, so it is enforced here rather than in ProductService.
fn is_public(p: &ProductDTO) -> bool {
    p.status == ProductStatus::Published.as_str()
}

/// Resolves the currency to price in.
///
/// The region is authoritative when given — a shopper browsing the EU store sees euros
/// regardless of what the query string says. `currency_code` is the fallback for clients
/// that have no region yet, and "usd" the last resort so a catalog still renders.
async fn resolve_currency(
    ctx: &CatalogContext,
    region_id: Option<&String>,
    explicit: Option<String>,
) -> String {
    if let Some(region_id) = region_id {
        if let Ok(region) = ctx.region_service.get_region(region_id).await {
            return region.currency_code.to_lowercase();
        }
    }
    explicit
        .map(|c| c.to_lowercase())
        .unwrap_or_else(|| "usd".to_string())
}

/// Collects a repeated or bracketed query parameter.
///
/// `@medusajs/js-sdk` sends list filters as `id[]=x` (and `id[0]=x` for multiples), which
/// `serde_urlencoded` — what `Query` uses — cannot express at all. The field simply stayed
/// `None`, and a `None` filter means *no filter*: `GET /store/products?id[]=X` returned the
/// entire catalogue.
///
/// That was not a cosmetic parsing miss. The product page fetches its add-to-cart panel
/// with `listProducts({ queryParams: { id: [id] } })` and takes `products[0]`, so on any
/// store with more than one product the panel bound to whichever product happened to sort
/// first — its variants, its price, its stock — while the page around it showed a different
/// product. A shopper could add an item they were not looking at.
fn multi_param(raw: Option<&str>, name: &str) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    form_urlencoded::parse(raw.as_bytes())
        .filter(|(key, _)| {
            key == name
                || key
                    .strip_prefix(name)
                    .is_some_and(|rest| rest == "[]" || rest.starts_with('['))
        })
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

pub async fn list_products(
    State(ctx): State<CatalogContext>,
    RawQuery(raw_query): RawQuery,
    Query(query): Query<ListProductsQuery>,
) -> Result<(axum::http::HeaderMap, Json<ListProductsResponse>), Response> {
    // Both spellings, so `?id=x` and `?id[]=x` behave the same.
    let mut ids = multi_param(raw_query.as_deref(), "id");
    ids.extend(query.id.clone());
    let mut handles = multi_param(raw_query.as_deref(), "handle");
    handles.extend(query.handle.clone());

    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20);
    let currency = resolve_currency(&ctx, query.region_id.as_ref(), query.currency_code).await;
    let tenant_id = stheno_db::tenant::current().unwrap_or_default();
    let cache_enabled = !tenant_id.is_empty();

    let cache_key = format!(
        "catalog:{}:products:list:{}:{}:{}:{}:{}:{}",
        tenant_id,
        offset,
        limit,
        currency,
        query.region_id.as_deref().unwrap_or(""),
        ids.join(","),
        handles.join(",")
    );

    if cache_enabled {
        if let Some(ref cache) = ctx.cache_service {
            if let Ok(Some(cached)) = cache.get_json::<ListProductsResponse>(&cache_key).await {
                let entities: Vec<_> = cached
                    .products
                    .iter()
                    .map(|product| ("product", product.product.handle.as_str()))
                    .collect();
                let headers = crate::cache::public_cache_headers(&tenant_id, &entities, true);
                return Ok((headers, Json(cached)));
            }
        }
    }

    let all = ctx
        .product_service
        .list_products()
        .await
        .map_err(internal)?;

    let filtered: Vec<ProductDTO> = all
        .into_iter()
        .filter(|p| {
            if !is_public(p) {
                return false;
            }
            if !handles.is_empty() && !handles.iter().any(|h| h == &p.handle) {
                return false;
            }
            if !ids.is_empty() {
                let id_str = p.id.to_string();
                if !ids.iter().any(|id| &id_str == id || id_str.ends_with(id)) {
                    return false;
                }
            }
            true
        })
        .collect();

    let count = filtered.len();

    // Expansion runs on the page, never the whole catalog: it costs a price evaluation
    // per variant, and paying that for rows nobody asked for is how a catalog endpoint
    // becomes the slowest thing in the product.
    let page: Vec<ProductDTO> = filtered.into_iter().skip(offset).take(limit).collect();

    let products = expand_products(
        page,
        &ctx.product_service,
        &ctx.pricing_service,
        &ctx.inventory_service,
        &currency,
        query.region_id,
    )
    .await
    .map_err(internal)?;

    let response = ListProductsResponse {
        products,
        count,
        offset,
        limit,
    };

    if cache_enabled {
        if let Some(ref cache) = ctx.cache_service {
            let _ = cache.set_json(&cache_key, &response, Some(300)).await;
        }
    }

    let entities: Vec<_> = response
        .products
        .iter()
        .map(|product| ("product", product.product.handle.as_str()))
        .collect();
    let headers = crate::cache::public_cache_headers(&tenant_id, &entities, true);

    Ok((headers, Json(response)))
}

pub async fn get_product(
    State(ctx): State<CatalogContext>,
    Path(id): Path<String>,
    Query(query): Query<GetProductQuery>,
) -> Result<(axum::http::HeaderMap, Json<GetProductResponse>), Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_default();
    let cache_enabled = !tenant_id.is_empty();
    let currency = resolve_currency(&ctx, query.region_id.as_ref(), query.currency_code).await;
    let cache_key = format!(
        "catalog:{}:products:get:{}:{}:{}",
        tenant_id,
        id,
        currency,
        query.region_id.as_deref().unwrap_or("")
    );

    if cache_enabled {
        if let Some(ref cache) = ctx.cache_service {
            if let Ok(Some(cached)) = cache.get_json::<GetProductResponse>(&cache_key).await {
                let headers = crate::cache::public_cache_headers(
                    &tenant_id,
                    &[("product", cached.product.product.handle.as_str())],
                    false,
                );
                return Ok((headers, Json(cached)));
            }
        }
    }

    let product = ctx
        .product_service
        .get_product(&id)
        .await
        .map_err(|_| product_not_found(&id))?;

    if !is_public(&product) {
        return Err(product_not_found(&id));
    }

    let mut expanded = expand_products(
        vec![product],
        &ctx.product_service,
        &ctx.pricing_service,
        &ctx.inventory_service,
        &currency,
        query.region_id,
    )
    .await
    .map_err(internal)?;

    let response = GetProductResponse {
        product: expanded.remove(0),
    };

    if cache_enabled {
        if let Some(ref cache) = ctx.cache_service {
            let _ = cache.set_json(&cache_key, &response, Some(300)).await;
        }
    }

    let headers = crate::cache::public_cache_headers(
        &tenant_id,
        &[("product", response.product.product.handle.as_str())],
        false,
    );

    Ok((headers, Json(response)))
}

#[derive(Debug, Deserialize, Default)]
pub struct ListProductOptionsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub product_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListProductOptionsResponse {
    pub product_options: Vec<stheno_product::ProductOptionDTO>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct GetProductOptionResponse {
    pub product_option: stheno_product::ProductOptionDTO,
}

/// `GET /store/product-options` — list product options
pub async fn list_product_options(
    State(product_service): State<Arc<ProductService>>,
    Query(query): Query<ListProductOptionsQuery>,
) -> Result<Json<ListProductOptionsResponse>, Response> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let (options, count) = product_service
        .list_product_options_paginated(query.product_id.as_deref(), limit, offset)
        .await
        .map_err(internal)?;

    Ok(Json(ListProductOptionsResponse {
        product_options: options,
        count,
        offset,
        limit,
    }))
}

/// `GET /store/product-options/{id}` — get product option by id
pub async fn get_product_option(
    State(product_service): State<Arc<ProductService>>,
    Path(id): Path<String>,
) -> Result<Json<GetProductOptionResponse>, Response> {
    let option = product_service.get_option(&id).await.map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: format!("Product option with id {id} was not found"),
            }),
        )
            .into_response()
    })?;

    Ok(Json(GetProductOptionResponse {
        product_option: option,
    }))
}
