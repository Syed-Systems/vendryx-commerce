//! Admin Reconciliation API Handlers (PHASE3_TASKS §P24-10).
//! Manages courier settlement cycles, lines, payout imports, and reconciliation runs.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use stheno_api_core::ErrorResponse;
use stheno_payment::{ImportSettlementReportInput, PaymentService};

#[derive(Clone)]
pub struct AdminReconciliationContext {
    pub payment_service: Arc<PaymentService>,
}

#[derive(Debug, Deserialize)]
pub struct ListCyclesQuery {
    pub provider_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub cycle_days: Option<i64>,
}

pub async fn list_cycles(
    State(ctx): State<AdminReconciliationContext>,
    Query(query): Query<ListCyclesQuery>,
) -> Result<impl IntoResponse, Response> {
    let cycles = ctx
        .payment_service
        .list_settlement_cycles(query.provider_id.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("reconciliation_error", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(json!({ "cycles": cycles }))))
}

pub async fn get_cycle(
    State(ctx): State<AdminReconciliationContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let cycle = ctx
        .payment_service
        .get_settlement_cycle(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("cycle_not_found", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(json!({ "cycle": cycle }))))
}

pub async fn list_cycle_lines(
    State(ctx): State<AdminReconciliationContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let lines = ctx
        .payment_service
        .list_settlement_lines(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("reconciliation_error", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(json!({ "lines": lines }))))
}

pub async fn import_settlement(
    State(ctx): State<AdminReconciliationContext>,
    Json(input): Json<ImportSettlementReportInput>,
) -> Result<impl IntoResponse, Response> {
    let output = ctx
        .payment_service
        .import_settlement_report(input)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("import_error", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((StatusCode::CREATED, Json(json!({ "import": output }))))
}

pub async fn run_reconciliation(
    State(ctx): State<AdminReconciliationContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let report = ctx
        .payment_service
        .run_courier_reconciliation(&id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("reconciliation_failed", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(json!({ "report": report }))))
}

pub async fn check_alerts(
    State(ctx): State<AdminReconciliationContext>,
    Path(id): Path<String>,
    Query(query): Query<AlertsQuery>,
) -> Result<impl IntoResponse, Response> {
    let days = query.cycle_days.unwrap_or(7);
    let alerts = ctx
        .payment_service
        .check_unsettled_line_alerts(&id, days)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("alerts_failed", &e.to_string())),
            )
                .into_response()
        })?;

    Ok((StatusCode::OK, Json(json!({ "alerts": alerts }))))
}

pub fn admin_reconciliation_routes(payment_service: Arc<PaymentService>) -> Router {
    let ctx = AdminReconciliationContext { payment_service };
    Router::new()
        .route("/admin/reconciliation/cycles", get(list_cycles))
        .route("/admin/reconciliation/cycles/{id}", get(get_cycle))
        .route(
            "/admin/reconciliation/cycles/{id}/lines",
            get(list_cycle_lines),
        )
        .route(
            "/admin/reconciliation/cycles/{id}/reconcile",
            post(run_reconciliation),
        )
        .route(
            "/admin/reconciliation/cycles/{id}/alerts",
            get(check_alerts),
        )
        .route("/admin/reconciliation/import", post(import_settlement))
        // Aliases for convenience
        .route("/admin/courier-settlements", get(list_cycles))
        .route("/admin/courier-settlements/{id}", get(get_cycle))
        .route(
            "/admin/courier-settlements/{id}/lines",
            get(list_cycle_lines),
        )
        .route(
            "/admin/courier-settlements/{id}/reconcile",
            post(run_reconciliation),
        )
        .route("/admin/courier-settlements/{id}/alerts", get(check_alerts))
        .route("/admin/courier-settlements/import", post(import_settlement))
        .with_state(ctx)
}
