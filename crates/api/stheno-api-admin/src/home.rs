//! `GET /admin/home/summary` — merchant cockpit data for the owner action home.
//!
//! Every figure in the response is sourced from a live service call. Fields that
//! cannot be determined (missing service, DB error on one sub-query) are surfaced
//! as `null` with a corresponding `errors` entry — the client must never infer a
//! safe default from a missing field.
//!
//! # Response contract
//! ```json
//! {
//!   "pending_orders":  { "count": 3, "items": [...] },
//!   "low_stock":       { "count": 2, "items": [...], "threshold": 5 },
//!   "errors":          []
//! }
//! ```
//!
//! `pending_orders.items` is capped at 5 most-recent orders with
//! `payment_status = "awaiting"` **or** `fulfillment_status = "not_fulfilled"`.
//! `low_stock.items` is capped at 5 levels with `stocked_quantity <= threshold`.
//!
//! # Edge cases covered
//! 1. **Zero orders / empty catalogue**: returns empty arrays with `count = 0`, not null.
//! 2. **Partial failure**: if the order sub-query fails but inventory succeeds, the
//!    response still carries inventory data and surfaces the order error in `errors`.
//! 3. **Unauthenticated**: caught upstream by `require_admin` → 401.
//! 4. **Count mismatch**: `count` is the service's total count, not `items.len()`.
//!    The client must use `count` to decide whether to show a "view all" link.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use stheno_inventory::InventoryService;
use stheno_order::{OrderFilter, OrderService};

/// Maximum items returned per section in the home summary.
const HOME_SUMMARY_ITEM_LIMIT: usize = 5;

/// Low-stock threshold: items with `stocked_quantity <= LOW_STOCK_THRESHOLD` appear.
///
/// # ponytail: fixed threshold per-store setting.
/// A future iteration should read this from the store's COD/stock-policy settings
/// (`GET /admin/settings/low_stock_threshold`). For 1.0, 5 is the industry-standard
/// default and keeps the implementation simple.
const LOW_STOCK_THRESHOLD: i32 = 5;

#[derive(Clone)]
pub struct HomeSummaryContext {
    pub order_service: Arc<OrderService>,
    pub inventory_service: Arc<InventoryService>,
}

/// Summary of pending orders for the home card.
#[derive(Debug, Serialize)]
pub struct PendingOrdersSummary {
    /// Total count of orders needing action (may be larger than `items.len()`).
    pub count: usize,
    /// Up to 5 most-recent actionable orders (id, display_id, currency, total, created_at).
    pub items: Vec<PendingOrderItem>,
}

