//! Store Order API handlers (TASKS P5-22, F6, F20).
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
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

#[derive(Debug, Clone, Serialize)]
pub struct TransferResponse {
    pub order_id: String,
    pub status: String,
}

/// `GET /store/orders`
pub async fn list_orders(
    State(ctx): State<StoreOrderContext>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<OrderListResponse>, Response> {
    let filter = OrderFilter {
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

/// `GET /store/orders/{id}`
pub async fn get_order(
    State(ctx): State<StoreOrderContext>,
    Path(id): Path<String>,
) -> Result<Json<OrderResponse>, Response> {
    let order = ctx.order_service.get_order(&id).await.map_err(map_error)?;
    Ok(Json(OrderResponse {
        order: store_order_expanded(&ctx, order).await,
    }))
}

/// `POST /store/orders/transfer`
pub async fn request_transfer(
    State(ctx): State<StoreOrderContext>,
    Json(body): Json<RequestTransferRequest>,
) -> Result<Json<TransferResponse>, Response> {
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
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<OrderResponse>, Response> {
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
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<TransferResponse>, Response> {
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
    Json(body): Json<TransferActionRequest>,
) -> Result<Json<TransferResponse>, Response> {
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
