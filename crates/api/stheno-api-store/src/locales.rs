//! Store Locale API handlers.
//!
//! Additive to Medusa's Store contract (ADR-002 permits additive endpoints).

use axum::Json;
use axum::extract::State;
use axum::response::Response;
use serde::Serialize;
use std::sync::Arc;
use stheno_translation::{LocaleDTO, TranslationService};

#[derive(Debug, Serialize)]
pub struct ListLocalesResponse {
    pub locales: Vec<LocaleDTO>,
}

pub async fn list_locales(
    State(service): State<Arc<TranslationService>>,
) -> Result<Json<ListLocalesResponse>, Response> {
    let locales = service
        .list_locales()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|l| l.is_enabled)
        .collect();

    Ok(Json(ListLocalesResponse { locales }))
}
