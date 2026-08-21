//! P33: Growth analytics routes (PHASE4_TASKS P33).

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::{actor, deny, PlatformApiContext};

pub async fn get_gmv_metrics(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_gmv_metrics().await {
        Ok(metrics) => Json(json!({ "gmv_metrics": metrics })).into_response(),
        Err(_) => Json(json!({ "error": "failed_to_fetch_gmv_metrics" })).into_response(),
    }
}

pub async fn get_funnel_metrics(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_funnel_metrics().await {
        Ok(metrics) => Json(json!({ "funnel": metrics })).into_response(),
        Err(_) => Json(json!({ "error": "failed_to_fetch_funnel_metrics" })).into_response(),
    }
}

pub async fn get_plan_distribution(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_plan_distribution().await {
        Ok(distribution) => Json(json!({ "distribution": distribution })).into_response(),
        Err(_) => Json(json!({ "error": "failed_to_fetch_plan_distribution" })).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct TopMerchantsQuery {
    pub limit: Option<usize>,
}

pub async fn get_top_merchants(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Query(query): Query<TopMerchantsQuery>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let limit = query.limit.unwrap_or(10);
    match ctx.platform.get_top_merchants_by_gmv(limit).await {
        Ok(merchants) => Json(json!({ "top_merchants": merchants })).into_response(),
        Err(_) => Json(json!({ "error": "failed_to_fetch_top_merchants" })).into_response(),
    }
}

pub async fn get_churn_cohorts(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_churn_cohorts().await {
        Ok(cohorts) => Json(json!({ "cohorts": cohorts })).into_response(),
        Err(_) => Json(json!({ "error": "failed_to_fetch_churn_cohorts" })).into_response(),
    }
}

pub async fn get_quota_headroom(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.get_quota_headroom().await {
        Ok(headroom) => Json(json!({ "headroom": headroom })).into_response(),
        Err(_) => Json(json!({ "error": "failed_to_fetch_quota_headroom" })).into_response(),
    }
}
