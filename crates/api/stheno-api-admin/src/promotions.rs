//! Admin Promotions and Campaigns API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_promotion::{
    CreateCampaignInput, CreatePromotionInput, PromotionService,
    UpdateCampaignInput, UpdatePromotionInput,
};

// 1. Promotion Handlers
pub async fn list_promotions(
    State(service): State<Arc<PromotionService>>,
) -> Result<impl IntoResponse, Response> {
    let all = service.list_promotions().await.map_err(|e| {
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
            "promotions": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_promotion(
    State(service): State<Arc<PromotionService>>,
    Json(input): Json<CreatePromotionInput>,
) -> Result<impl IntoResponse, Response> {
    let promo = service.create_promotion(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "promotion": promo,
            "data": promo,
        })),
    ))
}

pub async fn get_promotion(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_promotion(&id).await {
        Ok(promo) => Ok((
            StatusCode::OK,
            Json(json!({
                "promotion": promo,
                "data": promo,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Promotion with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_promotion(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePromotionInput>,
) -> Result<impl IntoResponse, Response> {
    let promo = service.update_promotion(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "promotion": promo,
            "data": promo,
        })),
    ))
}

pub async fn delete_promotion(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_promotion(&id).await.map_err(|e| {
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
            "object": "promotion",
            "deleted": true
        })),
    ))
}

// 2. Campaign Handlers
pub async fn list_campaigns(
    State(service): State<Arc<PromotionService>>,
) -> Result<impl IntoResponse, Response> {
    let all = service.list_campaigns().await.map_err(|e| {
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
            "campaigns": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_campaign(
    State(service): State<Arc<PromotionService>>,
    Json(input): Json<CreateCampaignInput>,
) -> Result<impl IntoResponse, Response> {
    let camp = service.create_campaign(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "campaign": camp,
            "data": camp,
        })),
    ))
}

pub async fn get_campaign(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_campaign(&id).await {
        Ok(camp) => Ok((
            StatusCode::OK,
            Json(json!({
                "campaign": camp,
                "data": camp,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Campaign with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_campaign(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCampaignInput>,
) -> Result<impl IntoResponse, Response> {
    let camp = service.update_campaign(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "campaign": camp,
            "data": camp,
        })),
    ))
}

pub async fn delete_campaign(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_campaign(&id).await.map_err(|e| {
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
            "object": "campaign",
            "deleted": true
        })),
    ))
}
