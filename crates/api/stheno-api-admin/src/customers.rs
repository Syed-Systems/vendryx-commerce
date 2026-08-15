//! Admin Customers and Customer Groups API Handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use stheno_api_core::{DeleteResponse, ErrorResponse, GetResponse, ListResponse};
use stheno_customer::{
    CreateCustomerGroupInput, CreateCustomerInput, CustomerDTO, CustomerGroupDTO,
    CustomerService, UpdateCustomerGroupInput, UpdateCustomerInput,
};

// 1. Customer Handlers
pub async fn list_customers(
    State(service): State<Arc<CustomerService>>,
) -> Result<Json<ListResponse<Vec<CustomerDTO>>>, Response> {
    let all = service.list_customers().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse {
        data: all,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_customer(
    State(service): State<Arc<CustomerService>>,
    Json(input): Json<CreateCustomerInput>,
) -> Result<(StatusCode, Json<GetResponse<CustomerDTO>>), Response> {
    let customer = service.create_customer(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: customer })))
}

pub async fn get_customer(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<Json<GetResponse<CustomerDTO>>, Response> {
    match service.get_customer(&id).await {
        Ok(customer) => Ok(Json(GetResponse { data: customer })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &format!("Customer with id {} was not found", id))),
        )
            .into_response()),
    }
}

pub async fn update_customer(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCustomerInput>,
) -> Result<Json<GetResponse<CustomerDTO>>, Response> {
    let customer = service.update_customer(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: customer }))
}

pub async fn delete_customer(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, Response> {
    service.delete_customer(&id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(DeleteResponse {
        id,
        object: "customer".to_string(),
        deleted: true,
    }))
}

// 2. Customer Groups Handlers
pub async fn list_customer_groups(
    State(service): State<Arc<CustomerService>>,
) -> Result<Json<ListResponse<Vec<CustomerGroupDTO>>>, Response> {
    let all = service.list_customer_groups().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("unexpected_state", &e.to_string())),
        )
            .into_response()
    })?;
    let count = all.len();
    Ok(Json(ListResponse {
        data: all,
        count,
        offset: 0,
        limit: 20,
    }))
}

pub async fn create_customer_group(
    State(service): State<Arc<CustomerService>>,
    Json(input): Json<CreateCustomerGroupInput>,
) -> Result<(StatusCode, Json<GetResponse<CustomerGroupDTO>>), Response> {
    let group = service.create_customer_group(input).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_data", &e.to_string())),
        )
            .into_response()
    })?;
    Ok((StatusCode::CREATED, Json(GetResponse { data: group })))
}

pub async fn update_customer_group(
    State(service): State<Arc<CustomerService>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateCustomerGroupInput>,
) -> Result<Json<GetResponse<CustomerGroupDTO>>, Response> {
    let group = service.update_customer_group(&id, input).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", &e.to_string())),
        )
            .into_response()
    })?;
    Ok(Json(GetResponse { data: group }))
}
