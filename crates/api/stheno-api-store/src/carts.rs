//! Store Cart & Line Item API handlers (Medusa v2 byte-compatible, TASKS P4-29, P13-10).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use stheno_cart::*;
use stheno_promotion::PromotionService;
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreCartContext {
    pub cart_service: Arc<CartService>,
    pub promotion_service: Option<Arc<PromotionService>>,
    /// Prices line items server-side.
    ///
    /// `add_line_item` took `unit_price` from the request body and defaulted it to zero,
    /// so a shopper could name their own price — or omit the field and get the item free.
    /// A price is not client input, and a store endpoint is an untrusted boundary.
    pub pricing_service: Arc<stheno_pricing::PricingService>,
    /// Supplies the currency to price in, from the cart's region.
    pub region_service: Arc<stheno_region::RegionService>,
    pub product_service: Arc<stheno_product::ProductService>,
    /// Supplies a shipping option's name and identity when one is attached to a cart.
    pub fulfillment_service: Arc<stheno_fulfillment::FulfillmentService>,
    pub links: Arc<stheno_links::PgLinkStore>,
    pub payment_service: Arc<stheno_payment::PaymentService>,
}

/// A cart as a storefront expects it: the DTO plus its expanded `region`.
///
/// Medusa's `StoreCart` carries the whole region object, and the storefront depends on it
/// structurally — the cart page renders its summary (and therefore the "Proceed to
/// Checkout" button) only `if (cart && cart.region)`. Returning `region_id` alone left a
/// cart that listed its items correctly and offered no way to buy them.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreCart {
    /// Flattened, so the wire shape is one object — `region` sits alongside the cart's own
    /// fields exactly as Medusa returns it.
    #[serde(flatten)]
    pub base: CartDTO,
    /// `None` for a cart with no region yet, rather than omitted: clients test for its
    /// presence.
    pub region: Option<stheno_region::RegionDTO>,
    /// The cart's open payment collection, once one exists.
    ///
    /// The checkout's review step renders only `if (cart.payment_collection)`, and the
    /// place-order button reads its `payment_sessions`. Both live behind
    /// `links.link_cart_payment_collection`, which nothing expanded — so the final step of
    /// the checkout never appeared.
    pub payment_collection: Option<stheno_payment::PaymentCollectionDTO>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartResponse {
    pub cart: StoreCart,
}

/// Attaches the region to a cart on its way out.
///
/// A failed region lookup yields `None` rather than an error: a cart whose region was
/// deleted should still be readable, so the shopper can pick another.
async fn with_region(ctx: &StoreCartContext, cart: CartDTO) -> CartResponse {
    let region = match &cart.region_id {
        Some(id) => ctx.region_service.get_region(id).await.ok(),
        None => None,
    };

    // A cart has at most one open collection; a missing link simply means checkout has not
    // reached the payment step yet.
    let payment_collection = match ctx
        .links
        .get_by_left("cart_payment_collection", &cart.id)
        .await
    {
        Ok(ids) => match ids.first() {
            Some(id) => ctx.payment_service.get_payment_collection(id).await.ok(),
            None => None,
        },
        Err(_) => None,
    };

    CartResponse {
        cart: StoreCart {
            base: cart,
            region,
            payment_collection,
        },
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteLineItemResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
    /// The cart the item was removed from — same expanded shape as everywhere else, so a
    /// client can render straight from it without a second request.
    pub parent: StoreCart,
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
    pub unit_price: Option<Decimal>,
    pub title: Option<String>,
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

    Ok(Json(with_region(&ctx, cart).await))
}

pub async fn get_cart(
    State(ctx): State<StoreCartContext>,
    Path(id): Path<String>,
) -> Result<Json<CartResponse>, Response> {
    let cart = ctx.cart_service.get_cart(&id).await.map_err(map_error)?;
    Ok(Json(with_region(&ctx, cart).await))
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

    Ok(Json(with_region(&ctx, cart).await))
}

/// Resolves what a variant costs in this cart, from the catalogue rather than the request.
/// The currency a cart prices in: its own, else its region's.
///
/// Never the request's — the same reason line-item prices are not the request's.
async fn cart_currency(ctx: &StoreCartContext, cart: &CartDTO) -> String {
    if !cart.currency_code.is_empty() {
        return cart.currency_code.to_lowercase();
    }
    match &cart.region_id {
        Some(region_id) => ctx
            .region_service
            .get_region(region_id)
            .await
            .map(|r| r.currency_code.to_lowercase())
            .unwrap_or_else(|_| "usd".to_string()),
        None => "usd".to_string(),
    }
}

async fn price_for_variant(
    ctx: &StoreCartContext,
    cart: &CartDTO,
    variant_id: &str,
    quantity: i32,
) -> Result<Decimal, Response> {
    let currency = cart_currency(ctx, cart).await;

    let prices = ctx
        .pricing_service
        .calculate_for_variants(
            &[variant_id.to_string()],
            &stheno_pricing::PricingContext {
                currency_code: currency,
                region_id: cart.region_id.clone(),
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(quantity),
                as_of: None,
            },
        )
        .await
        .map_err(map_error)?;

    // No price is a refusal, not a zero. Adding an unpriced variant to a cart silently at
    // 0.00 is how a store gives away stock.
    prices
        .get(variant_id)
        .map(|p| p.calculated_amount)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    r#type: "invalid_data".to_string(),
                    message: format!("Variant {variant_id} has no price in this currency."),
                }),
            )
                .into_response()
        })
}

