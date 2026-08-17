//! Internal models for `stheno-reporting`. Entity structs do not implement deserialization (AGENTS.md §2.9).

#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use diesel::sql_types::{BigInt, Date, Nullable, Numeric, Text, Timestamptz};
use diesel::QueryableByName;
use rust_decimal::Decimal;

#[derive(Debug, Clone, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct DailyRollup {
    #[diesel(sql_type = Date)]
    pub date: NaiveDate,
    #[diesel(sql_type = Text)]
    pub currency: String,
    #[diesel(sql_type = BigInt)]
    pub order_count: i64,
    #[diesel(sql_type = Numeric)]
    pub total_revenue: Decimal,
    #[diesel(sql_type = BigInt)]
    pub units_sold: i64,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct DailyProductRollup {
    #[diesel(sql_type = Date)]
    pub date: NaiveDate,
    #[diesel(sql_type = Text)]
    pub product_id: String,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = BigInt)]
    pub units_sold: i64,
    #[diesel(sql_type = Numeric)]
    pub total_revenue: Decimal,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct RevenueSeriesRow {
    #[diesel(sql_type = Date)]
    pub date: NaiveDate,
    #[diesel(sql_type = Text)]
    pub currency: String,
    #[diesel(sql_type = Numeric)]
    pub revenue: Decimal,
}

#[derive(Debug, Clone, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct OrdersSeriesRow {
    #[diesel(sql_type = Date)]
    pub date: NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub order_count: i64,
    #[diesel(sql_type = BigInt)]
    pub units_sold: i64,
}

#[derive(Debug, Clone, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct TopProductRow {
    #[diesel(sql_type = Text)]
    pub product_id: String,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = BigInt)]
    pub units_sold: i64,
    #[diesel(sql_type = Numeric)]
    pub total_revenue: Decimal,
}

#[derive(Debug, Clone, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct TotalAovRow {
    #[diesel(sql_type = BigInt)]
    pub order_count: i64,
    #[diesel(sql_type = Numeric)]
    pub total_revenue: Decimal,
}
