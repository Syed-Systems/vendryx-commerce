//! P35: Platform configuration, themes, feature flags, and broadcasts (PHASE4_TASKS P35).

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{actor, bad_request, deny, record, PlatformApiContext};

// ── Configuration and theme wire shapes ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCatalogItemDTO {
    pub id: String,
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub available_versions: Vec<String>,
    pub requires_tier_feature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantThemeAssignmentDTO {
    pub tenant_id: String,
    pub theme_id: String,
    pub version: String,
    pub is_custom: bool,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalFeatureFlagDTO {
    pub key: String,
    pub description: String,
    pub is_enabled: bool,
    pub rollout_percentage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDefaultsDTO {
    pub default_region: String,
    pub default_currency: String,
    pub default_tax_rate: f64,
    pub support_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfigDTO {
    pub email_provider: String,
    pub sms_provider: String,
    pub from_email: String,
    pub order_confirmation_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementDTO {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: String, // "info" | "warning" | "critical"
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// P37-08: theme assignments, feature flags, deployment defaults, messaging config and
// announcements all lived in `static OnceLock<RwLock<..>>` registries here. They are rows
// now — `platform.tenant_theme`, `platform.global_flag`, `platform.deployment_config` and
// `platform.announcement`. A restart used to reset every merchant's theme to unassigned
// and silently drop every live announcement, and on two nodes an operator's change only
// ever reached the one that served the request.

// ── P35-02: Theme Catalog ──────────────────────────────────────────────────────

pub async fn list_theme_catalog(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let themes = vec![
        ThemeCatalogItemDTO {
            id: "theme_default".to_string(),
            name: "Stheno Default Storefront".to_string(),
            description: "Modern, high-conversion default theme".to_string(),
            latest_version: "1.4.0".to_string(),
            available_versions: vec![
                "1.0.0".to_string(),
                "1.3.0".to_string(),
                "1.4.0".to_string(),
            ],
            requires_tier_feature: None,
        },
        ThemeCatalogItemDTO {
            id: "theme_luxury".to_string(),
            name: "Obsidian Luxury".to_string(),
            description: "Premium editorial fashion & cosmetics theme".to_string(),
            latest_version: "2.1.0".to_string(),
            available_versions: vec!["2.0.0".to_string(), "2.1.0".to_string()],
            requires_tier_feature: Some("custom_theme".to_string()),
        },
    ];

    let count = themes.len();
    Json(json!({ "themes": themes, "count": count })).into_response()
}

// ── P35-03: Per-Tenant Theme Assignment with Tier Validation ──────────────────

#[derive(Debug, Deserialize)]
pub struct AssignThemeRequest {
    pub theme_id: String,
    pub version: Option<String>,
}

pub async fn assign_tenant_theme(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AssignThemeRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    // Check tenant subscription tier for custom_theme feature if needed
    if body.theme_id == "theme_luxury" {
        if let Ok(sub) = ctx.platform.get_subscription(&id).await {
            if sub.tier == "free" {
                return (
                    axum::http::StatusCode::FORBIDDEN,
                    Json(json!({
                        "type": "forbidden",
                        "message": "Theme requires custom_theme feature on a paid tier"
                    })),
                )
                    .into_response();
            }
        }
    }

    let assignment = TenantThemeAssignmentDTO {
        tenant_id: id.clone(),
        theme_id: body.theme_id.clone(),
        version: body.version.unwrap_or_else(|| "latest".to_string()),
        is_custom: body.theme_id != "theme_default",
        assigned_at: Utc::now(),
    };

    if let Err(e) = ctx
        .platform
        .assign_tenant_theme(
            &id,
            &assignment.theme_id,
            &assignment.version,
            assignment.is_custom,
            &actor.id,
        )
        .await
    {
        return bad_request(format!("could not assign the theme: {e}"));
    }

    record(
        &ctx,
        &actor,
        "tenant.theme.assign",
        "tenant",
        &id,
        Some(json!({ "theme_id": assignment.theme_id, "version": assignment.version })),
    )
    .await;

    Json(json!({ "assignment": assignment })).into_response()
}

// ── P35-04: Theme Rollout ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ThemeRolloutRequest {
    pub target_version: String,
    pub percentage: u8,
}

pub async fn rollout_theme_version(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ThemeRolloutRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    record(
        &ctx,
        &actor,
        "theme.rollout",
        "theme",
        &id,
        Some(json!({
            "target_version": body.target_version,
            "percentage": body.percentage,
        })),
    )
    .await;

    Json(json!({
        "theme_id": id,
        "target_version": body.target_version,
        "percentage": body.percentage,
        "updated_at": Utc::now(),
    }))
    .into_response()
}

// ── P35-05: Global Feature Flags & Kill Switches ───────────────────────────────

pub async fn list_feature_flags(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let flags: Vec<GlobalFeatureFlagDTO> = match ctx.platform.list_global_flags("feature").await {
        Ok(rows) => rows
            .into_iter()
            .map(|row| GlobalFeatureFlagDTO {
                key: row.key,
                description: row.description,
                is_enabled: row.is_enabled,
                rollout_percentage: row.rollout_percentage.clamp(0, 100) as u8,
            })
            .collect(),
        Err(e) => return bad_request(format!("could not read the feature flags: {e}")),
    };
    let count = flags.len();
    Json(json!({ "feature_flags": flags, "count": count })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeatureFlagRequest {
    pub is_enabled: bool,
    pub rollout_percentage: Option<u8>,
}

pub async fn update_feature_flag(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateFeatureFlagRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let stored = match ctx
        .platform
        .set_global_flag(&id, "feature", body.is_enabled, None, &actor.id)
        .await
    {
        Ok(row) => row,
        Err(e) => return bad_request(format!("could not update the flag: {e}")),
    };

    let flag = GlobalFeatureFlagDTO {
        key: stored.key,
        description: stored.description,
        is_enabled: stored.is_enabled,
        rollout_percentage: body
            .rollout_percentage
            .unwrap_or(stored.rollout_percentage.clamp(0, 100) as u8),
    };

    record(
        &ctx,
        &actor,
        "feature_flag.update",
        "feature_flag",
        &id,
        Some(json!({ "is_enabled": body.is_enabled })),
    )
    .await;

    Json(json!({ "feature_flag": flag })).into_response()
}

// ── P35-06: Platform Region / Currency / Tax Defaults ──────────────────────────

pub async fn get_platform_defaults(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let config = match ctx.platform.get_deployment_config().await {
        Ok(row) => row,
        Err(e) => return bad_request(format!("could not read the defaults: {e}")),
    };

    let defaults = PlatformDefaultsDTO {
        default_region: config.default_region,
        default_currency: config.default_currency,
        default_tax_rate: config.default_tax_rate.to_string().parse().unwrap_or(0.0),
        support_email: config.support_email,
    };
    Json(json!({ "defaults": defaults })).into_response()
}

pub async fn update_platform_defaults(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<PlatformDefaultsDTO>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let mut config = match ctx.platform.get_deployment_config().await {
        Ok(row) => row,
        Err(e) => return bad_request(format!("could not read the defaults: {e}")),
    };
    config.default_region = body.default_region.clone();
    config.default_currency = body.default_currency.clone();
    // The column and every later multiplication are `Decimal` (ADR-006). The wire field is
    // still `f64`, so a rate that a float cannot represent exactly arrives already rounded.
    // Making the DTO a string — the fix `price` in config/tiers.toml already got — changes
    // the JSON contract apps/platform reads, so it is a follow-up rather than a drive-by.
    config.default_tax_rate =
        Decimal::from_f64_retain(body.default_tax_rate).unwrap_or(Decimal::ZERO);
    config.support_email = body.support_email.clone();

    if let Err(e) = ctx.platform.save_deployment_config(config, &actor.id).await {
        return bad_request(format!("could not save the defaults: {e}"));
    }

    record(
        &ctx,
        &actor,
        "platform.defaults.update",
        "platform_config",
        "defaults",
        Some(json!({ "defaults": body })),
    )
    .await;

    Json(json!({ "defaults": body })).into_response()
}

// ── P35-07: Messaging (Email/SMS) Configuration ───────────────────────────────

pub async fn get_messaging_config(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let stored = match ctx.platform.get_deployment_config().await {
        Ok(row) => row,
        Err(e) => return bad_request(format!("could not read the messaging config: {e}")),
    };

    let config = MessagingConfigDTO {
        email_provider: stored.email_provider,
        sms_provider: stored.sms_provider,
        from_email: stored.from_email,
        order_confirmation_template: stored.order_confirmation_template,
    };
    Json(json!({ "messaging": config })).into_response()
}

pub async fn update_messaging_config(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<MessagingConfigDTO>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    let mut config = match ctx.platform.get_deployment_config().await {
        Ok(row) => row,
        Err(e) => return bad_request(format!("could not read the messaging config: {e}")),
    };
    config.email_provider = body.email_provider.clone();
    config.sms_provider = body.sms_provider.clone();
    config.from_email = body.from_email.clone();
    config.order_confirmation_template = body.order_confirmation_template.clone();

    if let Err(e) = ctx.platform.save_deployment_config(config, &actor.id).await {
        return bad_request(format!("could not save the messaging config: {e}"));
    }

    record(
        &ctx,
        &actor,
        "platform.messaging.update",
        "platform_config",
        "messaging",
        Some(json!({ "messaging": body })),
    )
    .await;

    Json(json!({ "messaging": body })).into_response()
}

// ── P35-08: Broadcast Announcements with Expiry ────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub message: String,
    pub severity: Option<String>,
    pub expires_in_days: Option<i64>,
}

pub async fn list_announcements(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
) -> Response {
    let Some(_actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    // The operator's own view lists expired announcements too: the record of what
    // merchants were told during an outage is exactly what gets asked for afterwards.
    let items: Vec<AnnouncementDTO> = match ctx.platform.list_all_announcements().await {
        Ok(rows) => rows
            .into_iter()
            .map(|row| AnnouncementDTO {
                id: row.id.to_string(),
                title: row.title,
                message: row.message,
                severity: row.severity,
                expires_at: row.expires_at,
                created_at: row.created_at,
            })
            .collect(),
        Err(e) => return bad_request(format!("could not read announcements: {e}")),
    };
    let count = items.len();
    Json(json!({ "announcements": items, "count": count })).into_response()
}

pub async fn create_announcement(
    State(ctx): State<PlatformApiContext>,
    headers: HeaderMap,
    Json(body): Json<CreateAnnouncementRequest>,
) -> Response {
    let Some(actor) = actor(&ctx, &headers).await else {
        return deny();
    };

    if body.title.trim().is_empty() || body.message.trim().is_empty() {
        return bad_request("Title and message are required for announcements");
    }

    let now = Utc::now();
    let days = body.expires_in_days.unwrap_or(7);
    let expires_at = now + chrono::Duration::days(days);
    let severity = body.severity.unwrap_or_else(|| "info".to_string());

    let stored = match ctx
        .platform
        .create_announcement(&body.title, &body.message, &severity, expires_at, &actor.id)
        .await
    {
        Ok(row) => row,
        Err(e) => return bad_request(format!("could not create the announcement: {e}")),
    };

    let id = stored.id.to_string();
    let announcement = AnnouncementDTO {
        id: id.clone(),
        title: stored.title,
        message: stored.message,
        severity: stored.severity,
        expires_at: stored.expires_at,
        created_at: stored.created_at,
    };

    record(
        &ctx,
        &actor,
        "platform.announcement.create",
        "announcement",
        &id,
        Some(json!({ "title": announcement.title, "expires_at": expires_at })),
    )
    .await;

    Json(json!({ "announcement": announcement })).into_response()
}
