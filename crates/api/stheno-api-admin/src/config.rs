//! Admin Configuration API Handlers — regions, currencies, tax, sales channels, store settings.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_currency::CurrencyService;
use stheno_region::{CreateRegionInput, RegionService, UpdateRegionInput};
use stheno_sales_channel::{CreateSalesChannelInput, SalesChannelService};
use stheno_tax::{
    CreateTaxRateInput, CreateTaxRegionInput, TaxService, UpdateTaxRateInput, UpdateTaxRegionInput,
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

pub async fn list_regions(State(ctx): State<ConfigContext>) -> Result<impl IntoResponse, Response> {
    let all = ctx.region_service.list_regions().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "regions": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_region(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateRegionInput>,
) -> Result<impl IntoResponse, Response> {
    let region = ctx.region_service.create_region(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "region": region,
            "data": region,
        })),
    ))
}

pub async fn get_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match ctx.region_service.get_region(&id).await {
        Ok(r) => Ok((
            StatusCode::OK,
            Json(json!({
                "region": r,
                "data": r,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Region {} not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateRegionInput>,
) -> Result<impl IntoResponse, Response> {
    let r = ctx
        .region_service
        .update_region(&id, input)
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
            "region": r,
            "data": r,
        })),
    ))
}

pub async fn delete_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.region_service.delete_region(&id).await.map_err(|e| {
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
            "object": "region",
            "deleted": true
        })),
    ))
}

// ── Currencies ───────────────────────────────────────────────────────────────

pub async fn list_currencies(
    State(ctx): State<ConfigContext>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx.currency_service.list_currencies().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "currencies": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn get_currency(
    State(ctx): State<ConfigContext>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match ctx.currency_service.get_currency(&code).await {
        Ok(c) => Ok((
            StatusCode::OK,
            Json(json!({
                "currency": c,
                "data": c,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Currency {} not found", code),
            )),
        )
            .into_response()),
    }
}

// ── Tax Regions & Rates ──────────────────────────────────────────────────────

pub async fn list_tax_regions(
    State(ctx): State<ConfigContext>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx.tax_service.list_tax_regions().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let all_rates = ctx.tax_service.list_tax_rates().await.unwrap_or_default();

    let regions_with_rates: Vec<serde_json::Value> = all
        .into_iter()
        .map(|r| {
            let reg_id_str = r.id.to_string();
            let rates: Vec<&stheno_tax::TaxRateDTO> = all_rates
                .iter()
                .filter(|rate| rate.tax_region_id == reg_id_str)
                .collect();
            let mut v = serde_json::to_value(&r).unwrap_or(json!({}));
            if let Some(obj) = v.as_object_mut() {
                obj.insert("tax_rates".to_string(), json!(rates));
                obj.insert("children".to_string(), json!([]));
            }
            v
        })
        .collect();

    let count = regions_with_rates.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "tax_regions": regions_with_rates,
            "data": regions_with_rates,
            "count": count,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn list_tax_rates(
    State(ctx): State<ConfigContext>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx.tax_service.list_tax_rates().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "tax_rates": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn create_tax_region(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateTaxRegionInput>,
) -> Result<impl IntoResponse, Response> {
    let tr = ctx
        .tax_service
        .create_tax_region(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;
    let mut v = serde_json::to_value(&tr).unwrap_or(json!({}));
    if let Some(obj) = v.as_object_mut() {
        obj.insert("tax_rates".to_string(), json!([]));
        obj.insert("children".to_string(), json!([]));
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "tax_region": v,
            "data": v,
        })),
    ))
}

pub async fn get_tax_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match ctx.tax_service.get_tax_region(&id).await {
        Ok(r) => {
            let all_rates = ctx.tax_service.list_tax_rates().await.unwrap_or_default();
            let reg_id_str = r.id.to_string();
            let rates: Vec<&stheno_tax::TaxRateDTO> = all_rates
                .iter()
                .filter(|rate| rate.tax_region_id == reg_id_str)
                .collect();
            let mut v = serde_json::to_value(&r).unwrap_or(json!({}));
            if let Some(obj) = v.as_object_mut() {
                obj.insert("tax_rates".to_string(), json!(rates));
                obj.insert("children".to_string(), json!([]));
            }
            Ok((
                StatusCode::OK,
                Json(json!({
                    "tax_region": v,
                    "data": v,
                })),
            ))
        }
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Tax region {} not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_tax_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTaxRegionInput>,
) -> Result<impl IntoResponse, Response> {
    let r = ctx
        .tax_service
        .update_tax_region(&id, input)
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
            "tax_region": r,
            "data": r,
        })),
    ))
}

pub async fn delete_tax_region(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.tax_service.delete_tax_region(&id).await.map_err(|e| {
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
            "object": "tax_region",
            "deleted": true
        })),
    ))
}

