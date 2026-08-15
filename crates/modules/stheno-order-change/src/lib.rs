//! `stheno-order-change`: Standalone Order Change and Edit Engine (apply/preview, diff calculations).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_types::{Id, Money, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderChangeStatus {
    Pending,
    Requested,
    Confirmed,
    Declined,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderActionType {
    ItemAdd,
    ItemRemove,
    ItemUpdate,
    ShippingAdd,
    ShippingRemove,
    ReceiveReturn,
    RefundAdd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAction {
    pub id: Id<OrderAction>,
    pub action: OrderActionType,
    pub line_item_id: Option<String>,
    pub quantity: Option<i64>,
    pub unit_price: Option<Money>,
    pub amount: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderChangePreview {
    pub current_total: Money,
    pub difference_total: Money,
    pub new_total: Money,
    pub actions_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderChange {
    pub id: Id<OrderChange>,
    pub order_id: String,
    pub status: OrderChangeStatus,
    pub actions: Vec<OrderAction>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

impl OrderChange {
    pub fn new(order_id: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            order_id: order_id.into(),
            status: OrderChangeStatus::Pending,
            actions: Vec::new(),
            created_at: Utc::now(),
            confirmed_at: None,
        }
    }

    pub fn add_action(&mut self, action: OrderAction) {
        self.actions.push(action);
    }

    pub fn preview(&self, current_order_total: Money) -> Result<OrderChangePreview> {
        let currency = current_order_total.currency;
        let mut delta = Money::zero(currency);

        for act in &self.actions {
            match act.action {
                OrderActionType::ItemAdd => {
                    if let (Some(qty), Some(price)) = (act.quantity, act.unit_price) {
                        let item_total = price * Decimal::from(qty);
                        delta = (delta + item_total)?;
                    }
                }
                OrderActionType::ItemRemove => {
                    if let Some(amt) = act.amount {
                        delta = (delta - amt)?;
                    }
                }
                OrderActionType::ItemUpdate => {
                    if let Some(amt) = act.amount {
                        delta = (delta + amt)?;
                    }
                }
                OrderActionType::ShippingAdd => {
                    if let Some(amt) = act.amount {
                        delta = (delta + amt)?;
                    }
                }
                OrderActionType::ShippingRemove => {
                    if let Some(amt) = act.amount {
                        delta = (delta - amt)?;
                    }
                }
                OrderActionType::RefundAdd | OrderActionType::ReceiveReturn => {
                    if let Some(amt) = act.amount {
                        delta = (delta - amt)?;
                    }
                }
            }
        }

        let new_total = (current_order_total + delta)?;

        Ok(OrderChangePreview {
            current_total: current_order_total,
            difference_total: delta,
            new_total,
            actions_count: self.actions.len(),
        })
    }

    pub fn confirm(&mut self) {
        self.status = OrderChangeStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
    }
}
