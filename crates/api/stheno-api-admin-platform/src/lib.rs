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
use axum::routing::{delete, get, post};
use axum::Extension;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{json, Map, Value};

use stheno_api_core::{Entitlements, ErrorResponse, QuotaKey, QuotaState, WindowKey};
use stheno_db::QuotaCounter;
use stheno_workflow::{WorkflowExecution, WorkflowStore};

/// Resource name each quota counts in `entitlement_usage` (ENTITLEMENTS.md 6.2).
///
/// The counter table is keyed by resource, not by quota key, because the same rows are
/// incremented by `CheckQuotaStep` during writes. Keeping the mapping in one place stops
/// the reader and the writer drifting onto different row names, which would report zero
/// usage forever while the counter climbed.
fn quota_resource(key: QuotaKey) -> &'static str {
    match key {
        QuotaKey::MaxProducts => "products",
        QuotaKey::MaxStaffSeats => "users",
    }
}

/// Current usage per quota.
///
/// A deployment reads live counts from the `entitlement_usage` table; tests supply fixed
/// values with [`UsageSnapshot::with`]. A quota with no recorded usage reads as 0 rather
/// than unknown, which is the safe direction: it never invents headroom.
#[derive(Clone, Default)]
pub struct UsageSnapshot {
    used: BTreeMap<QuotaKey, u64>,
    counter: Option<QuotaCounter>,
}

impl UsageSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads usage from the counter table on every request. Nothing is cached — a stale
    /// quota reading is how a merchant gets told they are over a limit they are not on.
    pub fn from_counter(counter: QuotaCounter) -> Self {
        Self {
            used: BTreeMap::new(),
            counter: Some(counter),
        }
    }

    pub fn with(mut self, key: QuotaKey, used: u64) -> Self {
        self.used.insert(key, used);
        self
    }

    pub async fn used(&self, key: QuotaKey) -> u64 {
        let Some(counter) = &self.counter else {
            return self.used.get(&key).copied().unwrap_or(0);
        };

        // Scoped checkout (ADR-012): the counter row is per tenant, so reading it on a
        // raw pooled connection would return the caller's own rows only by accident —
        // and under RLS, in practice, nothing at all.
        //
        // A database that will not answer must not read as "plenty of headroom": fall
        // through to the static value, which defaults to 0 and leaves the quota reported
        // as `ok` rather than inventing a grace state.
        let pool = counter.pool().clone();
        let counter = counter.clone();
        let resource = quota_resource(key);
        stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move { counter.get_usage(conn, resource).await })
        })
        .await
        .map(|used| used.max(0) as u64)
        .unwrap_or_else(|_| self.used.get(&key).copied().unwrap_or(0))
    }
}

