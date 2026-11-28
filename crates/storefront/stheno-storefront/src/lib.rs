//! Tenant-scoped server-rendered storefront routes.

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use minijinja::{context, Environment};
use std::sync::Arc;
use stheno_store::StoreService;

/// Dependencies used to serve storefront routes for the current tenant.
pub struct StorefrontContext {
    pub store_service: Arc<StoreService>,
}

/// Routes served inside the engine's tenant layer.
pub fn storefront_router(ctx: StorefrontContext) -> Router {
    let store_service = ctx.store_service;

    Router::new().route(
        "/{country}",
        get(move || {
            let store_service = Arc::clone(&store_service);
            async move {
                let branding = match store_service.get_branding().await {
                    Ok(branding) => branding,
                    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                };

                let environment = Environment::new();
                match environment.render_named_str(
                    "index.html",
                    "<!doctype html><title>{{ store.name }}</title><h1>{{ store.name }}</h1>",
                    context! { store => context! { name => branding.store_name.unwrap_or_default() } },
                ) {
                    Ok(html) => Html(html).into_response(),
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            }
        }),
    )
}