pub async fn add_line_item(
    State(ctx): State<StoreCartContext>,
    Path(cart_id): Path<String>,
    Json(body): Json<StoreAddLineItemRequest>,
) -> Result<Json<CartResponse>, Response> {
    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;

    // A line item must name a variant: it is the only thing that has a price, and without
    // one there is nothing to look up.
    let variant_id = body.variant_id.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                r#type: "invalid_data".to_string(),
                message: "A line item requires variant_id.".to_string(),
            }),
        )
            .into_response()
    })?;

    let unit_price = price_for_variant(&ctx, &cart, &variant_id, body.quantity).await?;

    // Titles are cosmetic and may come from the client; the variant is the source for the
    // ones that identify what was bought.
    let variant = ctx.product_service.get_variant(&variant_id).await.ok();

    ctx.cart_service
        .add_line_item(
            &cart_id,
            CreateLineItemInput {
                title: body
                    .title
                    .or_else(|| variant.as_ref().map(|v| v.title.clone()))
                    .unwrap_or_else(|| "Item".to_string()),
                subtitle: None,
                thumbnail: None,
                quantity: body.quantity,
                unit_price,
                variant_id: Some(variant_id),
                product_id: None,
                product_title: None,
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: None,
                variant_sku: variant.as_ref().and_then(|v| v.sku.clone()),
                variant_barcode: variant.as_ref().and_then(|v| v.barcode.clone()),
                variant_title: variant.as_ref().map(|v| v.title.clone()),
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
    Ok(Json(with_region(&ctx, cart).await))
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
                // Same reason as `add_line_item`: a shopper changing the quantity must not
                // be able to change the price on the way through.
                unit_price: None,
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
    Ok(Json(with_region(&ctx, cart).await))
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
        parent: with_region(&ctx, cart).await.cart,
    }))
}

