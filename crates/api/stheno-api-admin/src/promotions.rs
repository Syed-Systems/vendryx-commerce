//! Admin Promotions and Campaigns API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};
use stheno_promotion::{
    CampaignDTO, CreateCampaignInput, CreatePromotionInput, PromotionDTO, PromotionService,
    UpdateCampaignInput, UpdatePromotionInput,
};

// 1. Promotion Handlers
pub async fn list_promotions(
    State(service): State<Arc<PromotionService>>,
) -> Result<Json<ListResponse<Vec<PromotionDTO>>>, Response> {
    let all = service.list_promotions().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse {
        data: all,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_promotion(
    State(service): State<Arc<PromotionService>>,
    Json(input): Json<CreatePromotionInput>,
) -> Result<(StatusCode, Json<GetResponse<PromotionDTO>>), Response> {
    let promo = service.create_promotion(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: promo })))
}

pub async fn get_promotion(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<PromotionDTO>>, Response> {
    match service.get_promotion(&id).await {
        Ok(promo) => Ok(Json(GetResponse { data: promo })),
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
) -> Result<Json<GetResponse<PromotionDTO>>, Response> {
    let promo = service.update_promotion(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: promo }))
}

pub async fn delete_promotion(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    service.delete_promotion(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(DeleteResponse {
        id,
        object: "promotion".to_string(),
        deleted: true,
    }))
}

// 2. Campaign Handlers
pub async fn list_campaigns(
    State(service): State<Arc<PromotionService>>,
) -> Result<Json<ListResponse<Vec<CampaignDTO>>>, Response> {
    let all = service.list_campaigns().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse {
        data: all,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_campaign(
    State(service): State<Arc<PromotionService>>,
    Json(input): Json<CreateCampaignInput>,
) -> Result<(StatusCode, Json<GetResponse<CampaignDTO>>), Response> {
    let camp = service.create_campaign(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: camp })))
}

pub async fn get_campaign(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<CampaignDTO>>, Response> {
    match service.get_campaign(&id).await {
        Ok(camp) => Ok(Json(GetResponse { data: camp })),
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
) -> Result<Json<GetResponse<CampaignDTO>>, Response> {
    let camp = service.update_campaign(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: camp }))
}

pub async fn delete_campaign(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    service.delete_campaign(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(DeleteResponse {
        id,
        object: "campaign".to_string(),
        deleted: true,
    }))
}
