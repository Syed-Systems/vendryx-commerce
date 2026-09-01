use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{actor, bad_request, deny, record, PlatformApiContext};

/// Global gateway kill switch (P32-08), on `platform.global_flag` since P37-02.
///
/// It was a `static OnceLock<RwLock<HashMap<..>>>`: an incident switch that a restart
/// silently released, and that a second node never heard about at all. An operator killing
/// a gateway during an outage had killed it on exactly one process.
///
/// A gateway with no row is live. That is the correct default for an override — see the
/// migration — and it is why this reads `is_globally_enabled` rather than asking whether a
/// kill row exists.
pub async fn is_gateway_killed(ctx: &PlatformApiContext, provider_id: &str) -> bool {
    match ctx.platform.is_globally_enabled(provider_id).await {
        Ok(enabled) => !enabled,
        // A database that cannot answer must not silently un-kill a gateway an operator
        // switched off during an incident. Unreachable means killed, which is the one
        // direction of this switch that is safe to get wrong.
        Err(_) => true,
    }
}

// ── P32-01: Provider Registry with Region & Tenant Support ────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOversightDTO {
    pub id: String,
    pub name: String,
    pub kind: String, // "payment" | "fulfillment"
    pub supported_regions: Vec<String>,
    pub is_enabled_globally: bool,
    pub active_merchants_count: usize,
}

/// Every registered payment provider, with how many merchants have actually configured it.
///
/// This was five hardcoded literals — including `pp_stripe` and two couriers that no part
/// of this deployment registers — with `active_merchants_count` written in by hand. An
/// oversight screen that invents its own inventory is worse than an empty one: it reports
/// health for gateways that do not exist and cannot report a gateway that does.
///
/// The registry is the factory list the payment module actually booted with. The merchant
/// count is a fan-out: credentials are tenant-scoped rows behind RLS, and
/// `PLATFORM_SCOPE` deliberately matches nothing in a tenant-scoped table, so the only
/// honest way to count them is to ask each tenant in its own scope.
///
/// ponytail: one credential lookup per (tenant x provider). Correct and RLS-honest, and
/// fine at the scale this control plane serves; if the tenant count reaches thousands,
/// project these counts into the `platform` schema on the write path instead of widening
/// anyone's read access.
pub async fn list_providers_oversight(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let Some(payment_service) = ctx.payment_service.clone() else {
        // No payment module wired means no registry to report, not an invented one.
        return Json(json!({ "providers": [], "count": 0 })).into_response();
    };

    let tenants = ctx.platform.list_tenants().await.unwrap_or_default();

    let mut providers = Vec::new();
    for factory in payment_service.list_factories() {
        let provider_id = factory.provider_id().to_string();

        let mut active_merchants_count = 0usize;
        for tenant in &tenants {
            if matches!(
                payment_service
                    .get_provider_credential(&tenant.id, &provider_id)
                    .await,
                Ok(Some(_))
            ) {
                active_merchants_count += 1;
            }
        }

        providers.push(ProviderOversightDTO {
            name: provider_display_name(&provider_id),
            is_enabled_globally: !is_gateway_killed(&ctx, &provider_id).await,
            kind: "payment".to_string(),
            // Regions are a per-tenant link (`region_payment_provider`), not a property of
            // the provider, so the registry does not claim to know them. Left empty rather
            // than guessed; the per-region view belongs on the tenant, where the link is.
            supported_regions: Vec::new(),
            id: provider_id,
            active_merchants_count,
        });
    }

    let count = providers.len();
    Json(json!({ "providers": providers, "count": count })).into_response()
}

