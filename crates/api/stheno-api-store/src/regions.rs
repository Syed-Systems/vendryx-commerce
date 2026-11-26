//! Store region route handlers (Medusa v2 API parity).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;
use stheno_region::RegionService;

pub async fn list_regions(
    State(service): State<Arc<RegionService>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service.list_regions().await {
        Ok(regions) => {
            let count = regions.len();
            let tenant_id = stheno_db::tenant::current().unwrap_or_default();
            let entities: Vec<_> = regions
                .iter()
                .map(|region| ("region", region.id.to_string()))
                .collect();
            let entity_refs: Vec<_> = entities
                .iter()
                .map(|(kind, id)| (*kind, id.as_str()))
                .collect();
            let headers = crate::cache::public_cache_headers(&tenant_id, &entity_refs, false);
            Ok((
                headers,
                Json(json!({
                    "regions": regions,
                    "count": count,
                    "offset": 0,
                    "limit": 50
                })),
            ))
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
        Ok(region) => {
            let tenant_id = stheno_db::tenant::current().unwrap_or_default();
            let region_id = region.id.to_string();
            let headers = crate::cache::public_cache_headers(
                &tenant_id,
                &[("region", region_id.as_str())],
                false,
            );
            Ok((headers, Json(json!({ "region": region }))))
        }
        Err(stheno_types::Error::NotFound { .. }) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "type": "not_found",
                "message": format!("Region with id '{id}' not found")
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": e.to_string() })),
        )),
    }
}
