//! Store Customer API handlers (Medusa v2 byte-compatible).
//!
//! `GET /store/customers/me` resolves the caller from the `Authorization: Bearer <jwt>`
//! header. A missing or invalid token is a 401, never a 404 — the storefront distinguishes
//! "logged out" from "customer gone".

use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_customer::{CustomerDTO, CustomerService};

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreCustomerContext {
    pub customer_service: Arc<CustomerService>,
    pub auth_service: Arc<AuthService>,
}

#[derive(Debug, Serialize)]
pub struct GetCustomerResponse {
    pub customer: CustomerDTO,
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            r#type: "unauthorized".to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

pub async fn get_current_customer(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
) -> Result<Json<GetCustomerResponse>, Response> {
    let token =
        bearer_token(&headers).ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;

    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;

    let actor_id = claims
        .actor_id
        .ok_or_else(|| unauthorized("Token is not bound to a customer"))?;

    match ctx.customer_service.get_customer(&actor_id).await {
        Ok(customer) => Ok(Json(GetCustomerResponse { customer })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                r#type: "not_found".to_string(),
                message: format!("Customer with id {} was not found", actor_id),
            }),
        )
            .into_response()),
    }
}
