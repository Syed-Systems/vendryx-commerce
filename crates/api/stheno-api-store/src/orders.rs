//! Store Order API handlers (TASKS P5-22, F6, F20).
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_flows::OrderWorkflowEngine;
use stheno_order::{OrderDTO, OrderFilter, OrderService};
use stheno_types::Error;

use crate::products::ErrorResponse;

#[derive(Clone)]
pub struct StoreOrderContext {
    pub order_service: Arc<OrderService>,
    pub order_flows: Arc<OrderWorkflowEngine>,
    /// Resolves `links.link_order_payment_collection`.
    pub links: Arc<stheno_links::PgLinkStore>,
    pub payment_service: Arc<stheno_payment::PaymentService>,
    pub notification_service: Arc<stheno_notification::NotificationService>,
    /// Resolves the caller from the bearer token so an order listing can be scoped to the
    /// shopper who owns it.
    pub auth_service: Arc<stheno_auth::AuthService>,
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "type": "unauthorized", "message": message })),
    )
        .into_response()
}

fn not_found(message: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "type": "not_found", "message": message })),
    )
        .into_response()
}

/// Resolves the caller's customer id from the bearer token.
///
/// Every `/store/orders` handler that touches one specific order goes through this.
/// `get_order` and the transfer routes previously took a bare `Path(id)` with no token at
/// all, so anyone holding an order id could read that order or move it to their own
/// account. `list_orders` was hardened against exactly this and its own doc comment says so;
/// the single-order routes were missed.
///
/// Returns 404 rather than 403 for "exists but is not yours" - 403 confirms the order is
/// real, which is the existence oracle AGENTS.md 2.10 rules out.
async fn caller_customer_id(
    ctx: &StoreOrderContext,
    headers: &axum::http::HeaderMap,
) -> Result<String, Response> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;

    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;

    // No actor means the identity exists but no customer was ever created for it. Such a
    // caller owns nothing, and must not therefore be shown everything.
    claims
        .actor_id
        .ok_or_else(|| unauthorized("Token is not bound to a customer"))
}

