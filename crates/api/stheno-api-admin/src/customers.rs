//! Admin Customers and Customer Groups API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_customer::{
    CreateCustomerAddressInput, CreateCustomerGroupInput, CreateCustomerInput, CustomerService,
    UpdateCustomerAddressInput, UpdateCustomerGroupInput, UpdateCustomerInput,
};

// 1. Customer Handlers
pub async fn list_customers(
    State(service): State<Arc<CustomerService>>,
) -> Result<impl IntoResponse, Response> {
    let all = service.list_customers().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "customers": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_customer(
    State(service): State<Arc<CustomerService>>,
    Json(input): Json<CreateCustomerInput>,
) -> Result<impl IntoResponse, Response> {
    let customer = service.create_customer(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "customer": customer,
            "data": customer,
        })),
    ))
}

pub async fn get_customer(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    match service.get_customer(&id).await {
        Ok(customer) => Ok((
            StatusCode::OK,
            Json(json!({
                "customer": customer,
                "data": customer,
            })),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Customer with id {} was not found", id),
            )),
        )
            .into_response()),
    }
}

pub async fn update_customer(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCustomerInput>,
) -> Result<impl IntoResponse, Response> {
    let customer = service.update_customer(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "customer": customer,
            "data": customer,
        })),
    ))
}

pub async fn delete_customer(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_customer(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "object": "customer",
            "deleted": true
        })),
    ))
}

// 2. Customer Groups Handlers
pub async fn list_customer_groups(
    State(service): State<Arc<CustomerService>>,
) -> Result<impl IntoResponse, Response> {
    let all = service.list_customer_groups().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "customer_groups": all,
            "data": all,
            "count": count,
            "offset": 0,
            "limit": 20,
        })),
    ))
}

pub async fn create_customer_group(
    State(service): State<Arc<CustomerService>>,
    Json(input): Json<CreateCustomerGroupInput>,
) -> Result<impl IntoResponse, Response> {
    let group = service.create_customer_group(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "customer_group": group,
            "data": group,
        })),
    ))
}

pub async fn update_customer_group(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCustomerGroupInput>,
) -> Result<impl IntoResponse, Response> {
    let group = service
        .update_customer_group(&id, input)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("not_found", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "customer_group": group,
            "data": group,
        })),
    ))
}

pub async fn get_customer_group(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let group = service.get_customer_group(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "customer_group": group, "data": group })),
    ))
}

pub async fn delete_customer_group(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    service.delete_customer_group(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "customer_group", "deleted": true })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchCustomerGroupCustomersRequest {
    pub add: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
}

/// POST /admin/customer-groups/{id}/customers
pub async fn batch_customer_group_customers(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(body): Json<BatchCustomerGroupCustomersRequest>,
) -> Result<impl IntoResponse, Response> {
    if let Some(to_add) = body.add {
        for cid in to_add {
            let _ = service.add_customer_to_group(&cid, &id).await;
        }
    }
    if let Some(to_remove) = body.remove {
        for cid in to_remove {
            let _ = service.remove_customer_from_group(&cid, &id).await;
        }
    }
    let group = service.get_customer_group(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "customer_group": group, "data": group })),
    ))
}

// ── Customer Addresses Handlers ───────────────────────────────────────────────

pub async fn list_customer_addresses(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let customer = service.get_customer(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "addresses": customer.addresses,
            "data": customer.addresses,
            "count": customer.addresses.len()
        })),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct AdminCreateCustomerAddressPayload {
    pub company: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub is_default_shipping: Option<bool>,
    pub is_default_billing: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn add_customer_address(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(body): Json<AdminCreateCustomerAddressPayload>,
) -> Result<impl IntoResponse, Response> {
    let input = CreateCustomerAddressInput {
        customer_id: id.clone(),
        company: body.company,
        first_name: body.first_name,
        last_name: body.last_name,
        address_1: body.address_1.unwrap_or_default(),
        address_2: body.address_2,
        city: body.city.unwrap_or_default(),
        country_code: body.country_code.unwrap_or_else(|| "US".to_string()),
        province: body.province,
        postal_code: body.postal_code.unwrap_or_default(),
        phone: body.phone,
        is_default_shipping: body.is_default_shipping,
        is_default_billing: body.is_default_billing,
        metadata: body.metadata,
    };
    let address = service.create_customer_address(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    let customer = service.get_customer(&id).await.ok();
    Ok((
        StatusCode::CREATED,
        Json(json!({ "customer": customer, "address": address, "data": address })),
    ))
}

pub async fn get_customer_address(
    State(service): State<Arc<CustomerService>>,
    Path((id, address_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    let customer = service.get_customer(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    let addr = customer.addresses.into_iter().find(|a| a.id == address_id);
    match addr {
        Some(a) => Ok((StatusCode::OK, Json(json!({ "address": a, "data": a })))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Address not found")),
        )
            .into_response()),
    }
}

pub async fn update_customer_address(
    State(service): State<Arc<CustomerService>>,
    Path((_id, address_id)): Path<(String, String)>,
    Json(body): Json<UpdateCustomerAddressInput>,
) -> Result<impl IntoResponse, Response> {
    let addr = service
        .update_customer_address(&address_id, body)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_data", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "address": addr, "data": addr })),
    ))
}

pub async fn delete_customer_address(
    State(service): State<Arc<CustomerService>>,
    Path((_id, address_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, Response> {
    service
        .delete_customer_address(&address_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("not_found", &e.to_string())),
            )
                .into_response()
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "id": address_id, "object": "address", "deleted": true })),
    ))
}

pub async fn manage_customer_groups(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(body): Json<BatchCustomerGroupCustomersRequest>,
) -> Result<impl IntoResponse, Response> {
    if let Some(to_add) = body.add {
        for gid in to_add {
            let _ = service.add_customer_to_group(&id, &gid).await;
        }
    }
    if let Some(to_remove) = body.remove {
        for gid in to_remove {
            let _ = service.remove_customer_from_group(&id, &gid).await;
        }
    }
    let customer = service.get_customer(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({ "customer": customer, "data": customer })),
    ))
}