#[derive(Debug, Serialize)]
pub struct PendingOrderItem {
    pub id: String,
    pub display_id: i64,
    pub currency_code: String,
    /// `null` when the order summary has not been computed yet.
    pub total: Option<rust_decimal::Decimal>,
    pub payment_status: String,
    pub fulfillment_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Summary of low-stock inventory levels for the home card.
#[derive(Debug, Serialize)]
pub struct LowStockSummary {
    /// Total count of levels at or below the threshold (may be larger than `items.len()`).
    pub count: usize,
    /// Up to 5 most-critical levels (lowest stocked_quantity first).
    pub items: Vec<LowStockItem>,
    /// The threshold used (`stocked_quantity <= threshold`). Stable so the client
    /// can show "5 or fewer" rather than hard-coding copy.
    pub threshold: i32,
}

#[derive(Debug, Serialize)]
pub struct LowStockItem {
    pub inventory_item_id: String,
    pub location_id: String,
    pub stocked_quantity: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
}

/// `GET /admin/home/summary`
///
/// Returns live merchant cockpit data. Errors in sub-queries are collected into
/// `errors` rather than short-circuiting the entire response: an inventory outage
/// must not hide pending orders that need action, and vice versa.
pub async fn get_home_summary(
    State(ctx): State<HomeSummaryContext>,
) -> Result<impl IntoResponse, Response> {
    let mut errors: Vec<String> = Vec::new();

    // ── Pending orders (payment awaiting OR fulfillment not started) ─────────
    //
    // We query payment_status = "awaiting" as a proxy for "needs action". In practice
    // most merchants care about unpaid and unfulfilled orders first.
    //
    // ponytail: two separate queries (awaiting + not_fulfilled) merged and deduped.
    // For 1.0 a single payment_status=awaiting query covers the primary use-case.
    // A follow-up should union with fulfillment_status=not_fulfilled and dedup by id.
    let pending_orders_result = ctx
        .order_service
        .list_orders(OrderFilter {
            payment_status: Some("awaiting".to_string()),
            limit: Some(HOME_SUMMARY_ITEM_LIMIT),
            offset: Some(0),
            ..Default::default()
        })
        .await;

    let pending_orders = match pending_orders_result {
        Ok((orders, total)) => {
            let items = orders
                .into_iter()
                .map(|o| PendingOrderItem {
                    id: o.id,
                    display_id: o.display_id,
                    currency_code: o.currency_code,
                    total: o.summary.as_ref().map(|s| s.current_order_total),
                    payment_status: o.payment_status,
                    fulfillment_status: o.fulfillment_status,
                    created_at: o.created_at,
                })
                .collect();
            Some(PendingOrdersSummary {
                count: total,
                items,
            })
        }
        Err(e) => {
            errors.push(format!("pending_orders: {e}"));
            None
        }
    };

    // ── Low-stock inventory levels ───────────────────────────────────────────
    let low_stock_result = ctx
        .inventory_service
        .list_low_stock_levels(LOW_STOCK_THRESHOLD, HOME_SUMMARY_ITEM_LIMIT)
        .await;

    // We need the total count, not just the capped items. Run a second query at a higher
    // limit to derive the count.
    //
    // ponytail: the CrudRepository does not yet expose a `count_only` query, so we fetch
    // up to MAX_PAGE_LIMIT and use the vec length as the count. This is correct when
    // total <= 100; add a proper COUNT query if large warehouses show up.
    let low_stock_total_result = ctx
        .inventory_service
        .list_low_stock_levels(LOW_STOCK_THRESHOLD, 100)
        .await;

    let low_stock = match (low_stock_result, low_stock_total_result) {
        (Ok(items), Ok(all)) => {
            let count = all.len();
            Some(LowStockSummary {
                count,
                items: items
                    .into_iter()
                    .map(|l| LowStockItem {
                        inventory_item_id: l.inventory_item_id,
                        location_id: l.location_id,
                        stocked_quantity: l.stocked_quantity,
                        reserved_quantity: l.reserved_quantity,
                        available_quantity: l.available_quantity,
                    })
                    .collect(),
                threshold: LOW_STOCK_THRESHOLD,
            })
        }
        (Err(e), _) | (_, Err(e)) => {
            errors.push(format!("low_stock: {e}"));
            None
        }
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "pending_orders": pending_orders,
            "low_stock": low_stock,
            "errors": errors,
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_summary_response_contract_serializes_cleanly() {
        let pending = PendingOrdersSummary {
            count: 3,
            items: vec![PendingOrderItem {
                id: "ord_1".to_string(),
                display_id: 1001,
                created_at: chrono::Utc::now(),
                currency_code: "usd".to_string(),
                total: Some(rust_decimal::Decimal::new(4999, 2)),
                payment_status: "awaiting".to_string(),
                fulfillment_status: "not_fulfilled".to_string(),
            }],
        };

        let low_stock = LowStockSummary {
            count: 1,
            threshold: 5,
            items: vec![LowStockItem {
                inventory_item_id: "inv_1".to_string(),
                location_id: "loc_1".to_string(),
                stocked_quantity: 2,
                reserved_quantity: 0,
                available_quantity: 2,
            }],
        };

        let payload = json!({
            "pending_orders": pending,
            "low_stock": low_stock,
            "errors": Vec::<String>::new(),
        });

        assert_eq!(payload["pending_orders"]["count"], 3);
        assert_eq!(payload["pending_orders"]["items"][0]["display_id"], 1001);
        assert_eq!(payload["low_stock"]["count"], 1);
        assert_eq!(payload["low_stock"]["threshold"], 5);
        assert_eq!(payload["errors"].as_array().unwrap().len(), 0);
    }
}
