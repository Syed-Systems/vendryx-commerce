use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_payment::*;
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StorePaymentContext {
    pub payment_service: Arc<PaymentService>,
    /// Supplies the amount and currency to collect. A payment collection created from
    /// client-supplied figures would be a shopper naming what they pay.
    pub cart_service: Arc<stheno_cart::CartService>,
    /// Records which cart the collection belongs to.
    pub links: Arc<stheno_links::PgLinkStore>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreatePaymentCollectionRequest {
    pub cart_id: String,
}

/// `POST /store/payment-collections` — open a payment collection for a cart.
///
/// This route did not exist. `@medusajs/js-sdk`'s `initiatePaymentSession` calls it first
/// and uses the id it returns to create the session, so with it absent the call 404'd, no
/// session was ever created, and the checkout's review step — which renders only once
/// `cart.payment_collection` is present — stayed empty. The last step of the checkout was
/// unreachable and nothing said so.
pub async fn create_payment_collection(
    State(ctx): State<StorePaymentContext>,
    Json(body): Json<CreatePaymentCollectionRequest>,
) -> Result<impl IntoResponse, Response> {
    let cart = ctx
        .cart_service
        .get_cart(&body.cart_id)
        .await
        .map_err(map_error)?;

    let collection = ctx
        .payment_service
        .create_payment_collection(stheno_payment::CreatePaymentCollectionInput {
            currency_code: cart.currency_code.clone(),
            // The cart's own total, never the request's.
            amount: cart.totals.total,
            metadata: None,
        })
        .await
        .map_err(map_error)?;

    // Without this the collection exists but belongs to nothing, and the cart can never
    // report it.
    ctx.links
        .link("cart_payment_collection", &body.cart_id, &collection.id)
        .await
        .map_err(map_error)?;

    Ok((
        StatusCode::OK,
        Json(PaymentCollectionResponse {
            payment_collection: collection,
        }),
    ))
}

#[derive(Debug, Serialize)]
pub struct PaymentCollectionResponse {
    pub payment_collection: PaymentCollectionDTO,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub payment: PaymentDTO,
}

#[derive(Debug, Serialize)]
pub struct ListPaymentProvidersResponse {
    pub payment_providers: Vec<PaymentProviderItem>,
}

#[derive(Debug, Serialize)]
pub struct PaymentProviderItem {
    pub id: String,
    pub is_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct StoreCreatePaymentSessionRequest {
    pub provider_id: String,
    pub context: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StoreAuthorizePaymentSessionRequest {
    pub context: Option<serde_json::Value>,
}

pub async fn create_payment_session(
    State(ctx): State<StorePaymentContext>,
    Path(collection_id): Path<String>,
    Json(body): Json<StoreCreatePaymentSessionRequest>,
) -> Result<Json<PaymentCollectionResponse>, Response> {
    let col = ctx
        .payment_service
        .get_payment_collection(&collection_id)
        .await
        .map_err(map_error)?;

    ctx.payment_service
        .create_payment_session(
            CreatePaymentSessionInput {
                payment_collection_id: collection_id.clone(),
                provider_id: body.provider_id,
                amount: col.amount,
                currency_code: col.currency_code.clone(),
                context: body.context,
                data: body.data,
            },
            None,
        )
        .await
        .map_err(map_error)?;

    let updated_col = ctx
        .payment_service
        .get_payment_collection(&collection_id)
        .await
        .map_err(map_error)?;

    Ok(Json(PaymentCollectionResponse {
        payment_collection: updated_col,
    }))
}

pub async fn authorize_payment_session(
    State(ctx): State<StorePaymentContext>,
    Path((_collection_id, session_id)): Path<(String, String)>,
    Json(body): Json<StoreAuthorizePaymentSessionRequest>,
) -> Result<Json<PaymentResponse>, Response> {
    let payment = ctx
        .payment_service
        .authorize_payment_session(AuthorizePaymentSessionInput {
            session_id,
            context: body.context,
        })
        .await
        .map_err(map_error)?;

    Ok(Json(PaymentResponse { payment }))
}

pub async fn list_payment_providers(
    State(_ctx): State<StorePaymentContext>,
) -> Result<Json<ListPaymentProvidersResponse>, Response> {
    Ok(Json(ListPaymentProvidersResponse {
        payment_providers: vec![
            PaymentProviderItem {
                id: "pp_system_default".to_string(),
                is_enabled: true,
            },
            PaymentProviderItem {
                id: "pp_stripe_stripe".to_string(),
                is_enabled: true,
            },
        ],
    }))
}

fn map_error(e: Error) -> Response {
    let (status, r#type) = match e {
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