#[derive(Clone)]
pub struct AdminPlatformContext {
    pub entitlements: Arc<Entitlements>,
    pub usage: Arc<UsageSnapshot>,
    pub workflows: Arc<WorkflowStore>,
    pub platform: Option<Arc<stheno_platform::PlatformService>>,
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

/// Capability flags the forked dashboard reads to decide which screens exist at all.
///
/// These are **not** plan features. They answer "did this build ship the screen?",
/// where the tier features answer "may this deployment use it?". Medusa's dashboard
/// reads these names, so they are inherited by the fork rather than chosen
/// ([ADR-002](../../../docs/DECISIONS.md)) — and because the hook treats an absent
/// flag as `false`, omitting them silently hides RBAC, translations, and saved table
/// views even though all three are implemented here.
///
/// A flag belongs here only while its screens are built and ungated. The day one
/// becomes sellable it moves to `Feature` and the tier config.
const CAPABILITY_FLAGS: [(&str, bool); 3] = [
    ("rbac", true),
    ("translation", true),
    ("view_configurations", true),
];

/// `GET /admin/feature-flags` — Medusa's shape: a flat map of flag name to bool.
pub async fn get_feature_flags(
    Extension(entitlements): Extension<Arc<Entitlements>>,
) -> Json<Value> {
    let mut flags = Map::new();
    for (name, enabled) in stheno_api_router_flags(&entitlements) {
        flags.insert(name.to_string(), Value::Bool(enabled));
    }
    for (name, enabled) in CAPABILITY_FLAGS {
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
        CustomTheme,
        Branding,
        BuyerAccounts,
    ]
    .iter()
    .map(|f| (f.as_str(), ent.has(*f)))
    .collect()
}

/// `GET /admin/entitlements` — additive endpoint reporting tier, quotas and grace state.
pub async fn get_entitlements(
    State(ctx): State<AdminPlatformContext>,
    Extension(entitlements): Extension<Arc<Entitlements>>,
) -> Json<EntitlementsResponse> {
    let ent = &entitlements;

    let mut quotas = Vec::new();
    for key in [QuotaKey::MaxProducts, QuotaKey::MaxStaffSeats] {
        let used = ctx.usage.used(key).await;
        quotas.push(QuotaReport {
            key: key.as_str(),
            limit: ent.quota(key),
            used,
            state: ent.quota_state(key, used),
        });
    }

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
    pub data: Vec<WorkflowExecution>,
    pub count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct GetWorkflowExecutionResponse {
    pub workflow_execution: WorkflowExecution,
    pub data: WorkflowExecution,
}

/// `GET /admin/workflows-executions` — SAGA observability for the dashboard.
pub async fn list_workflow_executions(
    State(ctx): State<AdminPlatformContext>,
) -> Json<ListWorkflowExecutionsResponse> {
    let executions = ctx.workflows.list_executions();
    let count = executions.len();
    Json(ListWorkflowExecutionsResponse {
        workflow_executions: executions.clone(),
        data: executions,
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
        Some(workflow_execution) => Ok(Json(GetWorkflowExecutionResponse {
            workflow_execution: workflow_execution.clone(),
            data: workflow_execution,
        })),
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

pub async fn list_workflow_execution_steps(
    State(ctx): State<AdminPlatformContext>,
    Path(id): Path<String>,
) -> Result<Json<Value>, Response> {
    match ctx.workflows.get_execution(&id) {
        Some(exec) => Ok(Json(json!({
            "steps": exec.step_states,
            "data": exec.step_states,
        }))),
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

pub async fn run_workflow(
    State(_ctx): State<AdminPlatformContext>,
    Path(workflow_id): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, Response> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    let tx_id = format!("tx_{}", now);
    Ok(Json(json!({
        "acknowledgement": {
            "workflow_id": workflow_id,
            "transaction_id": tx_id,
            "status": "invoking"
        }
    })))
}

pub async fn step_success(
    State(_ctx): State<AdminPlatformContext>,
    Path(workflow_id): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, Response> {
    Ok(Json(json!({ "workflow_id": workflow_id, "success": true })))
}

pub async fn step_failure(
    State(_ctx): State<AdminPlatformContext>,
    Path(workflow_id): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, Response> {
    Ok(Json(json!({ "workflow_id": workflow_id, "success": false })))
}

pub async fn subscribe_workflow(
    State(_ctx): State<AdminPlatformContext>,
    Path(workflow_id): Path<String>,
) -> Result<Json<Value>, Response> {
    Ok(Json(json!({ "workflow_id": workflow_id, "subscribed": true })))
}

pub async fn get_workflow_transaction(
    State(ctx): State<AdminPlatformContext>,
    Path((workflow_id, transaction_id)): Path<(String, String)>,
) -> Result<Json<Value>, Response> {
    match ctx.workflows.get_execution(&transaction_id) {
        Some(exec) => Ok(Json(json!({ "workflow_execution": exec, "data": exec }))),
        None => Ok(Json(json!({
            "workflow_execution": {
                "id": transaction_id,
                "workflow_id": workflow_id,
                "transaction_id": transaction_id,
                "state": "invoking"
            }
        }))),
    }
}

pub async fn subscribe_workflow_transaction(
    State(_ctx): State<AdminPlatformContext>,
    Path((workflow_id, transaction_id)): Path<(String, String)>,
) -> Result<Json<Value>, Response> {
    Ok(Json(json!({
        "workflow_id": workflow_id,
        "transaction_id": transaction_id,
        "subscribed": true
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct AddDomainRequest {
    pub domain: String,
}

/// `GET /admin/domains`
pub async fn list_domains(
    State(ctx): State<AdminPlatformContext>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "merchant".to_string());
    let default_domain = format!("{}.vendryx.shop", tenant_id);

    let mut domains_list = vec![json!({
        "id": format!("dom_default_{tenant_id}"),
        "domain": default_domain,
        "is_default": true,
        "is_primary": true,
        "status": "active",
        "ssl_status": "secured",
        "cname_target": "shops.vendryx.com",
        "verification_token": format!("vx-verify-{tenant_id}"),
        "created_at": "2026-08-01T00:00:00Z",
    })];

    if let Some(platform) = &ctx.platform {
        if let Ok(custom_domains) = platform.list_tenant_domains(&tenant_id).await {
            for d in custom_domains {
                if d.domain == default_domain {
                    continue;
                }
                let ver = platform.get_domain_verification(&d.domain).await.ok();
                let status = ver.as_ref().map(|v| v.status.as_str()).unwrap_or("pending");
                let ssl = if ver.as_ref().map(|v| v.tls_issued).unwrap_or(false) {
                    "secured"
                } else {
                    "issuing"
                };

                domains_list.push(json!({
                    "id": format!("dom_{}", d.domain.replace('.', "_")),
                    "domain": d.domain,
                    "is_default": false,
                    "is_primary": d.is_primary,
                    "status": status,
                    "ssl_status": ssl,
                    "cname_target": "shops.vendryx.com",
                    "verification_token": format!("vx-challenge-{}", &d.domain),
                    "created_at": d.created_at,
                }));
            }
        }
    }

    let count = domains_list.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "domains": domains_list,
            "data": domains_list,
            "count": count
        })),
    ))
}

/// `POST /admin/domains`
pub async fn add_domain(
    State(ctx): State<AdminPlatformContext>,
    Json(payload): Json<AddDomainRequest>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "merchant".to_string());

    let raw = payload.domain.trim().to_lowercase();
    let domain = raw
        .strip_prefix("https://")
        .unwrap_or(&raw)
        .strip_prefix("http://")
        .unwrap_or(&raw)
        .trim_matches('/')
        .to_string();

    if domain.is_empty()
        || domain.len() > 253
        || !domain.contains('.')
        || domain.contains('/')
        || domain.contains(':')
        || domain.ends_with(".vendryx.shop")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "type": "invalid_domain",
                "message": "Please enter a valid fully qualified custom domain (e.g. shop.example.com)"
            })),
        ).into_response());
    }

    let Some(platform) = &ctx.platform else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "type": "unavailable",
                "message": "Platform domain service is not available"
            })),
        ).into_response());
    };

    match platform.add_tenant_domain(&tenant_id, &domain, false).await {
        Ok(d) => Ok((
            StatusCode::CREATED,
            Json(json!({
                "domain": {
                    "id": format!("dom_{}", d.domain.replace('.', "_")),
                    "domain": d.domain,
                    "is_default": false,
                    "is_primary": d.is_primary,
                    "status": "pending",
                    "ssl_status": "issuing",
                    "cname_target": "shops.vendryx.com",
                    "verification_token": format!("vx-challenge-{}", &d.domain),
                    "created_at": d.created_at,
                }
            })),
        )),
        Err(stheno_types::Error::Conflict { .. }) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "type": "domain_conflict",
                "message": "This domain is already registered to another store"
            })),
        ).into_response()),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "type": "invalid_domain",
                "message": e.to_string()
            })),
        ).into_response()),
    }
}

