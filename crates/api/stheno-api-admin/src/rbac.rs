//! Admin RBAC routes — roles, policies, and the audit trail.
//!
//! `crates/modules/stheno-rbac` has had a complete service since P7-18 (roles,
//! inheritance closure, deny-beats-allow evaluation, sensitive-permission handling,
//! audit records) and was wired to nothing: `config.rs` answered
//! `GET /admin/rbac/roles` with three hardcoded fake roles and `GET /admin/rbac/policies`
//! with an empty array, while `rbac.role`, `rbac.policy`, `rbac.role_inheritance` and
//! `rbac.audit_record` sat unused. The admin's roles and policy screens were therefore
//! reading fiction.
//!
//! These are clean-room screens per ADR-008, so the routes are ours to shape rather than
//! Medusa's to match.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use stheno_api_core::ErrorResponse;
use stheno_rbac::{CreatePolicyInput, CreateRoleInput, EvaluatePermissionInput, RbacService};

#[derive(Clone)]
pub struct RbacContext {
    pub rbac_service: Arc<RbacService>,
}

fn failed(status: StatusCode, code: &'static str, message: String) -> Response {
    (status, Json(ErrorResponse::new(code, &message))).into_response()
}

pub async fn list_roles(State(ctx): State<RbacContext>) -> Result<impl IntoResponse, Response> {
    let roles = ctx.rbac_service.list_roles().await.map_err(|e| {
        failed(
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected_state",
            e.to_string(),
        )
    })?;

    let count = roles.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "roles": roles,
            "data": roles,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

pub async fn get_role(
    State(ctx): State<RbacContext>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let role = ctx
        .rbac_service
        .get_role(&name)
        .await
        .map_err(|e| failed(StatusCode::NOT_FOUND, "not_found", e.to_string()))?;

    Ok((StatusCode::OK, Json(json!({ "role": role, "data": role }))))
}

pub async fn create_role(
    State(ctx): State<RbacContext>,
    Json(input): Json<CreateRoleInput>,
) -> Result<impl IntoResponse, Response> {
    let role = ctx
        .rbac_service
        .create_role(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "role": role, "data": role })),
    ))
}

pub async fn delete_role(
    State(ctx): State<RbacContext>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.rbac_service
        .delete_role(&name)
        .await
        // A system role refuses deletion; that is a 400, not a missing record.
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "not_allowed", e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(json!({ "id": name, "object": "role", "deleted": true })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct PolicyQuery {
    pub role: Option<String>,
}

pub async fn list_policies(
    State(ctx): State<RbacContext>,
    Query(query): Query<PolicyQuery>,
) -> Result<impl IntoResponse, Response> {
    let policies = ctx
        .rbac_service
        .list_policies(query.role.as_deref())
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unexpected_state",
                e.to_string(),
            )
        })?;

    let count = policies.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "policies": policies,
            "data": policies,
            "count": count,
            "offset": 0,
            "limit": count,
        })),
    ))
}

pub async fn create_policy(
    State(ctx): State<RbacContext>,
    Json(input): Json<CreatePolicyInput>,
) -> Result<impl IntoResponse, Response> {
    let policy = ctx
        .rbac_service
        .create_policy(input)
        .await
        .map_err(|e| failed(StatusCode::BAD_REQUEST, "invalid_data", e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "policy": policy, "data": policy })),
    ))
}

pub async fn delete_policy(
    State(ctx): State<RbacContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    ctx.rbac_service
        .delete_policy(&id)
        .await
        .map_err(|e| failed(StatusCode::NOT_FOUND, "not_found", e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(json!({ "id": id, "object": "policy", "deleted": true })),
    ))
}

/// `POST /admin/rbac/evaluate` — answers "may this actor do this?".
///
/// Exposed so the dashboard can grey out actions the caller cannot perform, rather than
/// letting them fail at submit. Deny beats allow, and sensitive permissions require an
/// explicit grant — that logic lives in the service, not here.
pub async fn evaluate(
    State(ctx): State<RbacContext>,
    Json(input): Json<EvaluatePermissionInput>,
) -> Result<impl IntoResponse, Response> {
    let decision = ctx.rbac_service.evaluate(input).await.map_err(|e| {
        failed(
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected_state",
            e.to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "decision": decision, "data": decision })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub actor_id: Option<String>,
}

pub async fn list_audit_records(
    State(ctx): State<RbacContext>,
    Query(query): Query<AuditQuery>,
) -> Result<impl IntoResponse, Response> {
    let records = ctx
        .rbac_service
        .get_audit_records(query.actor_id.as_deref())
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unexpected_state",
                e.to_string(),
            )
        })?;

    let count = records.len();
    Ok((
        StatusCode::OK,
        Json(json!({
            "audit_records": records,
            "data": records,
            "count": count,
        })),
    ))
}

/// POST /admin/rbac/roles/{name}
pub async fn update_role(
    State(ctx): State<RbacContext>,
    Path(name): Path<String>,
    Json(input): Json<stheno_rbac::UpdateRoleInput>,
) -> Result<impl IntoResponse, Response> {
    let role = ctx
        .rbac_service
        .update_role(&name, input)
        .await
        .map_err(rbac_error)?;
    Ok((StatusCode::OK, Json(json!({ "role": role, "data": role }))))
}

/// GET /admin/rbac/policies/{id}
pub async fn get_policy(
    State(ctx): State<RbacContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let policy = ctx.rbac_service.get_policy(&id).await.map_err(rbac_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "policy": policy, "data": policy })),
    ))
}

/// POST /admin/rbac/policies/{id}
pub async fn update_policy(
    State(ctx): State<RbacContext>,
    Path(id): Path<String>,
    Json(input): Json<stheno_rbac::UpdatePolicyInput>,
) -> Result<impl IntoResponse, Response> {
    let policy = ctx
        .rbac_service
        .update_policy(&id, input)
        .await
        .map_err(rbac_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "policy": policy, "data": policy })),
    ))
}

/// Maps a domain error to its status.
///
/// `Invalid` covers the system-role guard and the effect validation, and both are the
/// caller's mistake rather than a missing row — answering 404 for "you may not edit a
/// system role" would tell the admin the role does not exist.
fn rbac_error(e: stheno_types::Error) -> Response {
    let status = match &e {
        stheno_types::Error::NotFound { .. } => StatusCode::NOT_FOUND,
        stheno_types::Error::Invalid { .. } => StatusCode::BAD_REQUEST,
        stheno_types::Error::Conflict { .. } => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let code = match &e {
        stheno_types::Error::NotFound { .. } => "not_found",
        stheno_types::Error::Conflict { .. } => "conflict",
        stheno_types::Error::Invalid { .. } => "invalid_data",
        _ => "unexpected_state",
    };
    (status, Json(ErrorResponse::new(code, &e.to_string()))).into_response()
}
