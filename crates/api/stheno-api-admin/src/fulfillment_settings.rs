//! Admin fulfillment-settings routes — shipping profiles, fulfillment sets, service
//! zones and shipping-option types.
//!
//! These replace handlers that invented their responses: `/admin/shipping-profiles`
//! returned a single fabricated row with a hardcoded id (`sp_default_shipping_profile`)
//! and a hardcoded timestamp, `/admin/fulfillment-sets` returned an invented set with an
//! invented service zone, and `/admin/shipping-option-types` returned two made-up types.
//! Every screen under Settings → Locations was therefore reading fiction that could never
//! be edited, because nothing was ever stored.
//!
//! `FulfillmentService` could already create all of these; it simply had no read side.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use stheno_api_core::ErrorResponse;
use stheno_fulfillment::{
    CreateFulfillmentSetInput, CreateServiceZoneInput, CreateShippingOptionTypeInput,
    CreateShippingProfileInput, FulfillmentService,
};

#[derive(Clone)]
pub struct FulfillmentSettingsContext {
    pub fulfillment_service: Arc<FulfillmentService>,
}

fn failed(status: StatusCode, code: &'static str, e: stheno_types::Error) -> Response {
    (status, Json(ErrorResponse::new(code, &e.to_string()))).into_response()
}

fn internal(e: stheno_types::Error) -> Response {
    failed(StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state", e)
}

fn not_found(e: stheno_types::Error) -> Response {
    failed(StatusCode::NOT_FOUND, "not_found", e)
}

fn bad_request(e: stheno_types::Error) -> Response {
    failed(StatusCode::BAD_REQUEST, "invalid_data", e)
}

// ── Shipping profiles ────────────────────────────────────────────────────────

pub async fn list_shipping_profiles(
    State(ctx): State<FulfillmentSettingsContext>,
) -> Result<impl IntoResponse, Response> {
    let profiles = ctx
        .fulfillment_service
        .list_shipping_profiles()
        .await
        .map_err(internal)?;
    let count = profiles.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "shipping_profiles": profiles, "data": profiles,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_shipping_profile(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let profile = ctx
        .fulfillment_service
        .get_shipping_profile(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "shipping_profile": profile, "data": profile })),
    ))
}

pub async fn create_shipping_profile(
    State(ctx): State<FulfillmentSettingsContext>,
    Json(input): Json<CreateShippingProfileInput>,
) -> Result<impl IntoResponse, Response> {
    let profile = ctx
        .fulfillment_service
        .create_shipping_profile(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "shipping_profile": profile, "data": profile })),
    ))
}

pub async fn delete_shipping_profile(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.fulfillment_service
        .delete_shipping_profile(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "shipping_profile", "deleted": true })),
    ))
}

// ── Fulfillment sets and service zones ───────────────────────────────────────

pub async fn list_fulfillment_sets(
    State(ctx): State<FulfillmentSettingsContext>,
) -> Result<impl IntoResponse, Response> {
    let sets = ctx
        .fulfillment_service
        .list_fulfillment_sets()
        .await
        .map_err(internal)?;
    let count = sets.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "fulfillment_sets": sets, "data": sets,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn get_fulfillment_set(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let set = ctx
        .fulfillment_service
        .get_fulfillment_set(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "fulfillment_set": set, "data": set })),
    ))
}

pub async fn create_fulfillment_set(
    State(ctx): State<FulfillmentSettingsContext>,
    Json(input): Json<CreateFulfillmentSetInput>,
) -> Result<impl IntoResponse, Response> {
    let set = ctx
        .fulfillment_service
        .create_fulfillment_set(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "fulfillment_set": set, "data": set })),
    ))
}

pub async fn list_service_zones(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let zones = ctx
        .fulfillment_service
        .list_service_zones(&id)
        .await
        .map_err(not_found)?;
    let count = zones.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "service_zones": zones, "data": zones, "count": count,
        })),
    ))
}

pub async fn create_service_zone(
    State(ctx): State<FulfillmentSettingsContext>,
    Json(input): Json<CreateServiceZoneInput>,
) -> Result<impl IntoResponse, Response> {
    let zone = ctx
        .fulfillment_service
        .create_service_zone(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "service_zone": zone, "data": zone })),
    ))
}

// ── Shipping option types ────────────────────────────────────────────────────