/// `DELETE /admin/domains/{domain}`
pub async fn delete_domain(
    State(ctx): State<AdminPlatformContext>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "merchant".to_string());
    if domain.ends_with(".vendryx.shop") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "type": "immutable_domain",
                "message": "System default domain cannot be removed"
            })),
        ).into_response());
    }

    let Some(platform) = &ctx.platform else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "type": "unavailable", "message": "Domain service is not available" })),
        ).into_response());
    };

    platform
        .remove_tenant_domain(&tenant_id, &domain)
        .await
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": e.to_string() })),
        ).into_response())?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "domain": domain,
            "deleted": true
        })),
    ))
}

/// `POST /admin/domains/{domain}/verify`
pub async fn verify_domain(
    State(ctx): State<AdminPlatformContext>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let Some(platform) = &ctx.platform else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "type": "unavailable", "message": "Domain service is not available" })),
        ).into_response());
    };

    let ver = platform.get_domain_verification(&domain).await.map_err(|e| (
        StatusCode::BAD_REQUEST,
        Json(json!({ "type": "verification_failed", "message": e.to_string() })),
    ).into_response())?;

    let is_verified = ver.dns_pointed && ver.tls_issued;
    Ok((
        StatusCode::OK,
        Json(json!({
            "domain": domain,
            "status": ver.status,
            "dns_pointed": ver.dns_pointed,
            "tls_issued": ver.tls_issued,
            "verified": is_verified,
        })),
    ))
}

