//! Admin Configuration API Handlers — regions, currencies, tax, sales channels, store settings.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};
use stheno_currency::{CurrencyDTO, CurrencyService};
use stheno_region::{CreateRegionInput, RegionDTO, RegionService, UpdateRegionInput};
use stheno_sales_channel::{CreateSalesChannelInput, SalesChannelDTO, SalesChannelService};
use stheno_tax::{
    CreateTaxRateInput, CreateTaxRegionInput, TaxRateDTO, TaxRegionDTO, TaxService,
    UpdateTaxRateInput, UpdateTaxRegionInput,
};

/// Combined state for all config-related services.
#[derive(Clone)]
pub struct ConfigContext {
    pub region_service: Arc<RegionService>,
    pub currency_service: Arc<CurrencyService>,
    pub tax_service: Arc<TaxService>,
    pub sales_channel_service: Arc<SalesChannelService>,
}

// ── Regions ──────────────────────────────────────────────────────────────────

pub async fn list_regions(
    State(ctx): State<ConfigContext>,
) -> Result<Json<ListResponse<Vec<RegionDTO>>>, Response> {
    let all = ctx.region_service.list_regions().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("unexpected_state", &e.to_string()))).into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse { data: all, count, offset: 0, limit: 20 }))
}

pub async fn create_region(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateRegionInput>,
) -> Result<(StatusCode, Json<GetResponse<RegionDTO>>), Response> {
    let region = ctx.region_service.create_region(input).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("invalid_data", &e.to_string()))).into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: region })))
}

pub async fn get_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<RegionDTO>>, Response> {
    match ctx.region_service.get_region(&id).await {
        Ok(r) => Ok(Json(GetResponse { data: r })),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &format!("Region {} not found", id)))).into_response()),
    }
}

pub async fn update_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateRegionInput>,
) -> Result<Json<GetResponse<RegionDTO>>, Response> {
    let r = ctx.region_service.update_region(&id, input).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(GetResponse { data: r }))
}

pub async fn delete_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    ctx.region_service.delete_region(&id).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(DeleteResponse { id, object: "region".to_string(), deleted: true }))
}

// ── Currencies ───────────────────────────────────────────────────────────────

pub async fn list_currencies(
    State(ctx): State<ConfigContext>,
) -> Result<Json<ListResponse<Vec<CurrencyDTO>>>, Response> {
    let all = ctx.currency_service.list_currencies().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("unexpected_state", &e.to_string()))).into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse { data: all, count, offset: 0, limit: 20 }))
}

pub async fn get_currency(
    State(ctx): State<ConfigContext>,
    Path(code): Path<String>,
) -> Result<Json<GetResponse<CurrencyDTO>>, Response> {
    match ctx.currency_service.get_currency(&code).await {
        Ok(c) => Ok(Json(GetResponse { data: c })),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &format!("Currency {} not found", code)))).into_response()),
    }
}

// ── Tax Regions & Rates ──────────────────────────────────────────────────────

pub async fn create_tax_region(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateTaxRegionInput>,
) -> Result<(StatusCode, Json<GetResponse<TaxRegionDTO>>), Response> {
    let tr = ctx.tax_service.create_tax_region(input).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("invalid_data", &e.to_string()))).into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: tr })))
}

pub async fn get_tax_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<TaxRegionDTO>>, Response> {
    match ctx.tax_service.get_tax_region(&id).await {
        Ok(r) => Ok(Json(GetResponse { data: r })),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &format!("Tax region {} not found", id)))).into_response()),
    }
}

pub async fn update_tax_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTaxRegionInput>,
) -> Result<Json<GetResponse<TaxRegionDTO>>, Response> {
    let r = ctx.tax_service.update_tax_region(&id, input).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(GetResponse { data: r }))
}

pub async fn delete_tax_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    ctx.tax_service.delete_tax_region(&id).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(DeleteResponse { id, object: "tax_region".to_string(), deleted: true }))
}

pub async fn create_tax_rate(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateTaxRateInput>,
) -> Result<(StatusCode, Json<GetResponse<TaxRateDTO>>), Response> {
    let rate = ctx.tax_service.create_tax_rate(input).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("invalid_data", &e.to_string()))).into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: rate })))
}

pub async fn get_tax_rate(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<TaxRateDTO>>, Response> {
    match ctx.tax_service.get_tax_rate(&id).await {
        Ok(r) => Ok(Json(GetResponse { data: r })),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &format!("Tax rate {} not found", id)))).into_response()),
    }
}

pub async fn update_tax_rate(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTaxRateInput>,
) -> Result<Json<GetResponse<TaxRateDTO>>, Response> {
    let r = ctx.tax_service.update_tax_rate(&id, input).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(GetResponse { data: r }))
}

pub async fn delete_tax_rate(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    ctx.tax_service.delete_tax_rate(&id).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(DeleteResponse { id, object: "tax_rate".to_string(), deleted: true }))
}

// ── Sales Channels ───────────────────────────────────────────────────────────

pub async fn list_sales_channels(
    State(ctx): State<ConfigContext>,
) -> Result<Json<ListResponse<Vec<SalesChannelDTO>>>, Response> {
    let all = ctx.sales_channel_service.list_channels().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new("unexpected_state", &e.to_string()))).into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse { data: all, count, offset: 0, limit: 20 }))
}

pub async fn create_sales_channel(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateSalesChannelInput>,
) -> Result<(StatusCode, Json<GetResponse<SalesChannelDTO>>), Response> {
    let sc = ctx.sales_channel_service.create_channel(input).await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(ErrorResponse::new("invalid_data", &e.to_string()))).into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: sc })))
}

pub async fn get_sales_channel(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<SalesChannelDTO>>, Response> {
    match ctx.sales_channel_service.get_channel(&id).await {
        Ok(sc) => Ok(Json(GetResponse { data: sc })),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &format!("Sales channel {} not found", id)))).into_response()),
    }
}

pub async fn delete_sales_channel(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    ctx.sales_channel_service.delete_channel(&id).await.map_err(|e| {
        (StatusCode::NOT_FOUND, Json(ErrorResponse::new("not_found", &e.to_string()))).into_response()
    })?;
    Ok(Json(DeleteResponse { id, object: "sales_channel".to_string(), deleted: true }))
}
