//! Store Cart & Line Item API handlers (Medusa v2 byte-compatible, TASKS P4-29).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_cart::*;
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreCartContext {
    pub cart_service: Arc<CartService>,
}

#[derive(Debug, Serialize)]
pub struct CartResponse {
    pub cart: CartDTO,
}

#[derive(Debug, Serialize)]
pub struct DeleteLineItemResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
    pub parent: CartDTO,
}

#[derive(Debug, Deserialize)]
pub struct StoreCreateCartRequest {
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub sales_channel_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: Option<String>,
    pub shipping_address: Option<CreateAddressInput>,
    pub billing_address: Option<CreateAddressInput>,
    pub items: Option<Vec<CreateLineItemInput>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StoreUpdateCartRequest {
    pub region_id: Option<String>,
    pub customer_id: Option<String>,
    pub email: Option<String>,
    pub currency_code: Option<String>,
    pub shipping_address: Option<CreateAddressInput>,
    pub billing_address: Option<CreateAddressInput>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StoreAddLineItemRequest {
    pub variant_id: Option<String>,
    pub quantity: i32,
    pub title: Option<String>,
    pub unit_price: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StoreUpdateLineItemRequest {
    pub quantity: Option<i32>,
    pub unit_price: Option<Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StoreAddShippingMethodRequest {
    pub option_id: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct StoreApplyPromotionRequest {
    pub promo_codes: Vec<String>,
}

pub async fn create_cart(
    State(ctx): State<StoreCartContext>,
    Json(body): Json<StoreCreateCartRequest>,
) -> Result<Json<CartResponse>, Response> {
    let cart = ctx
        .cart_service
        .create_cart(CreateCartInput {
            region_id: body.region_id,
            customer_id: body.customer_id,
            sales_channel_id: body.sales_channel_id,
            email: body.email,
            currency_code: body.currency_code.unwrap_or_else(|| "usd".to_string()),
            shipping_address: body.shipping_address,
            billing_address: body.billing_address,
            items: body.items,
            metadata: body.metadata,
        })
        .await
        .map_err(map_error)?;

    Ok(Json(CartResponse { cart }))
}

pub async fn get_cart(
    State(ctx): State<StoreCartContext>,
    Path(id): Path<String>,
) -> Result<Json<CartResponse>, Response> {
    let cart = ctx.cart_service.get_cart(&id).await.map_err(map_error)?;
    Ok(Json(CartResponse { cart }))
}

pub async fn update_cart(
    State(ctx): State<StoreCartContext>,
    Path(id): Path<String>,
    Json(body): Json<StoreUpdateCartRequest>,
) -> Result<Json<CartResponse>, Response> {
    let cart = ctx
        .cart_service
        .update_cart(
            &id,
            UpdateCartInput {
                region_id: body.region_id,
                customer_id: body.customer_id,
                email: body.email,
                currency_code: body.currency_code,
                shipping_address: body.shipping_address,
                billing_address: body.billing_address,
                metadata: body.metadata,
                ..Default::default()
            },
        )
        .await
        .map_err(map_error)?;

    Ok(Json(CartResponse { cart }))
}

pub async fn add_line_item(
    State(ctx): State<StoreCartContext>,
    Path(cart_id): Path<String>,
    Json(body): Json<StoreAddLineItemRequest>,
) -> Result<Json<CartResponse>, Response> {
    ctx.cart_service
        .add_line_item(
            &cart_id,
            CreateLineItemInput {
                title: body.title.unwrap_or_else(|| "Item".to_string()),
                subtitle: None,
                thumbnail: None,
                quantity: body.quantity,
                unit_price: body.unit_price.unwrap_or(Decimal::ZERO),
                variant_id: body.variant_id,
                product_id: None,
                product_title: None,
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: None,
                variant_sku: None,
                variant_barcode: None,
                variant_title: None,
                variant_option_values: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_tax_inclusive: Some(false),
                is_custom_price: Some(false),
                compare_at_unit_price: None,
                metadata: body.metadata,
            },
        )
        .await
        .map_err(map_error)?;

    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;
    Ok(Json(CartResponse { cart }))
}

pub async fn update_line_item(
    State(ctx): State<StoreCartContext>,
    Path((cart_id, line_id)): Path<(String, String)>,
    Json(body): Json<StoreUpdateLineItemRequest>,
) -> Result<Json<CartResponse>, Response> {
    ctx.cart_service
        .update_line_item(
            &cart_id,
            &line_id,
            UpdateLineItemInput {
                quantity: body.quantity,
                unit_price: body.unit_price,
                metadata: body.metadata,
                ..Default::default()
            },
        )
        .await
        .map_err(map_error)?;

    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;
    Ok(Json(CartResponse { cart }))
}

pub async fn delete_line_item(
    State(ctx): State<StoreCartContext>,
    Path((cart_id, line_id)): Path<(String, String)>,
) -> Result<Json<DeleteLineItemResponse>, Response> {
    ctx.cart_service
        .delete_line_item(&cart_id, &line_id)
        .await
        .map_err(map_error)?;

    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;
    Ok(Json(DeleteLineItemResponse {
        id: line_id,
        object: "line-item".to_string(),
        deleted: true,
        parent: cart,
    }))
}

pub async fn add_shipping_method(
    State(ctx): State<StoreCartContext>,
    Path(cart_id): Path<String>,
    Json(body): Json<StoreAddShippingMethodRequest>,
) -> Result<Json<CartResponse>, Response> {
    ctx.cart_service
        .add_shipping_method(
            &cart_id,
            CreateShippingMethodInput {
                name: "Standard Shipping".to_string(),
                amount: Decimal::ZERO,
                is_tax_inclusive: Some(false),
                shipping_option_id: Some(body.option_id),
                data: body.data,
                metadata: None,
            },
        )
        .await
        .map_err(map_error)?;

    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;
    Ok(Json(CartResponse { cart }))
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
