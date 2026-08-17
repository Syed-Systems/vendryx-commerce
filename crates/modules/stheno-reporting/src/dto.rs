//! DTOs for `stheno-reporting` (crossing module boundary).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPlacedRollupItem {
    pub product_id: String,
    pub title: String,
    pub units: i64,
    pub total_revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordOrderPlacedInput {
    pub date: NaiveDate,
    pub currency: String,
    pub total_revenue: Decimal,
    pub units: i64,
    pub items: Vec<OrderPlacedRollupItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordOrderDeltaInput {
    pub date: NaiveDate,
    pub currency: String,
    pub delta_revenue: Decimal,
    pub delta_units: i64,
    pub delta_orders: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueOverTimeItem {
    pub date: NaiveDate,
    pub currency: String,
    pub revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueOverTimeReport {
    pub currency: String,
    pub total_revenue: Decimal,
    pub time_series: Vec<RevenueOverTimeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersOverTimeItem {
    pub date: NaiveDate,
    pub order_count: i64,
    pub units_sold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersOverTimeReport {
    pub total_orders: i64,
    pub total_units: i64,
    pub time_series: Vec<OrdersOverTimeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProductItem {
    pub product_id: String,
    pub title: String,
    pub units_sold: i64,
    pub total_revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProductsReport {
    pub top_products: Vec<TopProductItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AverageOrderValueReport {
    pub currency: String,
    pub average_order_value: Decimal,
    pub order_count: i64,
    pub total_revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEnvelope<T> {
    pub report_type: String,
    pub window_days: Option<u32>,
    pub data: T,
}
