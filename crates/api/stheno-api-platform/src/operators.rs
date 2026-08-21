//! Operator credentials and the control-plane audit trail.
//!
//! `/platform/*` was gated on one shared `STHENO_PLATFORM_TOKEN`. That is workable while
//! exactly one person holds it and indefensible the moment a second does: there is no
//! record of who moved a merchant between plans, no way to retire one person's access
//! without retiring everyone's, and no rotation that is not an outage for all of them.
//!
//! The merchant dashboard has had RBAC and MFA since P14. The surface that can read and
//! rewrite *every* merchant's billing had neither, which is the wrong way round.
//!
//! # What this is not
//!
//! It is not RBAC. Every operator can do everything, because a control plane with three
//! holders does not need a permission matrix — it needs to know which of the three acted.
//! Roles are a later problem, and `platform.operator` has room for them.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use stheno_platform::{OperatorAuditDTO, OperatorDTO};

use crate::{actor, bad_request, deny, record, PlatformApiContext};

fn operator_json(op: &OperatorDTO) -> serde_json::Value {
    json!({
        "id": op.id,
        "name": op.name,
        // The only rendering of the credential that ever leaves the database.
        "redacted_token": op.redacted_token,
        "created_at": op.created_at.to_rfc3339(),
        "last_used_at": op.last_used_at.map(|d| d.to_rfc3339()),
        "revoked_at": op.revoked_at.map(|d| d.to_rfc3339()),
        "revoked": op.is_revoked(),
    })
}

fn audit_json(row: &OperatorAuditDTO) -> serde_json::Value {
    json!({
        "id": row.id,
        "actor_id": row.actor_id,
        "actor_name": row.actor_name,
        "action": row.action,
        "entity_type": row.entity_type,
        "entity_id": row.entity_id,
        "diff": row.diff,
        "created_at": row.created_at.to_rfc3339(),
    })
}

/// `GET /platform/operators` — every credential ever issued.
///
/// Revoked ones are included and flagged. A revoked credential that vanished from the
/// list would be indistinguishable from one that was never issued, which is the opposite
/// of what an access review needs to see.
pub async fn list_operators(State(ctx): State<PlatformApiContext>, headers: HeaderMap) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    let Ok(operators) = ctx.platform.list_operators().await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "type": "unexpected_state",
                "message": "could not read the operator registry"
            })),
        )
            .into_response();
    };

    let rows: Vec<_> = operators.iter().map(operator_json).collect();
    Json(json!({
        "operators": rows,
        "count": rows.len(),
        // The caller's own identity, so the UI can mark which row is them and warn before
        // they revoke it. Also the only way a bootstrap-token holder learns they are on a
        // shared secret rather than a named credential.
        "caller": {
            "id": caller.id,
            "name": caller.name,
            "bootstrap": caller.is_bootstrap(),
        },
        "bootstrap_enabled": ctx.operator_token.as_deref().is_some_and(|t| !t.is_empty()),
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct CreateOperatorRequest {
    pub name: String,
}

/// `POST /platform/operators` — issue a named credential.
///
/// The plaintext token is in this response and nowhere else, ever. Only its SHA-256 is
/// stored, so it cannot be recovered from a database read, a backup, or a support
/// request; losing it means issuing a new one and revoking the old.
pub async fn create_operator(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<CreateOperatorRequest>,
) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.create_operator(&body.name).await {
        Ok(issued) => {
            // The token is deliberately absent from the diff. An audit trail that
            // recorded the credentials it audits would be a credential store with worse
            // access control than the credential store.
            record(
                &ctx,
                &caller,
                "operator.create",
                "operator",
                &issued.operator.id,
                Some(json!({
                    "name": issued.operator.name,
                    "redacted_token": issued.operator.redacted_token,
                })),
            )
            .await;

            (
                StatusCode::CREATED,
                Json(json!({
                    "operator": operator_json(&issued.operator),
                    "token": issued.token,
                    "notice": "This token is shown once and cannot be recovered. Store it now.",
                })),
            )
                .into_response()
        }
        Err(e) => bad_request(e),
    }
}

/// `DELETE /platform/operators/{id}` — revoke one credential.
///
/// Revoking is a timestamp, not a delete: the audit rows this operator wrote still name
/// them, and a row that disappeared would leave those pointing at nothing.
pub async fn revoke_operator(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    match ctx.platform.revoke_operator(&id).await {
        Ok(op) => {
            record(
                &ctx,
                &caller,
                "operator.revoke",
                "operator",
                &op.id,
                Some(json!({ "name": op.name, "self_revoked": caller.id == op.id })),
            )
            .await;
            Json(json!({ "operator": operator_json(&op) })).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "type": "not_found", "message": format!("no operator {id}") })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct AuditQuery {
    /// Narrows the trail to one merchant or one plan.
    pub entity_id: Option<String>,
    pub limit: Option<usize>,
    /// The `next_cursor` from the previous page. Absent means start at the newest row.
    pub before: Option<String>,
}

/// Enough to answer "what happened recently" in one call, small enough that the trail is
/// paged rather than dumped.
const DEFAULT_AUDIT_LIMIT: usize = 50;

/// `GET /platform/audit` — one page of control-plane writes, newest first.
///
/// Paged by cursor rather than offset: the trail grows at the end being read, so an
/// offset page 2 would re-show rows page 1 already did. See
/// [`PlatformService::list_operator_audit`].
///
/// Reads across tenants on purpose. This is the query `audit.audit_log` cannot serve:
/// that table is RLS-forced, so "everything this operator changed" is unanswerable there
/// by construction, and half of what happens here (redefining a plan) belongs to no
/// merchant at all.
pub async fn get_audit(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }

    let limit = query.limit.unwrap_or(DEFAULT_AUDIT_LIMIT);
    match ctx
        .platform
        .list_operator_audit(query.entity_id.as_deref(), limit, query.before.as_deref())
        .await
    {
        Ok(rows) => {
            // A cursor only when the page came back full. A short page is the end of the
            // trail, and handing back a cursor there would make the caller fetch an empty
            // page to discover what this response already knew.
            let next_cursor = (rows.len() == limit)
                .then(|| rows.last().map(|r| r.id.clone()))
                .flatten();

            let entries: Vec<_> = rows.iter().map(audit_json).collect();
            Json(json!({
                "entries": entries,
                "count": entries.len(),
                "next_cursor": next_cursor,
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "type": "unexpected_state", "message": e.to_string() })),
        )
            .into_response(),
    }
}
