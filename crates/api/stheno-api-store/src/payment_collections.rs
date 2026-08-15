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