/// Loads an order and asserts the caller owns it, or returns 404.
async fn owned_order(
    ctx: &StoreOrderContext,
    headers: &axum::http::HeaderMap,
    order_id: &str,
) -> Result<OrderDTO, Response> {
    let caller = caller_customer_id(ctx, headers).await?;
    let order = ctx
        .order_service
        .get_order(order_id)
        .await
        .map_err(map_error)?;

    // An order with no customer is a guest order. Guests have no token to present, so
    // nothing can prove ownership of one; treat it as not-yet-claimable rather than
    // readable by whoever guesses the id.
    if order.customer_id.as_deref() != Some(caller.as_str()) {
        return Err(not_found("Order not found"));
    }
    Ok(order)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListOrdersQuery {
    pub display_id: Option<i64>,
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// An order as a storefront expects it: the DTO plus the flat totals clients read.
///
/// The order confirmation page renders `<CartTotals totals={order} />`, reading `total`,
/// `item_subtotal` and `shipping_subtotal` off the order itself. `OrderDTO` carries an
/// accounting `summary` instead, so every figure on the page a shopper sees *after paying*
/// rendered as $0.00.
#[derive(Debug, Clone, Serialize)]
pub struct StoreOrder {
    #[serde(flatten)]
    pub base: OrderDTO,
    pub item_subtotal: Decimal,
    pub item_total: Decimal,
    pub shipping_subtotal: Decimal,
    pub shipping_total: Decimal,
    pub tax_total: Decimal,
    pub discount_total: Decimal,
    pub subtotal: Decimal,
    pub total: Decimal,
    /// What was actually paid, and by which provider.
    ///
    /// The confirmation page reads `order.payment_collections[0].payments[0]` to show the
    /// amount and method. It lives behind a link table, so without this the buyer's receipt
    /// showed no payment at all.
    ///
    /// Omitted when empty, not sent as `[]`. The client's expression is
    /// `payment_collections?.[0].payments` — an *absent* field short-circuits the whole
    /// optional chain, but a present empty array does not, and `.payments` on `undefined`
    /// throws and takes the confirmation page down with it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub payment_collections: Vec<stheno_payment::PaymentCollectionDTO>,
}

/// `store_order` for callers outside this module (the cart-completion response).
pub fn store_order_public(order: OrderDTO) -> StoreOrder {
    store_order(order)
}

async fn store_order_expanded(ctx: &StoreOrderContext, order: OrderDTO) -> StoreOrder {
    let collections = match ctx
        .links
        .get_by_left("order_payment_collection", &order.id)
        .await
    {
        Ok(ids) => {
            let mut out = Vec::new();
            for id in ids {
                if let Ok(c) = ctx.payment_service.get_payment_collection(&id).await {
                    out.push(c);
                }
            }
            out
        }
        Err(_) => Vec::new(),
    };

    let mut expanded = store_order(order);
    expanded.payment_collections = collections;
    expanded
}

/// Derives the flat totals a storefront reads from the order's own rows.
///
/// `total` comes from the order summary — the figure the SAGA computed and the buyer was
/// actually charged — rather than a re-derivation that could quietly disagree with it. The
/// breakdown around it is summed from the order's adjustments and tax lines.
fn store_order(order: OrderDTO) -> StoreOrder {
    let line_total = |i: &stheno_order::LineItemDTO| i.unit_price * Decimal::from(i.quantity);
    let discounts_on = |adjustments: &[stheno_order::LineItemAdjustmentDTO]| -> Decimal {
        adjustments.iter().map(|a| a.amount).sum()
    };

    let item_subtotal: Decimal = order.items.iter().map(line_total).sum();
    let shipping_subtotal: Decimal = order.shipping_methods.iter().map(|m| m.amount).sum();

    // Promotions are stored as adjustments against the line they discount, and against the
    // shipping method for delivery promotions.
    let item_discount: Decimal = order
        .items
        .iter()
        .map(|i| discounts_on(&i.adjustments))
        .sum();
    let shipping_discount: Decimal = order
        .shipping_methods
        .iter()
        .flat_map(|m| m.adjustments.iter().map(|a| a.amount))
        .sum();
    let discount_total = item_discount + shipping_discount;

    // Tax lines carry a rate, not an amount, and apply after discounts — taxing the
    // pre-discount figure would overcharge every discounted order.
    let hundred = Decimal::from(100);
    let item_tax: Decimal = order
        .items
        .iter()
        .map(|i| {
            let taxable = line_total(i) - discounts_on(&i.adjustments);
            i.tax_lines
                .iter()
                .map(|t| taxable * t.rate / hundred)
                .sum::<Decimal>()
        })
        .sum();
    let shipping_tax: Decimal = order
        .shipping_methods
        .iter()
        .map(|m| {
            let discounts: Decimal = m.adjustments.iter().map(|a| a.amount).sum();
            let taxable = m.amount - discounts;
            m.tax_lines
                .iter()
                .map(|t| taxable * t.rate / hundred)
                .sum::<Decimal>()
        })
        .sum();
    let tax_total = item_tax + shipping_tax;

    let subtotal = item_subtotal + shipping_subtotal - discount_total;

    StoreOrder {
        item_subtotal,
        item_total: item_subtotal - item_discount,
        shipping_subtotal,
        shipping_total: shipping_subtotal - shipping_discount,
        tax_total,
        discount_total,
        subtotal,
        total: order
            .summary
            .as_ref()
            .map(|s| s.current_order_total)
            .unwrap_or(subtotal + tax_total),
        payment_collections: Vec::new(),
        base: order,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub order: StoreOrder,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<StoreOrder>,
    pub count: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestTransferRequest {
    pub order_id: String,
    pub target_customer_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransferActionRequest {
    pub order_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitOrderPaymentProofRequest {
    pub payment_method: String,
    pub transaction_id: String,
    pub proof_asset_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferResponse {
    pub order_id: String,
    pub status: String,
}

/// `GET /store/orders`
/// `GET /store/orders` — the signed-in shopper's own orders.
///
/// This took no token and set no `customer_id`, so it answered with every order in the
/// tenant to anyone holding the publishable key — which is public by design and shipped in
/// the storefront bundle. That is order enumeration: display ids, emails, addresses and
/// line items for the whole store. `OrderFilter` already had `customer_id`; nothing ever
/// set it.
pub async fn list_orders(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<OrderListResponse>, Response> {
    let customer_id = caller_customer_id(&ctx, &headers).await?;

    let filter = OrderFilter {
        customer_id: Some(customer_id),
        display_id: query.display_id,
        status: query.status,
        limit: query.limit,
        offset: query.offset,
        ..Default::default()
    };

    let (orders, count) = ctx
        .order_service
        .list_orders(filter)
        .await
        .map_err(map_error)?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    Ok(Json(OrderListResponse {
        orders: {
            let mut out = Vec::new();
            for o in orders {
                out.push(store_order_expanded(&ctx, o).await);
            }
            out
        },
        count,
        limit,
        offset,
    }))
}

/// `GET /store/orders/{id}` — one order, and only the shopper who owns it.
pub async fn get_order(
    State(ctx): State<StoreOrderContext>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<OrderResponse>, Response> {
    let order = owned_order(&ctx, &headers, &id).await?;
    Ok(Json(OrderResponse {
        order: store_order_expanded(&ctx, order).await,
    }))
}

/// `POST /store/orders/{id}/payment-proof`
///
/// Requires the signed-in customer bound to the order. The transaction reference remains
/// pending until merchant verification and never marks the order paid.
pub async fn submit_order_payment_proof(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Path(order_id): Path<String>,
    Json(body): Json<SubmitOrderPaymentProofRequest>,
) -> Result<Json<stheno_order::OrderPaymentProofDTO>, Response> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;
    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;
    let customer_id = claims
        .actor_id
        .ok_or_else(|| unauthorized("Token is not bound to a customer"))?;

    let proof = ctx
        .order_service
        .submit_manual_payment_proof(stheno_order::SubmitOrderPaymentProofInput {
            order_id,
            customer_id: customer_id.clone(),
            payment_method: body.payment_method,
            transaction_id: body.transaction_id,
            proof_asset_url: body.proof_asset_url,
        })
        .await
        .map_err(map_error)?;

    // The accepted proof remains pending verification; notification failure must not turn
    // a committed proof into an apparent submission failure for the shopper.
    match ctx.order_service.get_order(&proof.order_id).await {
        Ok(order) if order.customer_id.as_deref() == Some(customer_id.as_str()) => {
            if let Some(email) = order.email.filter(|email| email.contains('@')) {
                let subject = format!(
                    "Payment received for order #{} — verification pending",
                    order.display_id
                );
                let body = format!(
                    "<p>We received your payment reference for order <strong>#{}</strong>.</p>\
                     <p>Your payment is <strong>pending verification</strong>. The order is not marked paid until the merchant verifies the transfer.</p>",
                    order.display_id
                );
                let queued = ctx
                    .notification_service
                    .enqueue_notification_idempotent(
                        stheno_notification::SendNotificationInput {
                            to: email.clone(),
                            channel: "email".to_string(),
                            template: None,
                            subject: Some(subject),
                            body: Some(body),
                            data: None,
                            trigger_type: Some("order.payment_proof_submitted".to_string()),
                            resource_id: Some(proof.order_id.clone()),
                            resource_type: Some("order".to_string()),
                            receiver_id: Some(email),
                        },
                        &format!("order-payment-proof-submitted/{}", proof.id),
                    )
                    .await;
                if queued.is_err() {
                    tracing::warn!(
                        order_id = %proof.order_id,
                        "payment proof was accepted but its customer email could not be queued"
                    );
                }
            }
        }
        Ok(_) => tracing::warn!(
            order_id = %proof.order_id,
            "payment proof was accepted but order ownership changed before notification"
        ),
        Err(_) => tracing::warn!(
            order_id = %proof.order_id,
            "payment proof was accepted but order details could not be loaded for notification"
        ),
    }
    Ok(Json(proof))
}

/// `GET /store/orders/{id}/payment-proof`
pub async fn get_order_payment_proof(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, Response> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| unauthorized("Missing Authorization bearer token"))?;
    let claims = ctx
        .auth_service
        .verify_jwt(token)
        .await
        .map_err(|_| unauthorized("Invalid or expired token"))?;
    let customer_id = claims
        .actor_id
        .ok_or_else(|| unauthorized("Token is not bound to a customer"))?;
    let proof = ctx
        .order_service
        .get_manual_payment_proof(&order_id, &customer_id)
        .await
        .map_err(map_error)?;
    Ok(Json(serde_json::json!({ "payment_proof": proof })))
}

/// `POST /store/orders/transfer`
pub async fn request_transfer(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Json(body): Json<RequestTransferRequest>,
) -> Result<Json<TransferResponse>, Response> {
    transfer_party_order(&ctx, &headers, &body.order_id).await?;
    let req = ctx
        .order_flows
        .request_order_transfer(&body.order_id, &body.target_customer_id)
        .await
        .map_err(map_error)?;

    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

/// `POST /store/orders/transfer/accept`
pub async fn accept_transfer(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<OrderResponse>, Response> {
    transfer_party_order(&ctx, &headers, &body.order_id).await?;
    let order = ctx
        .order_flows
        .accept_order_transfer(&body.order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(OrderResponse {
        order: store_order_expanded(&ctx, order).await,
    }))
}

/// `POST /store/orders/transfer/decline`
pub async fn decline_transfer(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<TransferResponse>, Response> {
    transfer_party_order(&ctx, &headers, &body.order_id).await?;
    let req = ctx
        .order_flows
        .decline_order_transfer(&body.order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

/// `POST /store/orders/transfer/cancel`
pub async fn cancel_transfer(
    State(ctx): State<StoreOrderContext>,
    headers: axum::http::HeaderMap,
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<TransferResponse>, Response> {
    transfer_party_order(&ctx, &headers, &body.order_id).await?;
    let req = ctx
        .order_flows
        .cancel_order_transfer(&body.order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
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

/// Loads an order for a transfer action and checks the caller may act on it.
///
/// A transfer has two legitimate parties: the order's current owner, who may request,
/// decline or cancel it, and the pending target, who may accept it. Between request and
/// accept the order still belongs to the original customer, so an ownership-only check
/// would lock the target out of the one action they are allowed to take.
async fn transfer_party_order(
    ctx: &StoreOrderContext,
    headers: &axum::http::HeaderMap,
    order_id: &str,
) -> Result<OrderDTO, Response> {
    let caller = caller_customer_id(ctx, headers).await?;
    let order = ctx
        .order_service
        .get_order(order_id)
        .await
        .map_err(map_error)?;

    let is_owner = order.customer_id.as_deref() == Some(caller.as_str());
    let is_target = ctx
        .order_flows
        .pending_transfer(order_id)
        .map(|t| t.target_customer_id == caller)
        .unwrap_or(false);

    if !is_owner && !is_target {
        return Err(not_found("Order not found"));
    }
    Ok(order)
}

// ── Nested transfer routes ───────────────────────────────────────────────────
//
// `@medusajs/js-sdk` calls `/store/orders/{id}/transfer/{action}` — the order in the path.
// The flat `/store/orders/transfer/{action}` routes above take it from the body, so the
// storefront's transfer buttons hit a path nothing was mounted at and 404'd, while the
// working routes sat one shape away and were never called by anything.
//
// Both shapes stay mounted: the flat form is referenced by existing tests and by the
// server-to-server callers that predate the SDK.

/// Body of the nested request route; the order id comes from the path.
#[derive(Debug, Deserialize)]
pub struct NestedRequestTransferRequest {
    pub target_customer_id: String,
}

/// POST /store/orders/{id}/transfer/request
pub async fn request_transfer_for_order(
    State(ctx): State<StoreOrderContext>,
    Path(order_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<NestedRequestTransferRequest>,
) -> Result<Json<TransferResponse>, Response> {
    transfer_party_order(&ctx, &headers, &order_id).await?;
    let req = ctx
        .order_flows
        .request_order_transfer(&order_id, &body.target_customer_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

/// POST /store/orders/{id}/transfer/accept
pub async fn accept_transfer_for_order(
    State(ctx): State<StoreOrderContext>,
    Path(order_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<OrderResponse>, Response> {
    transfer_party_order(&ctx, &headers, &order_id).await?;
    let order = ctx
        .order_flows
        .accept_order_transfer(&order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(OrderResponse {
        order: store_order_expanded(&ctx, order).await,
    }))
}

/// POST /store/orders/{id}/transfer/decline
pub async fn decline_transfer_for_order(
    State(ctx): State<StoreOrderContext>,
    Path(order_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<TransferResponse>, Response> {
    transfer_party_order(&ctx, &headers, &order_id).await?;
    let req = ctx
        .order_flows
        .decline_order_transfer(&order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}

/// POST /store/orders/{id}/transfer/cancel
pub async fn cancel_transfer_for_order(
    State(ctx): State<StoreOrderContext>,
    Path(order_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Json<TransferResponse>, Response> {
    transfer_party_order(&ctx, &headers, &order_id).await?;
    let req = ctx
        .order_flows
        .cancel_order_transfer(&order_id)
        .await
        .map_err(map_error)?;
    Ok(Json(TransferResponse {
        order_id: req.order_id,
        status: req.status,
    }))
}