pub async fn create_tax_rate(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateTaxRateInput>,
) -> Result<impl IntoResponse, Response> {
    let rate = ctx.tax_service.create_tax_rate(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "tax_rate": rate,
            "data": rate,
        })),
    ))
}

pub async fn get_tax_rate(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match ctx.tax_service.get_tax_rate(&id).await {
        Ok(r) => Ok((
            StatusCode::OK,
            Json(json!({
                "tax_rate": r,
                "data": r,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Tax rate {} not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_tax_rate(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTaxRateInput>,
) -> Result<impl IntoResponse, Response> {
    let r = ctx
        .tax_service
        .update_tax_rate(&id, input)
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
            "tax_rate": r,
            "data": r,
        })),
    ))
}

pub async fn delete_tax_rate(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.tax_service.delete_tax_rate(&id).await.map_err(|e| {
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
            "object": "tax_rate",
            "deleted": true
        })),
    ))
}

// ── Sales Channels ───────────────────────────────────────────────────────────

pub async fn list_sales_channels(
    State(ctx): State<ConfigContext>,
) -> Result<impl IntoResponse, Response> {
    let all = ctx
        .sales_channel_service
        .list_channels()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("unexpected_state", &e.to_string())),
            )
                .into_response()
        })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "sales_channels": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_sales_channel(
    State(ctx): State<ConfigContext>,
    Json(input): Json<CreateSalesChannelInput>,
) -> Result<impl IntoResponse, Response> {
    let sc = ctx
        .sales_channel_service
        .create_channel(input)
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
            "sales_channel": sc,
            "data": sc,
        })),
    ))
}

pub async fn get_sales_channel(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match ctx.sales_channel_service.get_channel(&id).await {
        Ok(sc) => Ok((
            StatusCode::OK,
            Json(json!({
                "sales_channel": sc,
                "data": sc,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Sales channel {} not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn delete_sales_channel(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.sales_channel_service
        .delete_channel(&id)
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
            "id": id,
            "object": "sales_channel",
            "deleted": true
        })),
    ))
}

// ── Shipping Options & Fulfillment Sets ───────────────────────────────────────

pub async fn list_shipping_options() -> Result<impl IntoResponse, Response> {
    let standard = json!({
        "id": "so_standard_shipping",
        "name": "Standard Delivery (3-5 days)",
        "price_type": "flat",
        "service_zone_id": "sz_default",
        "shipping_profile_id": "sp_default_shipping_profile",
        "provider_id": "manual_fulfillment",
        "data": {},
        "rules": [],
        "prices": [
            {
                "id": "sop_usd",
                "currency_code": "usd",
                "amount": "5.0000"
            }
        ],
        "created_at": "2026-08-17T17:29:39.266111Z",
        "updated_at": "2026-08-17T17:29:39.266111Z"
    });
    let express = json!({
        "id": "so_express_shipping",
        "name": "Express Delivery (Next Day)",
        "price_type": "flat",
        "service_zone_id": "sz_default",
        "shipping_profile_id": "sp_default_shipping_profile",
        "provider_id": "manual_fulfillment",
        "data": {},
        "rules": [],
        "prices": [
            {
                "id": "sop_exp_usd",
                "currency_code": "usd",
                "amount": "15.0000"
            }
        ],
        "created_at": "2026-08-17T17:29:39.266111Z",
        "updated_at": "2026-08-17T17:29:39.266111Z"
    });

    let options = vec![standard, express];

    Ok((
        StatusCode::OK,
        Json(json!({
            "shipping_options": options,
            "data": options,
            "count": 2,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn create_shipping_option(
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let opt_id = format!("so_{}", ulid::Ulid::new());
    let mut opt = input;
    if let Some(map) = opt.as_object_mut() {
        map.insert("id".to_string(), json!(opt_id));
        map.insert("created_at".to_string(), json!(Utc::now()));
        map.insert("updated_at".to_string(), json!(Utc::now()));
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "shipping_option": opt,
            "data": opt,
        })),
    ))
}

pub async fn list_payment_providers() -> Result<impl IntoResponse, Response> {
    let providers = vec![
        json!({
            "id": "pp_system_default",
            "is_enabled": true,
            "name": "Stheno Payment Gateway"
        }),
        json!({
            "id": "stripe",
            "is_enabled": true,
            "name": "Stripe Payments"
        }),
        json!({
            "id": "manual_payment",
            "is_enabled": true,
            "name": "Manual / Cash on Delivery"
        }),
    ];

    Ok((
        StatusCode::OK,
        Json(json!({
            "payment_providers": providers,
            "data": providers,
            "count": 3,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn list_payment_collections() -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "payment_collections": [],
            "data": [],
            "count": 0,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

// RBAC roles and policies now live in `rbac.rs`, backed by `stheno-rbac`. The handlers
// that used to sit here returned three hardcoded roles and an empty policy list.

// ── Locales & Translations ──────────────────────────────────────────────────
