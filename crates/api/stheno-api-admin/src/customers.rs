//! Admin Customers and Customer Groups API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_customer::{
    CreateCustomerGroupInput, CreateCustomerInput, CustomerService, UpdateCustomerGroupInput,
    UpdateCustomerInput,
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
