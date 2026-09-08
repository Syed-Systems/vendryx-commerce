//! Admin Promotions and Campaigns API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_promotion::{
    CreateCampaignInput, CreatePromotionInput, PromotionService, UpdateCampaignInput,
    UpdatePromotionInput,
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

// ── Promotion Rule Options & Batch Rules ──────────────────────────────────────

pub async fn list_rule_attribute_options(
    State(_service): State<Arc<PromotionService>>,
    Path(rule_type): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let options = vec![
        json!({ "id": "customer_group_id", "label": "Customer Group", "value": "customer_group_id" }),
        json!({ "id": "currency_code", "label": "Currency Code", "value": "currency_code" }),
        json!({ "id": "product_id", "label": "Product", "value": "product_id" }),
    ];
    Ok((
        StatusCode::OK,
        Json(json!({ "attributes": options, "data": options, "rule_type": rule_type })),
    ))
}

pub async fn list_rule_value_options(
    State(_service): State<Arc<PromotionService>>,
    Path((rule_type, rule_attribute_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "values": [],
            "data": [],
            "rule_type": rule_type,
            "rule_attribute_id": rule_attribute_id
        })),
    ))
}

pub async fn batch_promotion_rules(
    State(_service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "promotion": { "id": id }, "created": [], "updated": [], "deleted": [] })),
    ))
}

pub async fn batch_promotion_buy_rules(
    State(_service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "promotion": { "id": id }, "created": [], "updated": [], "deleted": [] })),
    ))
}

pub async fn batch_promotion_target_rules(
    State(_service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    Ok((
        StatusCode::OK,
        Json(json!({ "promotion": { "id": id }, "created": [], "updated": [], "deleted": [] })),
    ))
}

pub async fn list_promotion_rules_by_type(
    State(service): State<Arc<PromotionService>>,
    Path((id, rule_type)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let mut rules = service
        .get_rules_for_target(&id, &rule_type)
        .await
        .unwrap_or_default();

    let norm = rule_type.replace('-', "_");
    if rules.is_empty() && norm != "rules" {
        if let Ok(app) = service.get_application_method(&id).await {
            if let Ok(app_rules) = service.get_rules_for_target(&app.id, &rule_type).await {
                rules = app_rules;
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "rules": rules, "data": rules, "promotion_id": id, "rule_type": rule_type })),
    ))
}

pub async fn add_campaign_promotions(
    State(service): State<Arc<PromotionService>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Response> {
    let campaign = service.get_campaign(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;

    let mut to_add = Vec::new();
    if let Some(arr) = body.get("add").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                to_add.push(s.to_string());
            }
        }
    }
    if let Some(arr) = body.get("promotion_ids").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                to_add.push(s.to_string());
            }
        }
    }
    for promo_id in to_add {
        let input = UpdatePromotionInput {
            campaign_id: Some(id.clone()),
            ..Default::default()
        };
        let _ = service.update_promotion(&promo_id, input).await;
    }

    if let Some(arr) = body.get("remove").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(promo_id) = v.as_str() {
                let input = UpdatePromotionInput {
                    campaign_id: Some(String::new()),
                    ..Default::default()
                };
                let _ = service.update_promotion(promo_id, input).await;
            }
        }
    }

    let updated_campaign = service.get_campaign(&id).await.unwrap_or(campaign);

    Ok((StatusCode::OK, Json(json!({ "campaign": updated_campaign }))))
}
