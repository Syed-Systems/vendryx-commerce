//! Merchant account routes. This surface is intentionally outside `/platform/*`: a merchant can
//! read only their own draft and launch request, while operators retain the cross-tenant queue.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use stheno_api_core::{Tier, TierConfig};
use stheno_auth::AuthService;
use stheno_platform::{PlatformService, SubmitLaunchRequestInput};

use crate::bad_request;

#[derive(Clone)]
pub struct MerchantApiContext {
    pub platform: Arc<PlatformService>,
    pub auth_service: Arc<AuthService>,
    pub tier_config: Arc<TierConfig>,
}

async fn merchant_id(ctx: &MerchantApiContext, headers: &HeaderMap) -> Option<String> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?;
    let claims = ctx.auth_service.verify_jwt(token).await.ok()?;
    (claims.actor_type.as_deref() == Some("user")).then_some(claims.sub)
}

fn unavailable(capability: &str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(
            json!({ "type": "unavailable", "message": format!("{capability} is not configured") }),
        ),
    )
        .into_response()
}

/// Safe public plan projection. It reveals commercial terms, never the operator-editable plan
/// management endpoints.
pub async fn public_plans(State(ctx): State<MerchantApiContext>) -> Response {
    let trial = ctx.tier_config.definition(Tier::Trial).ok();
    let pro = ctx.tier_config.definition(Tier::Pro).ok();
    Json(json!({
        "plans": [
            { "id": "free", "trial_days": trial.and_then(|plan| plan.duration_days), "trial_starts_on": trial.and_then(|plan| plan.starts_on.clone()) },
            { "id": "pro", "price": pro.and_then(|plan| plan.price.clone()), "currency": pro.and_then(|plan| plan.currency.clone()), "interval": pro.and_then(|plan| plan.interval.clone()), "term_months": pro.map(|plan| plan.term_months.clone()).unwrap_or_default() }
        ]
    })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct SaveDraftBody {
    pub email: String,
    pub selected_tier: String,
    #[serde(default)]
    pub signup_draft: serde_json::Value,
}

pub async fn save_draft(
    State(ctx): State<MerchantApiContext>,
    headers: HeaderMap,
    Json(body): Json<SaveDraftBody>,
) -> Response {
    let Some(merchant_id) = merchant_id(&ctx, &headers).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "type": "unauthorized" })),
        )
            .into_response();
    };
    if body.email.trim().is_empty() {
        return bad_request("email is required");
    }
    match ctx
        .platform
        .upsert_merchant_account(
            merchant_id,
            body.email,
            None,
            body.signup_draft,
            body.selected_tier,
        )
        .await
    {
        Ok(account) => Json(json!({ "account": account })).into_response(),
        Err(error) => bad_request(error),
    }
}

#[derive(Debug, Deserialize)]
pub struct SubmitLaunchBody {
    pub store_name: String,
    pub subdomain: String,
    pub tier: String,
    pub logo_url: Option<String>,
    pub accent_color: String,
    pub icon_choice: String,
    /// Omitted by a client that predates the archetype choice: default false keeps the
    /// no-account storefront the product has always shipped (PRODUCT.md 2).
    #[serde(default)]
    pub buyer_accounts: bool,
    pub bespoke_brief: Option<String>,
    #[serde(default)]
    pub bespoke_assets: Vec<String>,
}

pub async fn submit_launch_request(
    State(ctx): State<MerchantApiContext>,
    headers: HeaderMap,
    Json(body): Json<SubmitLaunchBody>,
) -> Response {
    let Some(merchant_id) = merchant_id(&ctx, &headers).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "type": "unauthorized" })),
        )
            .into_response();
    };
    match ctx
        .platform
        .submit_launch_request(SubmitLaunchRequestInput {
            merchant_id,
            store_name: body.store_name,
            subdomain: body.subdomain,
            tier: body.tier,
            logo_url: body.logo_url,
            accent_color: body.accent_color,
            icon_choice: body.icon_choice,
            buyer_accounts: body.buyer_accounts,
            bespoke_brief: body.bespoke_brief,
            bespoke_assets: body.bespoke_assets,
        })
        .await
    {
        Ok(launch_request) => Json(json!({ "launch_request": launch_request })).into_response(),
        Err(error) => bad_request(error),
    }
}

/// Owner-only launch reads. The identifier is checked against the authenticated merchant before
/// returning anything, so a guessed request id does not reveal another merchant's store.
pub async fn get_launch_status(
    State(ctx): State<MerchantApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(merchant_id) = merchant_id(&ctx, &headers).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "type": "unauthorized" })),
        )
            .into_response();
    };
    match ctx.platform.get_launch_request(&id).await {
        Ok(request) if request.merchant_id == merchant_id => {
            Json(json!({ "launch_request": request })).into_response()
        }
        _ => (StatusCode::NOT_FOUND, Json(json!({ "type": "not_found" }))).into_response(),
    }
}

/// Payment initiation remains explicitly unavailable until a platform billing provider is wired to
/// merchant-account payments. It must not return a made-up checkout URL or success state.
pub async fn initiate_payment() -> Response {
    unavailable("Merchant payment initiation")
}
pub async fn payment_status() -> Response {
    unavailable("Merchant payment status")
}

pub fn merchant_routes(ctx: MerchantApiContext) -> Router {
    Router::new()
        .route("/merchant/plans", get(public_plans))
        .route("/merchant/account/draft", post(save_draft))
        .route("/merchant/launch-requests", post(submit_launch_request))
        .route("/merchant/launch-requests/{id}", get(get_launch_status))
        .route("/merchant/payments", post(initiate_payment))
        .route("/merchant/payments/{id}", get(payment_status))
        .with_state(ctx)
}