pub async fn list_shipping_option_types(
    State(ctx): State<FulfillmentSettingsContext>,
) -> Result<impl IntoResponse, Response> {
    let types = ctx
        .fulfillment_service
        .list_shipping_option_types()
        .await
        .map_err(internal)?;
    let count = types.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "shipping_option_types": types, "data": types,
            "count": count, "offset": 0, "limit": count,
        })),
    ))
}

pub async fn create_shipping_option_type(
    State(ctx): State<FulfillmentSettingsContext>,
    Json(input): Json<CreateShippingOptionTypeInput>,
) -> Result<impl IntoResponse, Response> {
    let ty = ctx
        .fulfillment_service
        .create_shipping_option_type(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "shipping_option_type": ty, "data": ty })),
    ))
}

/// `GET /admin/fulfillment-providers` — which providers this build registered.
///
/// Genuinely static: providers are compile-time crates wired into the registry
/// (ADR-001), so this is a report of what was built in, not a database read. It is
/// derived from the service's live registry rather than a hardcoded list, so it cannot
/// drift from what is actually registered.
pub async fn list_fulfillment_providers(
    State(ctx): State<FulfillmentSettingsContext>,
) -> Result<impl IntoResponse, Response> {
    let providers: Vec<_> = ctx
        .fulfillment_service
        .registered_provider_ids()
        .into_iter()
        .map(|id| json!({ "id": id, "is_enabled": true }))
        .collect();
    let count = providers.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "fulfillment_providers": providers, "data": providers, "count": count,
        })),
    ))
}

/// POST /admin/fulfillment-sets/{id}/service-zones
///
/// The same handler as `create_service_zone`, reading the set from the path segment. The
/// dashboard only ever calls the nested form; the flat `/admin/service-zones` route stays
/// mounted because the storefront gate tests reference it.
pub async fn create_service_zone_for_set(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(fulfillment_set_id): Path<String>,
    Json(payload): Json<CreateServiceZoneForSetRequest>,
) -> Result<impl IntoResponse, Response> {
    let zone = ctx
        .fulfillment_service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id,
            name: payload.name,
            metadata: payload.metadata,
        })
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "service_zone": zone, "data": zone })),
    ))
}

/// The nested body: everything `CreateServiceZoneInput` needs except the set, which the
/// path already carries.
#[derive(Debug, serde::Deserialize)]
pub struct CreateServiceZoneForSetRequest {
    pub name: String,
    pub metadata: Option<serde_json::Value>,
}

/// POST /admin/shipping-profiles/{id}
pub async fn update_shipping_profile(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(id): Path<String>,
    Json(input): Json<stheno_fulfillment::UpdateShippingProfileInput>,
) -> Result<impl IntoResponse, Response> {
    let profile = ctx
        .fulfillment_service
        .update_shipping_profile(&id, input)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "shipping_profile": profile, "data": profile })),
    ))
}

/// DELETE /admin/fulfillment-sets/{id}
pub async fn delete_fulfillment_set(
    State(ctx): State<FulfillmentSettingsContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.fulfillment_service
        .delete_fulfillment_set(&id)
        .await
        .map_err(not_found)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "fulfillment_set", "deleted": true })),
    ))
}

/// GET /admin/shipping-options
///
/// Moved here from `config.rs`, which returned one fabricated option and had no
/// `FulfillmentService` to ask. `list_shipping_options` has existed since P5-08.
pub async fn list_shipping_options(
    State(ctx): State<FulfillmentSettingsContext>,
    Query(filter): Query<stheno_fulfillment::ShippingOptionFilter>,
) -> Result<impl IntoResponse, Response> {
    let (options, count) = ctx
        .fulfillment_service
        .list_shipping_options(filter)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "shipping_options": options,
            "data": options,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

/// POST /admin/shipping-options
///
/// The previous handler echoed the request back with an invented `so_<ulid>` and stored
/// nothing, so a shipping option configured by a merchant never reached checkout.
pub async fn create_shipping_option(
    State(ctx): State<FulfillmentSettingsContext>,
    Json(input): Json<stheno_fulfillment::CreateShippingOptionInput>,
) -> Result<impl IntoResponse, Response> {
    let option = ctx
        .fulfillment_service
        .create_shipping_option(input)
        .await
        .map_err(bad_request)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "shipping_option": option, "data": option })),
    ))
}
