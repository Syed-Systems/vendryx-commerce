//! Store region route handlers (Medusa v2 API parity).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_region::RegionService;

pub async fn list_regions(
    State(service): State<Arc<RegionService>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service.list_regions().await {
        Ok(regions) => {
            let count = regions.len();
            Ok(Json(json!({
                "regions": regions,
                "count": count,
                "offset": 0,
                "limit": 50
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": e.to_string() })),
        )),
    }
}

pub async fn get_region(
    State(service): State<Arc<RegionService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service.get_region(&id).await {
        Ok(region) => Ok(Json(json!({ "region": region }))),
        Err(stheno_types::Error::NotFound { .. }) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "type": "not_found",
                "message": format!("Region with id '{}' not found", id)
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": e.to_string() })),
        )),
    }
}