pub async fn add_shipping_method(
    State(ctx): State<StoreCartContext>,
    Path(cart_id): Path<String>,
    Json(body): Json<StoreAddShippingMethodRequest>,
) -> Result<Json<CartResponse>, Response> {
    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;

    // The name and the price both came from here rather than from the option: every
    // delivery was called "Standard Shipping" and cost zero, whatever the merchant had
    // configured. A shipping option with no price set stays free — that is a legitimate
    // configuration, unlike a variant with no price — but it is now the merchant's choice
    // rather than a hardcoded constant.
    let option = ctx
        .fulfillment_service
        .get_shipping_option(&body.option_id)
        .await
        .map_err(map_error)?;

    let currency = cart_currency(&ctx, &cart).await;
    let amount = ctx
        .pricing_service
        .calculate_for_shipping_options(
            std::slice::from_ref(&body.option_id),
            &stheno_pricing::PricingContext {
                currency_code: currency,
                region_id: cart.region_id.clone(),
                customer_group_id: None,
                sales_channel_id: None,
                quantity: Some(1),
                as_of: None,
            },
        )
        .await
        .map_err(map_error)?
        .get(&body.option_id)
        .map(|p| p.calculated_amount)
        .unwrap_or(Decimal::ZERO);

    ctx.cart_service
        .add_shipping_method(
            &cart_id,
            CreateShippingMethodInput {
                name: option.name,
                amount,
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
    Ok(Json(with_region(&ctx, cart).await))
}

pub async fn apply_promotions(
    State(ctx): State<StoreCartContext>,
    Path(cart_id): Path<String>,
    Json(body): Json<StoreApplyPromotionRequest>,
) -> Result<Json<CartResponse>, Response> {
    let promo_service = match &ctx.promotion_service {
        Some(s) => s,
        None => {
            return Err(map_error(Error::Invalid {
                code: "promotions_disabled",
                message: "Promotions not configured".into(),
                field: None,
            }))
        }
    };

    let cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;

    let mut promotions = Vec::new();
    for code in &body.promo_codes {
        match promo_service.get_promotion_by_code(code).await {
            Ok(p) => promotions.push(p),
            Err(_) => {
                promo_service.record_failed_promo_attempt(&cart_id);
                return Err(map_error(Error::NotFound {
                    entity: "promotion",
                    id: code.clone(),
                }));
            }
        }
    }

    let mut item_contexts = Vec::new();
    for item in &cart.items {
        item_contexts.push(stheno_promotion::PromotionItemContext {
            id: item.id.clone(),
            product_id: item.product_id.clone(),
            product_category_id: None,
            product_collection_id: item.product_collection.clone(),
            product_type_id: item.product_type.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            subtotal: item.unit_price * Decimal::from(item.quantity),
            is_discountable: item.is_discountable,
        });
    }

    let mut shipping_contexts = Vec::new();
    for sm in &cart.shipping_methods {
        shipping_contexts.push(stheno_promotion::PromotionShippingContext {
            id: sm.id.clone(),
            shipping_option_id: sm.shipping_option_id.clone(),
            amount: sm.amount,
        });
    }

    let mut attributes = HashMap::new();
    attributes.insert("currency_code".to_string(), cart.currency_code.clone());
    if let Some(cid) = &cart.customer_id {
        attributes.insert("customer_id".to_string(), cid.clone());
    }

    let eval_context = stheno_promotion::PromotionEvaluationContext {
        attributes,
        items: item_contexts,
        shipping_methods: shipping_contexts,
    };

    let adjustments = promo_service
        .evaluate_promotions(&promotions, &eval_context)
        .await;

    for item in &cart.items {
        let _ = ctx
            .cart_service
            .delete_line_item_adjustments(&item.id)
            .await;
    }

    for adj in adjustments {
        if let Some(item_id) = &adj.line_item_id {
            let _ = ctx
                .cart_service
                .add_line_item_adjustment(
                    item_id,
                    CreateLineItemAdjustmentInput {
                        amount: adj.amount,
                        promotion_id: Some(adj.promotion_id),
                        code: Some(adj.code),
                        description: adj.description,
                    },
                )
                .await
                .map_err(map_error)?;
        } else if let Some(sm_id) = &adj.shipping_method_id {
            let _ = ctx
                .cart_service
                .add_shipping_method_adjustment(
                    sm_id,
                    CreateShippingMethodAdjustmentInput {
                        amount: adj.amount,
                        promotion_id: Some(adj.promotion_id),
                        code: Some(adj.code),
                        description: adj.description,
                    },
                )
                .await
                .map_err(map_error)?;
        }
    }

    let updated_cart = ctx
        .cart_service
        .get_cart(&cart_id)
        .await
        .map_err(map_error)?;
    Ok(Json(with_region(&ctx, updated_cart).await))
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
