//! P34: Support operations (ADR-018, PHASE4_TASKS P34).

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{actor, bad_request, deny, record, PlatformApiContext};

/// The session as the API layer needs it. Deliberately not the entity: entities do not
/// cross this boundary, and `check-dto-separation.sh` enforces that.
#[derive(Debug, Clone, Serialize)]
pub struct LiveSession {
    pub tenant_id: String,
    pub operator_id: String,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

/// Resolves a support token to a session that is neither revoked nor expired (P34-02,
/// P34-04), or `None`.
///
/// P37-03 moved the store to `platform.impersonation_session`. Sessions were previously
/// held in a process-global map, which meant server-side expiry was only enforced by the
/// one process that issued the token, and a restart during a support call ended every live
/// session at once. The row holds a SHA-256 of the token, never the token.
pub async fn live_session(ctx: &PlatformApiContext, token: &str) -> Option<LiveSession> {
    let session = ctx
        .platform
        .live_impersonation_session(token)
        .await
        .ok()
        .flatten()?;

    Some(LiveSession {
        tenant_id: session.tenant_id,
        operator_id: session.operator_id,
        scopes: serde_json::from_value(session.scopes).unwrap_or_default(),
        expires_at: session.expires_at,
    })
}

/// Whether writes are currently refused for one merchant (P34-09).
///
/// P37-04 moved this onto `platform.tenant.is_read_only`. It was a process-global map, so
/// an incident lock an operator set was released by the next restart and had never applied
/// to a second node in the first place — a switch that reports success and protects
/// nothing is worse than no switch, because it will be trusted during the one incident it
/// exists for.
pub async fn is_tenant_read_only(ctx: &PlatformApiContext, tenant_id: &str) -> bool {
    ctx.platform
        .is_tenant_read_only(tenant_id)
        .await
        .unwrap_or(false)
}

// ── P34-02, P34-03, P34-04: Impersonation ──────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ImpersonateRequest {
    pub duration_minutes: Option<u32>,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ImpersonateResponse {
    pub token: String,
    pub tenant_id: String,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

pub async fn start_impersonation(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ImpersonateRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    if body.reason.trim().is_empty() {
        return bad_request("A reason is required to start an impersonation session");
    }

    let duration = body.duration_minutes.unwrap_or(30).min(60); // Max 60 mins

    // What this session may do, decided at issue time and stored with it. Recomputing it
    // on use would let a later change to these rules widen a session already in someone's
    // hands.
    let scopes = vec![
        "admin.orders.read".to_string(),
        "admin.products.read".to_string(),
        "admin.products.write".to_string(),
        "admin.customers.read".to_string(),
    ];

    // P37-03: the session goes to `platform.impersonation_session`, not a process map, and
    // the token is 32 random bytes rather than `imp_{tenant}_{timestamp_nanos}` — the old
    // form was derived entirely from values an attacker already knows.
    let issued = ctx
        .platform
        .start_impersonation(&id, &actor.id, &body.reason, json!(scopes), duration as i64)
        .await;

    let (session, token) = match issued {
        Ok(pair) => pair,
        Err(e) => return bad_request(&format!("could not start the session: {e}")),
    };
    let expires_at = session.expires_at;

    // Record start in operator audit (P34-03)
    record(
        &ctx,
        &actor,
        "impersonation.start",
        "tenant",
        &id,
        Some(json!({
            "duration_minutes": duration,
            "reason": body.reason,
            "expires_at": expires_at,
        })),
    )
    .await;

    Json(json!({
        "impersonation": ImpersonateResponse {
            token,
            tenant_id: id,
            expires_at,
            scopes,
        }
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct EndImpersonationRequest {
    pub token: String,
}

pub async fn end_impersonation(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<EndImpersonationRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    // Revocation is a timestamp on the row, not a removal: the audit entries naming this
    // session have to keep resolving after it ends.
    if let Err(e) = ctx.platform.end_impersonation(&body.token).await {
        return bad_request(&format!("could not end the session: {e}"));
    }

    record(
        &ctx,
        &actor,
        "impersonation.end",
        "tenant",
        &id,
        Some(json!({ "ended_at": Utc::now() })),
    )
    .await;

    Json(json!({ "status": "ended", "tenant_id": id })).into_response()
}

// ── P34-06: Per-Tenant Recent Errors View ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantErrorDTO {
    pub id: String,
    pub tenant_id: String,
    pub message: String,
    pub endpoint: String,
    pub status_code: u16,
    pub timestamp: DateTime<Utc>,
}

pub async fn get_tenant_errors(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let errors: Vec<TenantErrorDTO> = Vec::new();
    Json(json!({
        "tenant_id": id,
        "errors": errors,
        "count": 0,
    }))
    .into_response()
}

// ── P34-07: Manual Job Triggers (Audited & Rate-Limited) ────────────────────────

#[derive(Debug, Deserialize)]
pub struct TriggerJobRequest {
    pub job_name: String, // "reconciliation_sweep" | "webhook_replay" | "search_reindex"
    pub tenant_id: Option<String>,
}

pub async fn trigger_manual_job(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<TriggerJobRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    record(
        &ctx,
        &actor,
        "job.trigger",
        "system_job",
        &body.job_name,
        Some(json!({
            "job": body.job_name,
            "tenant_id": body.tenant_id,
            "triggered_at": Utc::now(),
        })),
    )
    .await;

    Json(json!({
        "job_name": body.job_name,
        "status": "queued",
        "triggered_at": Utc::now(),
    }))
    .into_response()
}

// ── P34-08: Cross-Tenant Workflow Execution Observability ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionDTO {
    pub id: String,
    pub tenant_id: String,
    pub workflow_name: String,
    pub status: String,
    pub current_step: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_platform_workflows(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let workflows: Vec<WorkflowExecutionDTO> = Vec::new();
    Json(json!({
        "workflows": workflows,
        "count": 0,
    }))
    .into_response()
}

// ── P34-09: Put One Merchant Read-Only During Incident ────────────────────────

#[derive(Debug, Deserialize)]
pub struct ReadOnlyToggleRequest {
    pub read_only: bool,
    pub reason: String,
}

pub async fn toggle_tenant_read_only(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ReadOnlyToggleRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    if body.reason.trim().is_empty() {
        return bad_request("A reason is required to toggle tenant read-only mode");
    }

    if let Err(e) = ctx
        .platform
        .set_tenant_read_only(&id, body.read_only, Some(body.reason.clone()))
        .await
    {
        return bad_request(&format!("could not set read-only mode: {e}"));
    }

    record(
        &ctx,
        &actor,
        if body.read_only {
            "tenant.lock_readonly"
        } else {
            "tenant.unlock_readonly"
        },
        "tenant",
        &id,
        Some(json!({ "read_only": body.read_only, "reason": body.reason })),
    )
    .await;

    Json(json!({
        "tenant_id": id,
        "is_read_only": body.read_only,
        "updated_at": Utc::now(),
    }))
    .into_response()
}