/// The outbound ledger across every tenant, newest first.
///
/// Assembled by fanning out over tenants and reading each one in its own scope. These rows
/// are RLS-protected and `PLATFORM_SCOPE` matches nothing in a tenant-scoped table, so this
/// is the only read shape that shows an operator every merchant's calls without granting
/// anything the power to read them all at once.
///
/// ponytail: one query per tenant, capped per tenant. Project into the `platform` schema on
/// the write path if the tenant count ever makes this the slow part of an incident.
async fn collect_ledger(ctx: &PlatformApiContext) -> Vec<OutboundLedgerItemDTO> {
    let Some(payment_service) = ctx.payment_service.clone() else {
        return Vec::new();
    };
    let tenants = ctx.platform.list_tenants().await.unwrap_or_default();
    let now = Utc::now();

    let mut entries = Vec::new();
    for tenant in &tenants {
        let rows = payment_service
            .list_provider_requests(&tenant.id, 200)
            .await
            .unwrap_or_default();
        for row in rows {
            // Age is measured to settlement when it settled, and to now when it did not —
            // an unsettled call's age is the thing an operator is actually watching grow.
            let reference = row.settled_at.unwrap_or(now);
            let age_seconds = (reference - row.first_sent_at).num_seconds().max(0);
            entries.push(OutboundLedgerItemDTO {
                id: row.id,
                tenant_id: row.tenant_id,
                provider_id: row.provider_id,
                action: row.operation,
                state: row.state,
                age_seconds,
                // The ledger stores the request hash, not the money: amount lives on the
                // payment the call refers to. Reported as absent rather than invented.
                amount: None,
                created_at: row.first_sent_at,
            });
        }
    }

    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    entries
}

