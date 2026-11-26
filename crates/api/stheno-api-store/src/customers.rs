//! Store Customer API handlers (Medusa v2 byte-compatible).
//!
//! `GET /store/customers/me` resolves the caller from the `Authorization: Bearer <jwt>`
//! header. A missing or invalid token is a 401, never a 404 — the storefront distinguishes
//! "logged out" from "customer gone".

use axum::extract::Path;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_auth::AuthService;
use stheno_customer::{
    CreateCustomerAddressInput, CreateCustomerInput, CustomerAddressDTO, CustomerDTO,
    CustomerService, UpdateCustomerAddressInput, UpdateCustomerInput,
};
use stheno_notification::NotificationService;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreCustomerContext {
    pub customer_service: Arc<CustomerService>,
    pub auth_service: Arc<AuthService>,
    pub notification_service: Arc<NotificationService>,
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
                message: format!("Customer with id {actor_id} was not found"),
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

// ── Authenticated customer profile and address book ──────────────────────────
//
// Every route below resolves the caller from the bearer token and never takes a customer
// id from the client. `/store/customers/me/addresses/{id}` carries an address id, and an
// address id is guessable, so each one re-checks that the address belongs to the caller
// before touching it — otherwise the address book is readable and editable across
// accounts by anyone who can enumerate ids.

/// Resolves the authenticated customer id, or the 401 the storefront expects.
async fn current_customer_id(
    ctx: &StoreCustomerContext,
    headers: &HeaderMap,
) -> Result<String, Response> {
    let token =
        bearer_token(headers).ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;
    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;
    claims
        .actor_id
        .ok_or_else(|| unauthorized("Token is not bound to a customer"))
}

/// DELETE /store/customers/me/whatsapp-consent
///
/// Revocation is keyed to the authenticated customer, not a caller-supplied phone number, so
/// consent remains revocable after the customer changes their profile phone.
pub async fn revoke_my_whatsapp_consent(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<serde_json::Value>), Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    let tenant_id = stheno_db::tenant::current().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                r#type: "unexpected_state".to_string(),
                message: "Tenant scope is unavailable.".to_string(),
            }),
        )
            .into_response()
    })?;

    ctx.notification_service
        .revoke_whatsapp_consent_for_customer(&tenant_id, &customer_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    r#type: "unexpected_state".to_string(),
                    message: "WhatsApp notification consent could not be updated.".to_string(),
                }),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(serde_json::json!({ "revoked": true }))))
}

fn address_not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            r#type: "not_found".to_string(),
            message: format!("Address with id {id} was not found"),
        }),
    )
        .into_response()
}

/// Loads an address and refuses it unless it belongs to the caller.
///
/// Answers 404 rather than 403 for someone else's address: a 403 confirms the id exists,
/// which is an enumeration oracle over other people's address books.
async fn owned_address(
    ctx: &StoreCustomerContext,
    customer_id: &str,
    address_id: &str,
) -> Result<CustomerAddressDTO, Response> {
    let address = ctx
        .customer_service
        .get_customer_address(address_id)
        .await
        .map_err(|_| address_not_found(address_id))?;

    if address.customer_id != customer_id {
        return Err(address_not_found(address_id));
    }
    Ok(address)
}

#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub address: CustomerAddressDTO,
}

#[derive(Debug, Serialize)]
pub struct AddressListResponse {
    pub addresses: Vec<CustomerAddressDTO>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

/// Body of `POST /store/customers/me`. Profile fields only.
///
/// Email is absent on purpose: it identifies the login, and changing it here would let a
/// customer move their profile onto an address they have not proved they control.
#[derive(Debug, Deserialize, Default)]
pub struct UpdateCustomerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// POST /store/customers/me
pub async fn update_current_customer(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
    Json(body): Json<UpdateCustomerRequest>,
) -> Result<Json<GetCustomerResponse>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;

    let customer = ctx
        .customer_service
        .update_customer(
            &customer_id,
            UpdateCustomerInput {
                email: None,
                first_name: body.first_name,
                last_name: body.last_name,
                phone: body.phone,
                company_name: body.company_name,
                metadata: body.metadata,
                // Not settable from the storefront profile form. Defaults are chosen on
                // the address itself, and `has_account` is decided by registration.
                default_billing_address_id: None,
                default_shipping_address_id: None,
                has_account: None,
            },
        )
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

    Ok(Json(GetCustomerResponse { customer }))
}

/// GET /store/customers/me/addresses
pub async fn list_my_addresses(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
) -> Result<Json<AddressListResponse>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    let addresses = ctx
        .customer_service
        .list_customer_addresses(&customer_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    r#type: "unexpected_state".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response()
        })?;
    let count = addresses.len();
    Ok(Json(AddressListResponse {
        addresses,
        count,
        offset: 0,
        limit: count,
    }))
}

/// Body of the address create route. `customer_id` is taken from the token, never here.
#[derive(Debug, Deserialize)]
pub struct CreateAddressRequest {
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: String,
    pub address_2: Option<String>,
    pub city: String,
    pub country_code: String,
    pub province: Option<String>,
    pub postal_code: String,
    pub phone: Option<String>,
    pub is_default_shipping: Option<bool>,
    pub is_default_billing: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// POST /store/customers/me/addresses
pub async fn create_my_address(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
    Json(body): Json<CreateAddressRequest>,
) -> Result<(StatusCode, Json<AddressResponse>), Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;

    let address = ctx
        .customer_service
        .create_customer_address(CreateCustomerAddressInput {
            customer_id,
            company: body.company,
            first_name: body.first_name,
            last_name: body.last_name,
            address_1: body.address_1,
            address_2: body.address_2,
            city: body.city,
            country_code: body.country_code,
            province: body.province,
            postal_code: body.postal_code,
            phone: body.phone,
            is_default_shipping: body.is_default_shipping,
            is_default_billing: body.is_default_billing,
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

    Ok((StatusCode::CREATED, Json(AddressResponse { address })))
}

/// GET /store/customers/me/addresses/{address_id}
pub async fn get_my_address(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
    Path(address_id): Path<String>,
) -> Result<Json<AddressResponse>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    let address = owned_address(&ctx, &customer_id, &address_id).await?;
    Ok(Json(AddressResponse { address }))
}

/// POST /store/customers/me/addresses/{address_id}
pub async fn update_my_address(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
    Path(address_id): Path<String>,
    Json(body): Json<UpdateCustomerAddressInput>,
) -> Result<Json<AddressResponse>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    owned_address(&ctx, &customer_id, &address_id).await?;

    let address = ctx
        .customer_service
        .update_customer_address(&address_id, body)
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

    Ok(Json(AddressResponse { address }))
}

/// DELETE /store/customers/me/addresses/{address_id}
pub async fn delete_my_address(
    State(ctx): State<StoreCustomerContext>,
    headers: HeaderMap,
    Path(address_id): Path<String>,
) -> Result<Json<serde_json::Value>, Response> {
    let customer_id = current_customer_id(&ctx, &headers).await?;
    owned_address(&ctx, &customer_id, &address_id).await?;

    ctx.customer_service
        .delete_customer_address(&address_id)
        .await
        .map_err(|_| address_not_found(&address_id))?;

    Ok(Json(serde_json::json!({
        "id": address_id, "object": "address", "deleted": true,
    })))
}
