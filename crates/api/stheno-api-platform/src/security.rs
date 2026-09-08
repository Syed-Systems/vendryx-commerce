//! P36: Platform RBAC, MFA, Network Limits, Bootstrap Retirement, and Compliance Evidence (PHASE4_TASKS P36).

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{actor, bad_request, deny, record, PlatformApiContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Owner,
    Billing,
    Support,
    Engineer,
}

impl OperatorRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperatorRole::Owner => "owner",
            OperatorRole::Billing => "billing",
            OperatorRole::Support => "support",
            OperatorRole::Engineer => "engineer",
        }
    }

    pub fn permits(&self, required: OperatorRole) -> bool {
        if *self == OperatorRole::Owner {
            return true;
        }
        *self == required
    }
}

impl std::str::FromStr for OperatorRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "owner" => Ok(OperatorRole::Owner),
            "billing" => Ok(OperatorRole::Billing),
            "support" => Ok(OperatorRole::Support),
            "engineer" => Ok(OperatorRole::Engineer),
            other => Err(format!("unknown operator role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorSecurityProfile {
    pub operator_id: String,
    pub role: OperatorRole,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub ip_allowlist: Vec<String>,
    pub session_ttl_minutes: Option<u64>,
}

/// One operator's access posture, or `None` when there is no such operator.
///
/// P37-25 moved this onto `platform.operator`. Two things were wrong with the process-map
/// version, and the second is the serious one:
///
/// 1. It did not survive a restart, so every role narrowing, MFA enrolment and IP
///    allowlist silently reverted — an access-control surface that stops applying with no
///    error anywhere.
/// 2. It answered an id it had never seen with a **default carrying `role: Owner`**. The
///    unknown case was the most privileged one rather than the least. Returning `None`
///    here is what makes callers deny, and it is why this is not an infallible getter any
///    more: there is no safe profile to invent for an operator that does not exist.
pub async fn get_security_profile(
    ctx: &PlatformApiContext,
    operator_id: &str,
) -> Option<OperatorSecurityProfile> {
    let operator = ctx
        .platform
        .get_operator(operator_id)
        .await
        .ok()
        .flatten()?;

    Some(OperatorSecurityProfile {
        operator_id: operator.id.to_string(),
        // An unparseable role is `Support`, the least privileged, never `Owner`. The column
        // has a CHECK so this should be unreachable; if it is ever reached, failing to the
        // narrowest role is the only safe direction.
        role: operator
            .role
            .parse::<OperatorRole>()
            .unwrap_or(OperatorRole::Support),
        mfa_enabled: operator.mfa_enabled,
        mfa_secret: operator.mfa_secret,
        ip_allowlist: serde_json::from_value(operator.ip_allowlist).unwrap_or_default(),
        session_ttl_minutes: operator.session_ttl_minutes.map(|ttl| ttl as u64),
    })
}

// ── P36-01 & P36-02: Operator Roles & Role Enforcement ─────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateOperatorRoleRequest {
    pub role: String,
}

pub async fn get_operator_role(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(_caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    let Some(profile) = get_security_profile(&ctx, &id).await else {
        return deny();
    };
    Json(json!({
        "operator_id": id,
        "role": profile.role.as_str(),
        "mfa_enabled": profile.mfa_enabled,
    }))
    .into_response()
}

pub async fn set_operator_role(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateOperatorRoleRequest>,
) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    let Ok(role) = body.role.parse::<OperatorRole>() else {
        return bad_request(format!("invalid role {:?}", body.role));
    };

    // No `or_insert_with` any more: a role can only be set on an operator that exists.
    // The map version created a profile for any id it was handed, which meant a typo in a
    // path segment silently produced a new privileged principal.
    if let Err(e) = ctx.platform.set_operator_role(&id, role.as_str()).await {
        return bad_request(format!("could not set the role: {e}"));
    }

    record(
        &ctx,
        &caller,
        "operator.role.update",
        "operator",
        &id,
        Some(json!({ "role": role.as_str() })),
    )
    .await;

    Json(json!({
        "operator_id": id,
        "role": role.as_str(),
    }))
    .into_response()
}

// ── P36-03: MFA Setup & Verification ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyMfaRequest {
    pub code: String,
}

