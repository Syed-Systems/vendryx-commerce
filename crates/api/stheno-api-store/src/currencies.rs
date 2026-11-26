//! Store currency route handlers (Medusa v2 API parity).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;
use stheno_currency::CurrencyService;

pub async fn list_currencies(
    State(service): State<Arc<CurrencyService>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service.list_currencies().await {
        Ok(currencies) => {
            let count = currencies.len();
            Ok(Json(json!({
                "currencies": currencies,
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

pub async fn get_currency(
    State(service): State<Arc<CurrencyService>>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service.get_currency(&code).await {
        Ok(currency) => Ok(Json(json!({ "currency": currency }))),
        Err(stheno_types::Error::NotFound { .. }) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "type": "not_found",
                "message": format!("Currency with code '{code}' not found")
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": e.to_string() })),
        )),
    }
}
