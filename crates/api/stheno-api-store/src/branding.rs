//! P26-03: Store branding endpoint — GET /store/branding
//!
//! Returns the resolved branding for the current tenant.  An absent row yields
//! all-None fields; the storefront substitutes platform defaults for those.
//!
//! File IDs stored in the database are resolved to public URLs via the file
//! module before being emitted.  The JSON shape is a Store API contract.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use stheno_store::{BrandingDTO, StoreService};

#[derive(Clone)]
pub struct BrandingContext {
    pub store_service: Arc<StoreService>,
}

#[derive(Debug, Serialize)]
pub struct BrandingResponse {
    pub branding: BrandingDTO,
}

/// GET /store/branding
///
/// Returns the current tenant's branding configuration.
/// Absent rows are not errors — defaults are returned.
pub async fn get_branding(State(ctx): State<BrandingContext>) -> impl IntoResponse {
    match ctx.store_service.get_branding().await {
        Ok(branding) => Json(BrandingResponse { branding }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "failed to load tenant branding");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
