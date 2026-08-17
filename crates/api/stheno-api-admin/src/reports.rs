//! Admin reporting endpoints with Medusa envelope format and CSV export (P16-12, P16-13).

use axum::extract::{Query, State};
use axum::http::header::{HeaderMap, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_reporting::ReportingService;

#[derive(Clone)]
pub struct AdminReportingContext {
    pub reporting_service: Arc<ReportingService>,
}

#[derive(Debug, Deserialize)]
pub struct ReportQueryParams {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub currency: Option<String>,
    pub limit: Option<i64>,
    pub window_days: Option<u32>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportEnvelope<T> {
    pub report: ReportBody<T>,
}

#[derive(Debug, Serialize)]
pub struct ReportBody<T> {
    pub report_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_days: Option<u32>,
    pub data: T,
}

pub async fn revenue_over_time_handler(
    State(ctx): State<AdminReportingContext>,
    headers: HeaderMap,
    Query(params): Query<ReportQueryParams>,
) -> Response {
    let currency = params.currency.as_deref().unwrap_or("USD");
    match ctx
        .reporting_service
        .get_revenue_over_time(params.from, params.to, currency, params.window_days)
        .await
    {
        Ok(data) => {
            let is_csv = params.format.as_deref() == Some("csv")
                || headers
                    .get("accept")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("text/csv"))
                    .unwrap_or(false);

            if is_csv {
                let csv_content = ctx.reporting_service.export_revenue_csv(&data);
                (
                    StatusCode::OK,
                    [(CONTENT_TYPE, "text/csv; charset=utf-8")],
                    csv_content,
                )
                    .into_response()
            } else {
                Json(ReportEnvelope {
                    report: ReportBody {
                        report_type: "revenue_over_time".to_string(),
                        window_days: params.window_days,
                        data,
                    },
                })
                .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn orders_over_time_handler(
    State(ctx): State<AdminReportingContext>,
    headers: HeaderMap,
    Query(params): Query<ReportQueryParams>,
) -> Response {
    match ctx
        .reporting_service
        .get_orders_over_time(params.from, params.to, params.window_days)
        .await
    {
        Ok(data) => {
            let is_csv = params.format.as_deref() == Some("csv")
                || headers
                    .get("accept")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("text/csv"))
                    .unwrap_or(false);

            if is_csv {
                let csv_content = ctx.reporting_service.export_orders_csv(&data);
                (
                    StatusCode::OK,
                    [(CONTENT_TYPE, "text/csv; charset=utf-8")],
                    csv_content,
                )
                    .into_response()
            } else {
                Json(ReportEnvelope {
                    report: ReportBody {
                        report_type: "orders_over_time".to_string(),
                        window_days: params.window_days,
                        data,
                    },
                })
                .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn top_products_handler(
    State(ctx): State<AdminReportingContext>,
    headers: HeaderMap,
    Query(params): Query<ReportQueryParams>,
) -> Response {
    let limit = params.limit.unwrap_or(10);
    match ctx
        .reporting_service
        .get_top_products(params.from, params.to, limit, params.window_days)
        .await
    {
        Ok(data) => {
            let is_csv = params.format.as_deref() == Some("csv")
                || headers
                    .get("accept")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("text/csv"))
                    .unwrap_or(false);

            if is_csv {
                let csv_content = ctx.reporting_service.export_top_products_csv(&data);
                (
                    StatusCode::OK,
                    [(CONTENT_TYPE, "text/csv; charset=utf-8")],
                    csv_content,
                )
                    .into_response()
            } else {
                Json(ReportEnvelope {
                    report: ReportBody {
                        report_type: "top_products".to_string(),
                        window_days: params.window_days,
                        data,
                    },
                })
                .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn average_order_value_handler(
    State(ctx): State<AdminReportingContext>,
    headers: HeaderMap,
    Query(params): Query<ReportQueryParams>,
) -> Response {
    let currency = params.currency.as_deref().unwrap_or("USD");
    match ctx
        .reporting_service
        .get_average_order_value(params.from, params.to, currency, params.window_days)
        .await
    {
        Ok(data) => {
            let is_csv = params.format.as_deref() == Some("csv")
                || headers
                    .get("accept")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("text/csv"))
                    .unwrap_or(false);

            if is_csv {
                let csv_content = ctx.reporting_service.export_aov_csv(&data);
                (
                    StatusCode::OK,
                    [(CONTENT_TYPE, "text/csv; charset=utf-8")],
                    csv_content,
                )
                    .into_response()
            } else {
                Json(ReportEnvelope {
                    report: ReportBody {
                        report_type: "average_order_value".to_string(),
                        window_days: params.window_days,
                        data,
                    },
                })
                .into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
