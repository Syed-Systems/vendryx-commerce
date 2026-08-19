//! `stheno-reporting::service`: Materialised daily rollups and analytical reporting service (P16).

use chrono::{NaiveDate, Utc};
use diesel::sql_types::{BigInt, Date, Numeric, Text};
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use stheno_db::tenant;
use stheno_db::DbPool;
use stheno_types::{Error, Result};

use crate::dto::{
    AverageOrderValueReport, OrdersOverTimeItem, OrdersOverTimeReport, RecordOrderDeltaInput,
    RecordOrderPlacedInput, RevenueOverTimeItem, RevenueOverTimeReport, TopProductItem,
    TopProductsReport,
};
use crate::models::{OrdersSeriesRow, RevenueSeriesRow, TopProductRow, TotalAovRow};

#[derive(Clone)]
pub struct ReportingService {
    pool: DbPool,
}

impl ReportingService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Incremental rollup update on order.placed event (P16-03)
    pub async fn record_order_placed(&self, input: RecordOrderPlacedInput) -> Result<()> {
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {


                // 1. Upsert daily_rollup
                let rollup_query = diesel::sql_query(
                    "INSERT INTO reporting.daily_rollup (date, currency, order_count, total_revenue, units_sold, created_at, updated_at) \
                     VALUES ($1, $2, 1, $3, $4, now(), now()) \
                     ON CONFLICT (tenant_id, date, currency) DO UPDATE \
                        SET order_count = reporting.daily_rollup.order_count + 1, \
                            total_revenue = reporting.daily_rollup.total_revenue + EXCLUDED.total_revenue, \
                            units_sold = reporting.daily_rollup.units_sold + EXCLUDED.units_sold, \
                            updated_at = now();"
                )
                .bind::<Date, _>(input.date)
                .bind::<Text, _>(&input.currency)
                .bind::<Numeric, _>(input.total_revenue)
                .bind::<BigInt, _>(input.units);

                rollup_query
                    .execute(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                // 2. Upsert product rollups
                for item in input.items {
                    let prod_query = diesel::sql_query(
                        "INSERT INTO reporting.daily_product_rollup (date, product_id, title, units_sold, total_revenue, created_at, updated_at) \
                         VALUES ($1, $2, $3, $4, $5, now(), now()) \
                         ON CONFLICT (tenant_id, date, product_id) DO UPDATE \
                            SET units_sold = reporting.daily_product_rollup.units_sold + EXCLUDED.units_sold, \
                                total_revenue = reporting.daily_product_rollup.total_revenue + EXCLUDED.total_revenue, \
                                title = EXCLUDED.title, \
                                updated_at = now();"
                    )
                    .bind::<Date, _>(input.date)
                    .bind::<Text, _>(&item.product_id)
                    .bind::<Text, _>(&item.title)
                    .bind::<BigInt, _>(item.units)
                    .bind::<Numeric, _>(item.total_revenue);

                    prod_query
                        .execute(conn)
                        .await
                        .map_err(|e| Error::Database(e.to_string()))?;
                }

                Ok(())

            })
        })
        .await
    }

    /// Rollup invalidation & adjustments on order edits, cancellations, refunds, returns (P16-04)
    pub async fn record_order_delta(&self, input: RecordOrderDeltaInput) -> Result<()> {
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {


                let query = diesel::sql_query(
                    "INSERT INTO reporting.daily_rollup (date, currency, order_count, total_revenue, units_sold, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, now(), now()) \
                     ON CONFLICT (tenant_id, date, currency) DO UPDATE \
                        SET order_count = GREATEST(0, reporting.daily_rollup.order_count + EXCLUDED.order_count), \
                            total_revenue = GREATEST(0, reporting.daily_rollup.total_revenue + EXCLUDED.total_revenue), \
                            units_sold = GREATEST(0, reporting.daily_rollup.units_sold + EXCLUDED.units_sold), \
                            updated_at = now();"
                )
                .bind::<Date, _>(input.date)
                .bind::<Text, _>(&input.currency)
                .bind::<BigInt, _>(input.delta_orders)
                .bind::<Numeric, _>(input.delta_revenue)
                .bind::<BigInt, _>(input.delta_units);

                query
                    .execute(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())

            })
        })
        .await
    }

    /// Helper calculating effective window lower bound (P16-09)
    fn calculate_window_start(
        requested_from: Option<NaiveDate>,
        window_days: Option<u32>,
    ) -> NaiveDate {
        let today = Utc::now().date_naive();
        if let Some(days) = window_days {
            let window_earliest = today - chrono::Duration::days(days as i64);
            match requested_from {
                Some(from) if from > window_earliest => from,
                _ => window_earliest,
            }
        } else {
            requested_from.unwrap_or(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
        }
    }

    /// Revenue-over-time aggregate (P16-05)
    pub async fn get_revenue_over_time(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        currency: &str,
        window_days: Option<u32>,
    ) -> Result<RevenueOverTimeReport> {
        // Owned before the closure: `with_tenant_conn` moves it into a 'static future.
        let currency = currency.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let effective_from = Self::calculate_window_start(from, window_days);
                let effective_to = to.unwrap_or_else(|| Utc::now().date_naive());

                let rows: Vec<RevenueSeriesRow> = diesel::sql_query(
                    "SELECT date, currency, total_revenue as revenue \
                     FROM reporting.daily_rollup \
                     WHERE date >= $1 AND date <= $2 AND currency = $3 AND deleted_at IS NULL \
                     ORDER BY date ASC;",
                )
                .bind::<Date, _>(effective_from)
                .bind::<Date, _>(effective_to)
                .bind::<Text, _>(&currency)
                .load(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                let mut total_revenue = dec!(0);
                let mut time_series = Vec::with_capacity(rows.len());

                for r in rows {
                    let rev = r.revenue.round_dp(2);
                    total_revenue += rev;
                    time_series.push(RevenueOverTimeItem {
                        date: r.date,
                        currency: r.currency,
                        revenue: rev,
                    });
                }

                Ok(RevenueOverTimeReport {
                    currency,
                    total_revenue: total_revenue.round_dp(2),
                    time_series,
                })
            })
        })
        .await
    }

    /// Orders-over-time aggregate (P16-06)
    pub async fn get_orders_over_time(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        window_days: Option<u32>,
    ) -> Result<OrdersOverTimeReport> {
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let effective_from = Self::calculate_window_start(from, window_days);
                let effective_to = to.unwrap_or_else(|| Utc::now().date_naive());


                let rows: Vec<OrdersSeriesRow> = diesel::sql_query(
                    "SELECT date, SUM(order_count)::bigint as order_count, SUM(units_sold)::bigint as units_sold \
                     FROM reporting.daily_rollup \
                     WHERE date >= $1 AND date <= $2 AND deleted_at IS NULL \
                     GROUP BY date \
                     ORDER BY date ASC;"
                )
                .bind::<Date, _>(effective_from)
                .bind::<Date, _>(effective_to)
                .load(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                let mut total_orders = 0;
                let mut total_units = 0;
                let mut time_series = Vec::with_capacity(rows.len());

                for r in rows {
                    total_orders += r.order_count;
                    total_units += r.units_sold;
                    time_series.push(OrdersOverTimeItem {
                        date: r.date,
                        order_count: r.order_count,
                        units_sold: r.units_sold,
                    });
                }

                Ok(OrdersOverTimeReport {
                    total_orders,
                    total_units,
                    time_series,
                })

            })
        })
        .await
    }

    /// Top-products aggregate (P16-07)
    pub async fn get_top_products(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        limit: i64,
        window_days: Option<u32>,
    ) -> Result<TopProductsReport> {
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {

                let effective_from = Self::calculate_window_start(from, window_days);
                let effective_to = to.unwrap_or_else(|| Utc::now().date_naive());


                let rows: Vec<TopProductRow> = diesel::sql_query(
                    "SELECT product_id, title, SUM(units_sold)::bigint as units_sold, SUM(total_revenue)::numeric as total_revenue \
                     FROM reporting.daily_product_rollup \
                     WHERE date >= $1 AND date <= $2 AND deleted_at IS NULL \
                     GROUP BY product_id, title \
                     ORDER BY total_revenue DESC, units_sold DESC \
                     LIMIT $3;"
                )
                .bind::<Date, _>(effective_from)
                .bind::<Date, _>(effective_to)
                .bind::<BigInt, _>(limit)
                .load(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                let top_products = rows
                    .into_iter()
                    .map(|r| TopProductItem {
                        product_id: r.product_id,
                        title: r.title,
                        units_sold: r.units_sold,
                        total_revenue: r.total_revenue.round_dp(2),
                    })
                    .collect();

                Ok(TopProductsReport { top_products })

            })
        })
        .await
    }

    /// Average-order-value aggregate (P16-08)
    pub async fn get_average_order_value(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        currency: &str,
        window_days: Option<u32>,
    ) -> Result<AverageOrderValueReport> {
        let currency = currency.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let effective_from = Self::calculate_window_start(from, window_days);
                let effective_to = to.unwrap_or_else(|| Utc::now().date_naive());

                let rows: Vec<TotalAovRow> = diesel::sql_query(
                    "SELECT COALESCE(SUM(order_count), 0)::bigint as order_count, \
                            COALESCE(SUM(total_revenue), 0)::numeric as total_revenue \
                     FROM reporting.daily_rollup \
                     WHERE date >= $1 AND date <= $2 AND currency = $3 AND deleted_at IS NULL;",
                )
                .bind::<Date, _>(effective_from)
                .bind::<Date, _>(effective_to)
                .bind::<Text, _>(&currency)
                .load(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                let (order_count, total_revenue) = rows
                    .as_slice()
                    .first()
                    .map(|r| (r.order_count, r.total_revenue))
                    .unwrap_or((0, dec!(0)));

                let total_revenue = total_revenue.round_dp(2);
                let average_order_value = if order_count > 0 {
                    (total_revenue / Decimal::from(order_count)).round_dp(2)
                } else {
                    dec!(0)
                };

                Ok(AverageOrderValueReport {
                    currency,
                    average_order_value,
                    order_count,
                    total_revenue,
                })
            })
        })
        .await
    }

    /// Export revenue report as CSV with UTF-8 BOM (P16-13)
    pub fn export_revenue_csv(&self, report: &RevenueOverTimeReport) -> String {
        let mut out = String::from("\u{FEFF}Date,Currency,Revenue\n");
        for item in &report.time_series {
            out.push_str(&format!(
                "{},{},{}\n",
                item.date, item.currency, item.revenue
            ));
        }
        out
    }

    /// Export orders report as CSV with UTF-8 BOM (P16-13)
    pub fn export_orders_csv(&self, report: &OrdersOverTimeReport) -> String {
        let mut out = String::from("\u{FEFF}Date,Order Count,Units Sold\n");
        for item in &report.time_series {
            out.push_str(&format!(
                "{},{},{}\n",
                item.date, item.order_count, item.units_sold
            ));
        }
        out
    }

    /// Export top products report as CSV with UTF-8 BOM (P16-13)
    pub fn export_top_products_csv(&self, report: &TopProductsReport) -> String {
        let mut out = String::from("\u{FEFF}Product ID,Title,Units Sold,Total Revenue\n");
        for item in &report.top_products {
            out.push_str(&format!(
                "{},\"{}\",{},{}\n",
                item.product_id, item.title, item.units_sold, item.total_revenue
            ));
        }
        out
    }

    /// Export average order value report as CSV with UTF-8 BOM (P16-13)
    pub fn export_aov_csv(&self, report: &AverageOrderValueReport) -> String {
        let mut out =
            String::from("\u{FEFF}Currency,Average Order Value,Order Count,Total Revenue\n");
        out.push_str(&format!(
            "{},{},{},{}\n",
            report.currency, report.average_order_value, report.order_count, report.total_revenue
        ));
        out
    }
}