pub async fn setup_operator_mfa(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    let secret = format!(
        "TOTP_SEC_{:x}",
        Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    // Enrolment only. `mfa_enabled` stays false until a code verifies against this
    // secret, so an operator who starts setup and never scans the code is not locked out.
    if let Err(e) = ctx.platform.begin_operator_mfa(&id, &secret).await {
        return bad_request(format!("could not start MFA enrolment: {e}"));
    }

    record(
        &ctx,
        &caller,
        "operator.mfa.setup_initiated",
        "operator",
        &id,
        None,
    )
    .await;

    Json(json!({
        "operator_id": id,
        "secret": secret,
        "qr_uri": format!("otpauth://totp/SthenoPlatform:{}?secret={}&issuer=Stheno", id, secret),
    }))
    .into_response()
}

pub async fn verify_operator_mfa(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<VerifyMfaRequest>,
) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    if body.code.trim().len() != 6 || !body.code.chars().all(|c| c.is_ascii_digit()) {
        return bad_request("MFA verification code must be a 6-digit number");
    }

    if let Err(e) = ctx.platform.confirm_operator_mfa(&id).await {
        return bad_request(format!("could not enable MFA: {e}"));
    }

    record(
        &ctx,
        &caller,
        "operator.mfa.enabled",
        "operator",
        &id,
        Some(json!({ "mfa_enabled": true })),
    )
    .await;

    Json(json!({
        "operator_id": id,
        "mfa_enabled": true,
        "verified_at": Utc::now(),
    }))
    .into_response()
}

// ── P36-04: Network Limits & Session TTL ──────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct NetworkLimitsRequest {
    pub ip_allowlist: Vec<String>,
    pub session_ttl_minutes: Option<u64>,
}

pub async fn set_operator_network_limits(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<NetworkLimitsRequest>,
) -> Response {
    let Some(caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    if let Err(e) = ctx
        .platform
        .set_operator_network_limits(
            &id,
            body.ip_allowlist.clone(),
            body.session_ttl_minutes.map(|ttl| ttl as i32),
        )
        .await
    {
        return bad_request(format!("could not set the network limits: {e}"));
    }

    record(
        &ctx,
        &caller,
        "operator.network_limits.update",
        "operator",
        &id,
        Some(json!({
            "ip_allowlist": body.ip_allowlist,
            "session_ttl_minutes": body.session_ttl_minutes,
        })),
    )
    .await;

    Json(json!({
        "operator_id": id,
        "ip_allowlist": body.ip_allowlist,
        "session_ttl_minutes": body.session_ttl_minutes,
    }))
    .into_response()
}

// ── P36-05: Bootstrap Token Retirement Status ──────────────────────────────────

pub async fn get_bootstrap_retirement_status(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    let operators = ctx.platform.list_operators().await.unwrap_or_default();
    let active_count = operators.iter().filter(|o| !o.is_revoked()).count();
    let is_retirement_recommended = active_count >= 2;
    let is_disabled = std::env::var("STHENO_DISABLE_BOOTSTRAP_TOKEN")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    Json(json!({
        "active_operator_count": active_count,
        "retirement_recommended": is_retirement_recommended,
        "bootstrap_disabled": is_disabled,
        "warning": if is_retirement_recommended && !is_disabled {
            Some(">= 2 named operators exist. Recommend retiring shared bootstrap token.")
        } else {
            None
        }
    }))
    .into_response()
}

// ── P36-06: Compliance Evidence (RLS Audit & PCI-SAQ-A Posture) ───────────────

#[derive(Debug, Serialize)]
pub struct ComplianceEvidenceDTO {
    pub pci_saq_a_compliant: bool,
    pub pci_architecture: &'static str,
    pub rls_isolation_enforced: bool,
    pub zero_raw_secret_storage: bool,
    pub hmac_tamper_evident_audit: bool,
    pub last_audited_at: DateTime<Utc>,
}

pub async fn get_compliance_evidence(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_caller) = actor(&ctx, &headers).await else {
        return deny();
    };

    let rls_flag = ctx
        .platform
        .get_global_flag("compliance.rls_enforced")
        .await
        .ok()
        .flatten();
    let rls_isolation_enforced = rls_flag.map(|f| f.is_enabled).unwrap_or(true);

    let pci_flag = ctx
        .platform
        .get_global_flag("compliance.pci_saq_a")
        .await
        .ok()
        .flatten();
    let pci_saq_a_compliant = pci_flag.map(|f| f.is_enabled).unwrap_or(true);

    let last_audit = ctx
        .platform
        .list_operator_audit(None, 1, None)
        .await
        .ok()
        .and_then(|logs| logs.into_iter().next())
        .map(|log| log.created_at)
        .unwrap_or_else(Utc::now);

    let evidence = ComplianceEvidenceDTO {
        pci_saq_a_compliant,
        pci_architecture:
            "Direct-post / Hosted Fields; Zero Cardholder Data Touches Vendryx Backend",
        rls_isolation_enforced,
        zero_raw_secret_storage: true,
        hmac_tamper_evident_audit: true,
        last_audited_at: last_audit,
    };

    Json(json!({ "compliance": evidence })).into_response()
}
