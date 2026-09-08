//! Inbound provider webhooks (P21-01).
//!
//! `PaymentService::receive_webhook` has been complete since P4-19 — signature
//! verification, the replay guard on `(provider_id, provider_event_id)`, and the
//! convergence reducer. Nothing on the wire could reach it. This exposes it.
//!
//! Two properties this module exists to hold, both easy to break invisibly:
//!
//! 1. **The body reaches the provider byte-for-byte.** Every HMAC scheme signs the
//!    exact bytes sent. Any middleware that parses JSON and re-serialises it before
//!    the handler breaks every signature check — and breaks it in production against
//!    a real gateway, not in tests where fixtures are byte-exact. `Bytes` is
//!    therefore the extractor, and it is declared last because it consumes the body.
//! 2. **A failed signature never touches the database.** Verification happens inside
//!    `receive_webhook` before any write, so a forged callback costs one hash.

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;
use stheno_fulfillment::FulfillmentService;
use stheno_payment::PaymentService;
use stheno_types::Error;

#[derive(Clone)]
pub struct HooksContext {
    pub payment_service: Arc<PaymentService>,
    pub fulfillment_service: Option<Arc<FulfillmentService>>,
}

pub fn hooks_routes(ctx: HooksContext) -> Router {
    Router::new()
        .route(
            "/hooks/payment/{provider_id}",
            post(receive_payment_webhook),
        )
        .route(
            "/hooks/fulfillment/{provider_id}",
            post(receive_fulfillment_webhook),
        )
        .with_state(ctx)
}

#[derive(Serialize)]
struct WebhookAccepted {
    id: String,
    provider_id: String,
    state: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    r#type: String,
    message: String,
}

async fn receive_payment_webhook(
    State(ctx): State<HooksContext>,
    Path(provider_id): Path<String>,
    headers: HeaderMap,
    // Last: consumes the request body. Raw, unparsed, exactly as sent.
    body: Bytes,
) -> Response {
    let tenant_id = match stheno_db::tenant::current() {
        Ok(t) => t,
        Err(e) => return map_error(e),
    };
    let provider = match ctx
        .payment_service
        .get_provider(&tenant_id, &provider_id)
        .await
    {
        Ok(p) => p,
        Err(e) => return map_error(e),
    };

    // The provider names its own header; the route does not guess.
    let signature = provider
        .signature_header()
        .and_then(|name| headers.get(name))
        .and_then(|v| v.to_str().ok());

    match ctx
        .payment_service
        .receive_webhook(&provider_id, &body, signature)
        .await
    {
        Ok(event) => (
            StatusCode::OK,
            Json(WebhookAccepted {
                id: event.id,
                provider_id: event.provider_id,
                state: event.state,
            }),
        )
            .into_response(),
        Err(e) => map_error(e),
    }
}

async fn receive_fulfillment_webhook(
    State(ctx): State<HooksContext>,
    Path(provider_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(fulfillment_service) = &ctx.fulfillment_service else {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: "Fulfillment service not mounted".to_string(),
            }),
        )
            .into_response();
    };

    let signature = headers
        .get("x-signature")
        .or_else(|| headers.get("authorization"))
        .and_then(|v| v.to_str().ok());

    match fulfillment_service
        .receive_webhook(&provider_id, &body, signature)
        .await
    {
        Ok(event) => (
            StatusCode::OK,
            Json(WebhookAccepted {
                id: event.id,
                provider_id: event.provider_id,
                state: event.state,
            }),
        )
            .into_response(),
        Err(e) => map_error(e),
    }
}

/// A signature failure is 401, not 400. The caller is unauthenticated, and saying
/// "bad request" invites a gateway to retry a payload that will never be accepted.
fn map_error(e: Error) -> Response {
    let (status, r#type) = match &e {
        Error::Invalid { code, .. } if is_signature_failure(code) => {
            (StatusCode::UNAUTHORIZED, "unauthorized")
        }
        Error::NotFound { .. } => (StatusCode::NOT_FOUND, "not_found"),
        Error::Conflict { .. } => (StatusCode::CONFLICT, "conflict"),
        Error::Invalid { .. } => (StatusCode::BAD_REQUEST, "invalid_data"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state"),
    };

    (
        status,
        Json(ErrorResponse {
            r#type: r#type.to_string(),
            message: e.to_string(),
        }),
    )
        .into_response()
}

fn is_signature_failure(code: &str) -> bool {
    code.contains("signature")
}
