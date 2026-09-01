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
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_customer::{CreateCustomerInput, CustomerDTO, CustomerService};

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

/// Body of `POST /store/customers`. Medusa sends the profile fields here; the email is
/// taken from the authenticated identity, not from this body — see below.
#[derive(Debug, Deserialize, Default)]
pub struct CreateCustomerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Creates the customer record for an already-registered login.
///
/// Registration is two calls: `/auth/customer/emailpass/register` mints the identity, and
/// this creates the profile and binds the two together. Only the first existed, so the
/// storefront's sign-up ended in "Error: Not Found", left an orphaned identity behind, and
/// `/store/customers/me` answered 401 forever after with a perfectly valid token.
///
/// The email comes from the identity's `entity_id`, never from the request body: the body
/// is attacker-controlled, and taking it on trust would let anyone create a customer under
/// somebody else's address and then read it back through their own token.
pub async fn create_customer(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
    body: Option<Json<CreateCustomerRequest>>,
) -> Result<Response, Response> {
    let Json(body) = body.unwrap_or_default();

    let token =
        bearer_token(&headers).ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;
    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;

    // Already bound: registering twice is a retry, not a second account. Returning the
    // existing customer keeps a double-submit from minting a duplicate profile.
    if let Some(actor_id) = claims.actor_id.clone() {
        let customer = ctx
            .customer_service
            .get_customer(&actor_id)
            .await
            .map_err(|_| unauthorized("Token is not bound to a customer"))?;
        return Ok((StatusCode::OK, Json(GetCustomerResponse { customer })).into_response());
    }

    let identity = ctx
        .auth_service
        .get_identity(&claims.sub)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;

    let customer = ctx
        .customer_service
        .create_customer(CreateCustomerInput {
            email: identity.entity_id.clone(),
            company_name: body.company_name,
            first_name: body.first_name,
            last_name: body.last_name,
            phone: body.phone,
            has_account: Some(true),
            metadata: body.metadata,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    r#type: "invalid_data".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response()
        })?;

    ctx.auth_service
        .bind_actor(&identity.id, &customer.id.to_string(), "customer")
        .await
        .map_err(|e| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    r#type: "conflict".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(GetCustomerResponse { customer })).into_response())
}
