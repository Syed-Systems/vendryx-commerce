//! `stheno-api-admin-platform`: admin platform surface — feature flags, entitlements,
//! and workflow-execution observability.
//!
//! `GET /admin/feature-flags` is Medusa's own endpoint and shape, so the forked
//! dashboard hides disabled screens unmodified (ENTITLEMENTS.md 7.1).
//! `GET /admin/entitlements` is additive — quotas and grace state have no Medusa
//! equivalent, and the dashboard ignores what it does not read (ADR-002).

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{Map, Value};

use stheno_api_core::{Entitlements, ErrorResponse, QuotaKey, QuotaState, WindowKey};
use stheno_workflow::{WorkflowExecution, WorkflowStore};

/// Current usage per quota, supplied by the caller.
///
/// The engine keeps these in an atomic counter table (ENTITLEMENTS.md 6.2); this API
/// only reports them. A quota with no recorded usage reads as 0 rather than unknown,
/// which is the safe direction: it never invents headroom.
#[derive(Clone, Default)]
pub struct UsageSnapshot {
    used: BTreeMap<QuotaKey, u64>,
}

impl UsageSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: QuotaKey, used: u64) -> Self {
        self.used.insert(key, used);
        self
    }

    pub fn used(&self, key: QuotaKey) -> u64 {
        self.used.get(&key).copied().unwrap_or(0)
    }
}

#[derive(Clone)]
pub struct AdminPlatformContext {
    pub entitlements: Arc<Entitlements>,
    pub usage: Arc<UsageSnapshot>,
    pub workflows: Arc<WorkflowStore>,
}

#[derive(Debug, Serialize)]
pub struct QuotaReport {
    pub key: &'static str,
    /// Absent means unlimited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    pub used: u64,
    pub state: QuotaState,
}

#[derive(Debug, Serialize)]
pub struct WindowReport {
    pub key: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct EntitlementsResponse {
    pub tier: &'static str,
    pub trial_ends_at: Option<String>,
    pub term_ends_at: Option<String>,
    pub features: Vec<&'static str>,
    pub quotas: Vec<QuotaReport>,
    pub windows: Vec<WindowReport>,
}

/// `GET /admin/feature-flags` — Medusa's shape: a flat map of flag name to bool.
pub async fn get_feature_flags(State(ctx): State<AdminPlatformContext>) -> Json<Value> {
    let mut flags = Map::new();
    for (name, enabled) in stheno_api_router_flags(&ctx.entitlements) {
        flags.insert(name.to_string(), Value::Bool(enabled));
    }
    Json(Value::Object(Map::from_iter([(
        "feature_flags".to_string(),
        Value::Object(flags),
    )])))
}

/// Kept local rather than depending on `stheno-api-router`, which depends on this crate.
fn stheno_api_router_flags(ent: &Entitlements) -> Vec<(&'static str, bool)> {
    use stheno_api_core::Feature::*;
    [
        SpreadsheetImport,
        CsvExport,
        WebPush,
        AnalyticsBasic,
        PaymentGateway,
        CourierDispatch,
        FraudScore,
        CustomDomain,
    ]
    .iter()
    .map(|f| (f.as_str(), ent.has(*f)))
    .collect()
}

/// `GET /admin/entitlements` — additive endpoint reporting tier, quotas and grace state.
pub async fn get_entitlements(
    State(ctx): State<AdminPlatformContext>,
) -> Json<EntitlementsResponse> {
    let ent = &ctx.entitlements;

    let quotas = [QuotaKey::MaxProducts, QuotaKey::MaxStaffSeats]
        .into_iter()
        .map(|key| {
            let used = ctx.usage.used(key);
            QuotaReport {
                key: key.as_str(),
                limit: ent.quota(key),
                used,
                state: ent.quota_state(key, used),
            }
        })
        .collect();

    let windows = [WindowKey::AnalyticsWindowDays]
        .into_iter()
        .map(|key| WindowReport {
            key: key.as_str(),
            limit: ent.window(key),
        })
        .collect();

    Json(EntitlementsResponse {
        tier: ent.tier().as_str(),
        trial_ends_at: ent.trial_ends_at().map(|d| d.to_rfc3339()),
        term_ends_at: ent.term_ends_at().map(|d| d.to_rfc3339()),
        features: ent.features().map(|f| f.as_str()).collect(),
        quotas,
        windows,
    })
}

#[derive(Debug, Serialize)]
pub struct ListWorkflowExecutionsResponse {
    pub workflow_executions: Vec<WorkflowExecution>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct GetWorkflowExecutionResponse {
    pub workflow_execution: WorkflowExecution,
}

/// `GET /admin/workflows-executions` — SAGA observability for the dashboard.
pub async fn list_workflow_executions(
    State(ctx): State<AdminPlatformContext>,
) -> Json<ListWorkflowExecutionsResponse> {
    let executions = ctx.workflows.list_executions();
    let count = executions.len();
    Json(ListWorkflowExecutionsResponse {
        workflow_executions: executions,
        count,
        offset: 0,
        limit: 50,
    })
}

pub async fn get_workflow_execution(
    State(ctx): State<AdminPlatformContext>,
    Path(id): Path<String>,
) -> Result<Json<GetWorkflowExecutionResponse>, Response> {
    match ctx.workflows.get_execution(&id) {
        Some(workflow_execution) => Ok(Json(GetWorkflowExecutionResponse { workflow_execution })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "not_found",
                &format!("Workflow execution with id {id} was not found"),
            )),
        )
            .into_response()),
    }
}

/// Mounts the admin platform routes. These are never gated: the dashboard needs to read
/// its own entitlements in order to hide the screens it is not allowed to show.
pub fn admin_platform_routes(ctx: AdminPlatformContext) -> Router {
    Router::new()
        .route("/admin/feature-flags", get(get_feature_flags))
        .route("/admin/entitlements", get(get_entitlements))
        .route("/admin/workflows-executions", get(list_workflow_executions))
        .route(
            "/admin/workflows-executions/{id}",
            get(get_workflow_execution),
        )
        .with_state(ctx)
}