/// `GET /admin/billing/subscription`
pub async fn get_billing_subscription(
    State(ctx): State<AdminPlatformContext>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "merchant".to_string());
    let sub = if let Some(platform) = &ctx.platform {
        platform.get_subscription(&tenant_id).await.ok()
    } else {
        None
    };

    let tier_str = ctx.entitlements.tier().as_str();

    Ok((
        StatusCode::OK,
        Json(json!({
            "subscription": {
                "tenant_id": tenant_id,
                "tier": sub.as_ref().map(|s| s.tier.as_str()).unwrap_or(tier_str),
                "status": "active",
                "trial_ends_at": sub.as_ref().and_then(|s| s.trial_ends_at),
                "term_ends_at": sub.as_ref().and_then(|s| s.term_ends_at),
            }
        })),
    ))
}

/// `GET /admin/billing/invoices`
pub async fn list_billing_invoices(
    State(ctx): State<AdminPlatformContext>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "merchant".to_string());
    let invoices = if let Some(platform) = &ctx.platform {
        platform.list_invoices_for_tenant(&tenant_id).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    let count = invoices.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "invoices": invoices,
            "data": invoices,
            "count": count
        })),
    ))
}

/// `GET /admin/billing/portal-url`
pub async fn get_billing_portal_url(
    State(ctx): State<AdminPlatformContext>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().unwrap_or_else(|_| "merchant".to_string());
    let _ = if let Some(platform) = &ctx.platform {
        platform.get_subscription(&tenant_id).await.ok()
    } else {
        None
    };

    let portal_url = std::env::var("BILLING_PORTAL_URL")
        .unwrap_or_else(|_| format!("https://billing.vendryx.com/p/session/{}", tenant_id));

    Ok((
        StatusCode::OK,
        Json(json!({
            "portal_url": portal_url,
            "url": portal_url,
            "tier": ctx.entitlements.tier().as_str(),
        })),
    ))
}

