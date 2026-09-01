//! Admin Configuration API Handlers — regions, currencies, tax, sales channels, store settings.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_currency::CurrencyService;
use stheno_links::PgLinkStore;
use stheno_payment::{
    CredentialField, CredentialHealth, CredentialMap, PaymentProviderCredentialDTO,
    PaymentProviderFactory, PaymentService,
};
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
    pub payment_service: Arc<PaymentService>,
    pub links: Arc<PgLinkStore>,
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



const REGION_PAYMENT_PROVIDER_LINK: &str = "region_payment_provider";

#[derive(Deserialize)]
pub struct UpsertPaymentProviderCredentialRequest {
    pub credentials: CredentialMap,
}

pub async fn list_payment_providers(
    State(ctx): State<ConfigContext>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let mut providers = Vec::new();

    for factory in ctx.payment_service.list_factories() {
        let credential = ctx
            .payment_service
            .get_provider_credential(&tenant_id, factory.provider_id())
            .await
            .map_err(map_error)?;
        let enabled_regions = ctx
            .links
            .get_by_right(REGION_PAYMENT_PROVIDER_LINK, factory.provider_id())
            .await
            .map_err(map_error)?;
        providers.push(provider_json(factory.as_ref(), credential, enabled_regions));
    }

    let count = providers.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "payment_providers": providers,
            "data": providers,
            "count": count,
            "offset": 0,
            "limit": 50,
        })),
    ))
}

pub async fn upsert_payment_provider_credentials(
    State(ctx): State<ConfigContext>,
    Path(provider_id): Path<String>,
    Json(input): Json<UpsertPaymentProviderCredentialRequest>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let credential = ctx
        .payment_service
        .upsert_provider_credential(&tenant_id, &provider_id, &input.credentials)
        .await
        .map_err(map_error)?;
    let factory = ctx
        .payment_service
        .get_factory(&provider_id)
        .map_err(map_error)?;
    let enabled_regions = ctx
        .links
        .get_by_right(REGION_PAYMENT_PROVIDER_LINK, &provider_id)
        .await
        .map_err(map_error)?;

    let provider = provider_json(factory.as_ref(), Some(credential), enabled_regions);
    Ok((
        StatusCode::OK,
        Json(json!({ "payment_provider": provider, "data": provider })),
    ))
}

pub async fn enable_region_payment_provider(
    State(ctx): State<ConfigContext>,
    Path((region_id, provider_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let factory = ctx
        .payment_service
        .get_factory(&provider_id)
        .map_err(map_error)?;
    ctx.links
        .link(REGION_PAYMENT_PROVIDER_LINK, &region_id, &provider_id)
        .await
        .map_err(map_error)?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let credential = ctx
        .payment_service
        .get_provider_credential(&tenant_id, &provider_id)
        .await
        .map_err(map_error)?;
    let enabled_regions = ctx
        .links
        .get_by_right(REGION_PAYMENT_PROVIDER_LINK, &provider_id)
        .await
        .map_err(map_error)?;

    let provider = provider_json(factory.as_ref(), credential, enabled_regions);
    Ok((
        StatusCode::OK,
        Json(json!({ "payment_provider": provider, "data": provider })),
    ))
}

pub async fn disable_region_payment_provider(
    State(ctx): State<ConfigContext>,
    Path((region_id, provider_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let factory = ctx
        .payment_service
        .get_factory(&provider_id)
        .map_err(map_error)?;
    ctx.links
        .unlink(REGION_PAYMENT_PROVIDER_LINK, &region_id, &provider_id)
        .await
        .map_err(map_error)?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let credential = ctx
        .payment_service
        .get_provider_credential(&tenant_id, &provider_id)
        .await
        .map_err(map_error)?;
    let enabled_regions = ctx
        .links
        .get_by_right(REGION_PAYMENT_PROVIDER_LINK, &provider_id)
        .await
        .map_err(map_error)?;

    let provider = provider_json(factory.as_ref(), credential, enabled_regions);
    Ok((
        StatusCode::OK,
        Json(json!({ "payment_provider": provider, "data": provider })),
    ))
}

fn provider_json(
    factory: &dyn PaymentProviderFactory,
    credential: Option<PaymentProviderCredentialDTO>,
    enabled_regions: Vec<String>,
) -> Value {
    let configured = credential.is_some();
    let credential_health = credential
        .as_ref()
        .map(|credential| credential.health)
        .unwrap_or(CredentialHealth::Unconfigured);

    json!({
        "id": factory.provider_id(),
        "name": provider_name(factory.provider_id()),
        "is_enabled": !enabled_regions.is_empty(),
        "enabled_regions": enabled_regions,
        "configured": configured,
        "credential_health": credential_health,
        "credential_schema": credential_schema_json(factory.credential_schema()),
        "credential": credential,
    })
}

fn credential_schema_json(schema: &[CredentialField]) -> Vec<Value> {
    schema.iter().map(|field| json!(field)).collect()
}

fn provider_name(provider_id: &str) -> String {
    provider_id
        .trim_start_matches("pp_")
        .replace('_', " ")
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn map_error(e: stheno_types::Error) -> Response {
    // Every branch used to return the same fixed string, so an operator configuring a
    // provider saw "Payment provider configuration failed" whatever the actual cause —
    // a missing STHENO_CREDENTIAL_KEY, a required field, an unknown field all looked
    // identical. A 4xx is the caller's to fix, so it carries the reason; a 5xx stays
    // generic because its detail is ours, not the client's.
    let (status, r#type, message) = match &e {
        stheno_types::Error::NotFound { .. } => {
            (StatusCode::NOT_FOUND, "not_found", e.to_string())
        }
        stheno_types::Error::Invalid { message, .. } => (
            StatusCode::BAD_REQUEST,
            "invalid_data",
            message.clone(),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected_state",
            "Payment provider configuration failed".to_string(),
        ),
    };
    (status, Json(ErrorResponse::new(r#type, &message))).into_response()
}


// RBAC roles and policies now live in `rbac.rs`, backed by `stheno-rbac`. The handlers
// that used to sit here returned three hardcoded roles and an empty policy list.

// ── Locales & Translations ──────────────────────────────────────────────────

/// POST /admin/sales-channels/{id}
///
/// Mounted `get`/`delete` only, so the sales-channel edit form could never save.
pub async fn update_sales_channel(
    State(ctx): State<ConfigContext>,
    Path(id): Path<String>,
    Json(input): Json<stheno_sales_channel::UpdateSalesChannelInput>,
) -> Result<impl IntoResponse, Response> {
    let channel = ctx
        .sales_channel_service
        .update_channel(&id, input)
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
        Json(json!({ "sales_channel": channel, "data": channel })),
    ))
}