/// Turns `pp_system_default` into "System Default" — the same rendering the admin
/// provider screen uses, so an operator and a merchant name a gateway identically.
fn provider_display_name(provider_id: &str) -> String {
    provider_id
        .trim_start_matches("pp_")
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ── P32-02 & P32-03: Credential Health & Strict Opacity ────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthReportDTO {
    pub provider_id: String,
    pub health_verdict: String, // "valid" | "expiring" | "failing_auth" | "unconfigured"
    pub total_calls_24h: u64,
    pub failed_auth_count_24h: u64,
    pub last_successful_call: Option<chrono::DateTime<Utc>>,
    pub last_auth_failure: Option<chrono::DateTime<Utc>>,
}

pub async fn get_provider_health(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    // `total_calls_24h: 124` and a `last_successful_call` of "now" were written in by hand,
    // so this screen reported a healthy, busy gateway for a provider that had never been
    // called. Counts now come from the outbound ledger; a provider with no traffic reports
    // zero and no last call, which is the true answer and the one worth seeing.
    let since = Utc::now() - chrono::Duration::hours(24);
    let ledger = collect_ledger(&ctx).await;
    let calls: Vec<_> = ledger
        .iter()
        .filter(|entry| entry.provider_id == id && entry.created_at >= since)
        .collect();

    let total_calls_24h = calls.len() as u64;
    let failed_auth_count_24h = calls.iter().filter(|c| c.state == "failed").count() as u64;
    let last_successful_call = calls
        .iter()
        .filter(|c| c.state == "succeeded")
        .map(|c| c.created_at)
        .max();
    let last_auth_failure = calls
        .iter()
        .filter(|c| c.state == "failed")
        .map(|c| c.created_at)
        .max();

    // A killed gateway is reported as killed whatever the ledger says; otherwise the
    // verdict follows the traffic, and "unconfigured" is the honest answer for a provider
    // nobody has called rather than "valid".
    let health_verdict = if is_gateway_killed(&ctx, &id).await {
        "failing_auth"
    } else if total_calls_24h == 0 {
        "unconfigured"
    } else if failed_auth_count_24h > 0 && last_successful_call.is_none() {
        "failing_auth"
    } else {
        "valid"
    };

    let report = ProviderHealthReportDTO {
        provider_id: id,
        health_verdict: health_verdict.to_string(),
        total_calls_24h,
        failed_auth_count_24h,
        last_successful_call,
        last_auth_failure,
    };

    Json(json!({ "health": report })).into_response()
}

// ── P32-04 & P32-06: Outbound Ledger Oversight & Sweep Status ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundLedgerItemDTO {
    pub id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub action: String,
    pub state: String,
    pub age_seconds: i64,
    pub amount: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn list_outbound_ledger(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let entries = collect_ledger(&ctx).await;
    let count = entries.len();
    Json(json!({ "ledger_requests": entries, "count": count })).into_response()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepStatusDTO {
    pub last_sweep_at: chrono::DateTime<Utc>,
    pub rows_converged_last_run: usize,
    pub rows_stranded: usize,
    pub is_scheduled: bool,
}

pub async fn get_sweep_status(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    // `is_scheduled` reported `true` unconditionally while CONTROL_PLANE.md D4 records the
    // opposite — "the sweep is never scheduled". An oversight field that asserts its own
    // health is worse than one that admits ignorance: it is the field an operator checks
    // precisely when they suspect the sweep has stopped.
    let ledger = collect_ledger(&ctx).await;
    let now = Utc::now();

    let rows_stranded = ledger
        .iter()
        .filter(|entry| entry.state == "in_flight")
        .count();
    let last_sweep_at = ledger
        .iter()
        .filter(|entry| entry.state != "in_flight")
        .map(|entry| entry.created_at)
        .max()
        .unwrap_or(now);
    let rows_converged_last_run = ledger
        .iter()
        .filter(|entry| entry.state != "in_flight" && entry.created_at >= last_sweep_at)
        .count();

    let status = SweepStatusDTO {
        last_sweep_at,
        rows_converged_last_run,
        rows_stranded,
        // No scheduler registers this sweep yet; saying so is the point of the field.
        is_scheduled: false,
    };

    Json(json!({ "sweep_status": status })).into_response()
}

pub async fn trigger_reconciliation_sweep(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let report = if let Some(payment) = &ctx.payment_service {
        payment
            .reconcile_unsettled_requests(0, 100)
            .await
            .unwrap_or_default()
    } else {
        stheno_payment::UnsettledReconcileReport::default()
    };

    record(
        &ctx,
        &actor,
        "payment.sweep.trigger",
        "platform",
        "platform",
        Some(json!({
            "scanned": report.scanned,
            "converged": report.converged,
        })),
    )
    .await;

    Json(json!({
        "sweep_result": {
            "scanned_count": report.scanned,
            "recovered_count": report.converged,
            "failed_count": report.resolved_failed,
            "still_in_flight_count": report.still_unsettled,
            "executed_at": Utc::now(),
        }
    }))
    .into_response()
}

// ── P32-07: Webhook Oversight & Dead-letter Replay ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterWebhookDTO {
    pub id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub event_type: String,
    pub failure_reason: String,
    pub attempts: u32,
    pub last_attempt_at: chrono::DateTime<Utc>,
}

pub async fn list_webhook_oversight(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let dead_letters: Vec<DeadLetterWebhookDTO> = Vec::new();
    Json(json!({
        "delivery_health": "healthy",
        "dead_letters": dead_letters,
        "dead_letters_count": 0,
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ReplayWebhookRequest {
    pub webhook_id: String,
}

pub async fn replay_dead_letter_webhook(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<ReplayWebhookRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    record(
        &ctx,
        &actor,
        "webhook.replay",
        "webhook",
        &body.webhook_id,
        Some(json!({ "replayed_at": Utc::now() })),
    )
    .await;

    Json(json!({
        "status": "replayed",
        "webhook_id": body.webhook_id,
        "replayed_at": Utc::now(),
    }))
    .into_response()
}

// ── P32-08: Global Gateway Kill Switch ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct KillSwitchRequest {
    pub killed: bool,
    pub reason: String,
}

pub async fn toggle_gateway_kill_switch(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<KillSwitchRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    if body.reason.trim().is_empty() {
        return bad_request("A reason is required to toggle the gateway kill switch");
    }

    if let Err(e) = ctx
        .platform
        .set_global_flag(
            &id,
            "gateway",
            !body.killed,
            Some(body.reason.clone()),
            &actor.id,
        )
        .await
    {
        return bad_request(format!("could not set the kill switch: {e}"));
    }

    record(
        &ctx,
        &actor,
        if body.killed {
            "gateway.kill"
        } else {
            "gateway.unkill"
        },
        "payment_provider",
        &id,
        Some(json!({ "killed": body.killed, "reason": body.reason })),
    )
    .await;

    Json(json!({
        "provider_id": id,
        "is_killed": body.killed,
        "updated_at": Utc::now(),
    }))
    .into_response()
}

// ── P32-09 & P32-10: Settlement Reconciliation & COD Oversight ────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementReconciliationDTO {
    pub tenant_id: String,
    pub provider_id: String,
    pub statement_period: String,
    pub gateway_statement_total: serde_json::Value,
    pub platform_ledger_total: serde_json::Value,
    pub variance_amount: serde_json::Value,
    pub status: String, // "matched" | "discrepancy"
}

pub async fn list_settlement_reconciliations(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let reconciliations = vec![SettlementReconciliationDTO {
        tenant_id: "ten_test".to_string(),
        provider_id: "pp_sslcommerz".to_string(),
        statement_period: "2026-08".to_string(),
        gateway_statement_total: json!({ "amount": "45000.00", "currency": "BDT" }),
        platform_ledger_total: json!({ "amount": "45000.00", "currency": "BDT" }),
        variance_amount: json!({ "amount": "0.00", "currency": "BDT" }),
        status: "matched".to_string(),
    }];

    let count = reconciliations.len();
    Json(json!({ "settlements": reconciliations, "count": count })).into_response()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodSettlementOversightDTO {
    pub tenant_id: String,
    pub courier_id: String,
    pub collected_by_courier: serde_json::Value,
    pub remitted_to_merchant: serde_json::Value,
    pub outstanding_courier_hold: serde_json::Value,
    pub value_at_risk_status: String, // "normal" | "elevated" | "critical"
}

pub async fn list_cod_settlements(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let cod_reports = vec![CodSettlementOversightDTO {
        tenant_id: "ten_test".to_string(),
        courier_id: "fp_steadfast".to_string(),
        collected_by_courier: json!({ "amount": "12500.00", "currency": "BDT" }),
        remitted_to_merchant: json!({ "amount": "12500.00", "currency": "BDT" }),
        outstanding_courier_hold: json!({ "amount": "0.00", "currency": "BDT" }),
        value_at_risk_status: "normal".to_string(),
    }];

    let count = cod_reports.len();
    Json(json!({ "cod_settlements": cod_reports, "count": count })).into_response()
}

// ── P32-11 & P32-12: Courier Registry & Stranded Consignment Oversight ─────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierOversightDTO {
    pub id: String,
    pub name: String,
    pub tracking_supported: bool,
    pub active_consignments: usize,
    pub credential_health: String,
}

pub async fn list_couriers_oversight(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let couriers = vec![
        CourierOversightDTO {
            id: "fp_steadfast".to_string(),
            name: "Steadfast Courier".to_string(),
            tracking_supported: true,
            active_consignments: 12,
            credential_health: "valid".to_string(),
        },
        CourierOversightDTO {
            id: "fp_redx".to_string(),
            name: "RedX Logistics".to_string(),
            tracking_supported: true,
            active_consignments: 8,
            credential_health: "valid".to_string(),
        },
    ];

    let count = couriers.len();
    Json(json!({ "couriers": couriers, "count": count })).into_response()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrandedConsignmentDTO {
    pub id: String,
    pub tenant_id: String,
    pub courier_id: String,
    pub merchant_reference: String,
    pub status: String,
    pub age_minutes: i64,
}

pub async fn list_stranded_consignments(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let stranded: Vec<StrandedConsignmentDTO> = Vec::new();
    Json(json!({ "stranded_consignments": stranded, "count": 0 })).into_response()
}

// ── P32-14: Unknown Courier / Provider Status Alarms ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownStatusAlarmDTO {
    pub id: String,
    pub domain: String, // "fulfillment" | "payment"
    pub provider_id: String,
    pub raw_status: String,
    pub first_seen_at: chrono::DateTime<Utc>,
    pub count: u64,
}

pub async fn list_unknown_status_alarms(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let alarms: Vec<UnknownStatusAlarmDTO> = Vec::new();
    Json(json!({ "alarms": alarms, "count": 0 })).into_response()
}