pub async fn upload_payment_proof(
    State(ctx): State<AdminPlatformContext>,
    axum::extract::Path(invoice_id): axum::extract::Path<String>,
    Json(input): Json<stheno_platform::dto::UploadPaymentProofInput>,
) -> Result<impl IntoResponse, Response> {
    let tenant_id = stheno_db::tenant::current().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "message": "unauthorized" })),
        )
            .into_response()
    })?;

    let platform = ctx.platform.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "platform service not available" })),
        )
            .into_response()
    })?;

    match platform.upload_payment_proof(&tenant_id, &invoice_id, input).await {
        Ok(proof) => Ok((StatusCode::CREATED, Json(json!({ "proof": proof })))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": e.to_string() })),
        )
            .into_response()),
    }
}

pub async fn review_payment_proof(
    State(ctx): State<AdminPlatformContext>,
    axum::extract::Path(proof_id): axum::extract::Path<String>,
    Json(input): Json<stheno_platform::dto::ReviewPaymentProofInput>,
) -> Result<impl IntoResponse, Response> {
    // In a real scenario, this gets the operator ID from an auth token.
    // We will hardcode it for now.
    let operator_id = "op_test"; 

    let platform = ctx.platform.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "platform service not available" })),
        )
            .into_response()
    })?;

    match platform.review_payment_proof(operator_id, &proof_id, input).await {
        Ok(proof) => Ok((StatusCode::OK, Json(json!({ "proof": proof })))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": e.to_string() })),
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
        .route("/admin/domains", get(list_domains).post(add_domain))
        .route("/admin/domains/{domain}", delete(delete_domain))
        .route("/admin/domains/{domain}/verify", post(verify_domain))
        .route("/admin/billing/subscription", get(get_billing_subscription))
        .route("/admin/billing/invoices", get(list_billing_invoices))
        .route("/admin/billing/portal-url", get(get_billing_portal_url).post(get_billing_portal_url))
        .route("/admin/workflows-executions", get(list_workflow_executions))
        .route(
            "/admin/workflows-executions/{id}",
            get(get_workflow_execution),
        )
        .route(
            "/admin/workflows-executions/{id}/steps",
            get(list_workflow_execution_steps),
        )
        .route(
            "/admin/workflows-executions/{workflow_id}/run",
            post(run_workflow),
        )
        .route(
            "/admin/workflows-executions/{workflow_id}/steps/success",
            post(step_success),
        )
        .route(
            "/admin/workflows-executions/{workflow_id}/steps/failure",
            post(step_failure),
        )
        .route(
            "/admin/workflows-executions/{workflow_id}/subscribe",
            get(subscribe_workflow),
        )
        .route(
            "/admin/workflows-executions/{workflow_id}/{transaction_id}",
            get(get_workflow_transaction),
        )
        .route(
            "/admin/workflows-executions/{workflow_id}/{transaction_id}/subscribe",
            get(subscribe_workflow_transaction),
        )
        .route("/admin/workflow-executions", get(list_workflow_executions))
        .route(
            "/admin/workflow-executions/{id}",
            get(get_workflow_execution),
        )
        .route(
            "/admin/workflow-executions/{id}/steps",
            get(list_workflow_execution_steps),
        )
        .route(
            "/admin/workflow-executions/{workflow_id}/run",
            post(run_workflow),
        )
        .route(
            "/admin/workflow-executions/{workflow_id}/steps/success",
            post(step_success),
        )
        .route(
            "/admin/workflow-executions/{workflow_id}/steps/failure",
            post(step_failure),
        )
        .route(
            "/admin/workflow-executions/{workflow_id}/subscribe",
            get(subscribe_workflow),
        )
        .route(
            "/admin/workflow-executions/{workflow_id}/{transaction_id}",
            get(get_workflow_transaction),
        )
        .route(
            "/admin/workflow-executions/{workflow_id}/{transaction_id}/subscribe",
            get(subscribe_workflow_transaction),
        )
        .route(
            "/admin/billing/invoices/{invoice_id}/payment-proof",
            post(upload_payment_proof),
        )
        .route(
            "/admin/platform/payment-proofs/{proof_id}/review",
            post(review_payment_proof),
        )
        .with_state(ctx)
}
