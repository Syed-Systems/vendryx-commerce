//! Operator-only launch queue. These records exist before a tenant, so they stay on the
//! control-plane surface and never appear under `/admin/*`.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{actor, bad_request, deny, record, PlatformApiContext};

#[derive(Debug, Serialize)]
struct LaunchRequestView {
    id: String,
    merchant_id: String,
    merchant_email: String,
    merchant_name: Option<String>,
    business_name: Option<String>,
    whatsapp: Option<String>,
    category: Option<String>,
    store_name: String,
    subdomain: String,
    tier: String,
    payment_status: String,
    status: String,
    logo_url: Option<String>,
    accent_color: String,
    icon_choice: String,
    buyer_accounts: bool,
    bespoke_brief: Option<String>,
    validation_findings: Vec<String>,
    assigned_developer: Option<String>,
    submitted_at: String,
    updated_at: String,
}

fn string_draft_field(draft: &serde_json::Value, field: &str) -> Option<String> {
    draft.get(field)?.as_str().map(ToString::to_string)
}

async fn view(
    ctx: &PlatformApiContext,
    request: stheno_platform::LaunchRequestDTO,
) -> LaunchRequestView {
    let account = ctx
        .platform
        .get_merchant_account(&request.merchant_id)
        .await
        .ok();
    let draft = account
        .as_ref()
        .map(|account| &account.signup_draft)
        .cloned()
        .unwrap_or_else(|| json!({}));
    let validation_findings = request
        .validation_findings
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();
    LaunchRequestView {
        id: request.id,
        merchant_id: request.merchant_id,
        merchant_email: account
            .as_ref()
            .map(|account| account.email.clone())
            .unwrap_or_else(|| "Unavailable".to_string()),
        merchant_name: string_draft_field(&draft, "owner_contact"),
        business_name: string_draft_field(&draft, "business_name"),
        whatsapp: string_draft_field(&draft, "whatsapp"),
        category: string_draft_field(&draft, "category"),
        store_name: request.store_name,
        subdomain: request.subdomain,
        tier: request.tier,
        payment_status: request.payment_status,
        status: request.launch_status,
        logo_url: request.logo_url,
        accent_color: request.accent_color,
        icon_choice: request.icon_choice,
        buyer_accounts: request.buyer_accounts,
        bespoke_brief: request.bespoke_brief,
        validation_findings,
        assigned_developer: request.assigned_developer,
        submitted_at: request.submitted_at.to_rfc3339(),
        updated_at: request.updated_at.to_rfc3339(),
    }
}

pub async fn list_launch_requests(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }
    match ctx.platform.list_launch_requests().await {
        Ok(rows) => {
            let mut launch_requests = Vec::with_capacity(rows.len());
            for row in rows {
                launch_requests.push(view(&ctx, row).await);
            }
            let count = launch_requests.len();
            Json(json!({ "launch_requests": launch_requests, "count": count })).into_response()
        }
        Err(error) => bad_request(error),
    }
}

pub async fn get_launch_request(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if actor(&ctx, &headers).await.is_none() {
        return deny();
    }
    let request = match ctx.platform.get_launch_request(&id).await {
        Ok(request) => request,
        Err(_) => return deny(),
    };
    match ctx.platform.list_launch_request_events(&id).await {
        Ok(timeline) => {
            Json(json!({ "launch_request": view(&ctx, request).await, "timeline": timeline }))
                .into_response()
        }
        Err(error) => bad_request(error),
    }
}

#[derive(Debug, Deserialize)]
pub struct LaunchActionBody {
    pub assigned_developer: Option<String>,
    pub note: Option<String>,
}

pub async fn transition_launch_request(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path((id, action)): Path<(String, String)>,
    Json(body): Json<LaunchActionBody>,
) -> Response {
    let Some(operator) = actor(&ctx, &headers).await else {
        return deny();
    };
    if !matches!(
        action.as_str(),
        "assign"
            | "request_changes"
            | "begin_provisioning"
            | "retry"
            | "activate"
            | "reject"
            | "reopen"
    ) {
        return deny();
    }
    let result = match action.as_str() {
        "begin_provisioning" | "retry" => {
            match ctx
                .platform
                .transition_launch_request(
                    &id,
                    &action,
                    operator.id.clone(),
                    operator.name.clone(),
                    body.assigned_developer,
                    body.note,
                )
                .await
            {
                Ok(_) => {
                    ctx.platform
                        .provision_launch_request(&id, &operator.id, &operator.name)
                        .await
                }
                Err(error) => Err(error),
            }
        }
        "activate" => {
            ctx.platform
                .activate_launch_request(&id, &operator.id, &operator.name)
                .await
        }
        _ => {
            ctx.platform
                .transition_launch_request(
                    &id,
                    &action,
                    operator.id.clone(),
                    operator.name.clone(),
                    body.assigned_developer,
                    body.note,
                )
                .await
        }
    };
    match result {
        Ok(request) => {
            record(
                &ctx,
                &operator,
                &format!("launch_request.{action}"),
                "launch_request",
                &id,
                Some(json!({ "status": request.launch_status })),
            )
            .await;
            Json(json!({ "launch_request": view(&ctx, request).await })).into_response()
        }
        Err(error) => bad_request(error),
    }
}
