//! Tenant-scoped AI and social channel configuration endpoints.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::NaiveDate;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use std::str::FromStr;
use std::sync::OnceLock;
use stheno_ai::{
    compare_adjacent_source_snapshots, AiConfigurationService, ChatOfferService,
    CreateKnowledgeEntry, KnowledgeEvidenceReference, KnowledgeService, SaveAiConfiguration,
    SaveChatOffer, StoreCategoryObservation, StoreIntelligenceRanking,
    StoreIntelligenceReviewService, StoreIntelligenceSourceService, StoreOrderObservation,
    UpdateKnowledgeEntry,
};
use stheno_api_core::ErrorResponse;
use stheno_auth::JwtClaims;
use stheno_user::UserService;

static LOCAL_RANKING_SLOT: OnceLock<tokio::sync::Semaphore> = OnceLock::new();

fn try_local_ranking_slot(
) -> Result<tokio::sync::SemaphorePermit<'static>, tokio::sync::TryAcquireError> {
    LOCAL_RANKING_SLOT
        .get_or_init(|| tokio::sync::Semaphore::new(1))
        .try_acquire()
}

#[derive(Clone)]
pub struct AiAdminContext {
    pub service: Arc<AiConfigurationService>,
    pub pool: stheno_db::DbPool,
    pub knowledge: KnowledgeService,
    pub chat_offers: ChatOfferService,
    pub users: Arc<UserService>,
    pub order_service: Arc<stheno_order::OrderService>,
    pub intelligence_reviews: StoreIntelligenceReviewService,
    pub intelligence_sources: StoreIntelligenceSourceService,
}

#[derive(Deserialize)]
pub struct KnowledgePage {
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

#[derive(Deserialize, Default)]
pub struct StoreIntelligenceQuery {
    #[serde(default)]
    rank_local: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchConsoleQuery {
    property: String,
    start_date: String,
    end_date: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchConsoleSnapshotQuery {
    property: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaInsightsQuery {
    channel: String,
    account_id: String,
    start_date: String,
    end_date: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaInsightsSnapshotQuery {
    channel: String,
    account_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhatsAppInsightsQuery {
    account_id: String,
    start_date: String,
    end_date: String,
}

#[derive(Serialize)]
struct WhatsAppInsightsReport {
    source: &'static str,
    account_id: String,
    window_started_on: NaiveDate,
    window_ended_on: NaiveDate,
    window_timezone: &'static str,
    ingested_at: chrono::DateTime<chrono::Utc>,
    coverage: stheno_ai::MetricCoverage,
    rows_received: usize,
    inbound_events: i64,
    consented_conversations: i64,
}

fn default_limit() -> u32 {
    50
}

pub async fn list_knowledge(
    State(context): State<AiAdminContext>,
    Query(page): Query<KnowledgePage>,
) -> Result<impl IntoResponse, Response> {
    let pool = context.pool.clone();
    let knowledge = context.knowledge;
    let entries = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { knowledge.list(&mut conn, page.limit, page.offset).await })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "knowledge": entries }))))
}

pub async fn create_knowledge(
    State(context): State<AiAdminContext>,
    Json(input): Json<CreateKnowledgeEntry>,
) -> Result<impl IntoResponse, Response> {
    let pool = context.pool.clone();
    let knowledge = context.knowledge;
    let entry = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { knowledge.create(&mut conn, input).await })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::CREATED, Json(json!({ "knowledge": entry }))))
}

pub async fn update_knowledge(
    State(context): State<AiAdminContext>,
    Path(id): Path<String>,
    Json(input): Json<UpdateKnowledgeEntry>,
) -> Result<impl IntoResponse, Response> {
    let pool = context.pool.clone();
    let knowledge = context.knowledge;
    let entry = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { knowledge.update(&mut conn, &id, input).await })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "knowledge": entry }))))
}

pub async fn delete_knowledge(
    State(context): State<AiAdminContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let pool = context.pool.clone();
    let knowledge = context.knowledge;
    stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { knowledge.delete(&mut conn, &id).await })
    })
    .await
    .map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_configurations(
    State(context): State<AiAdminContext>,
) -> Result<impl IntoResponse, Response> {
    let configurations = context.service.list().await.map_err(map_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "configurations": configurations })),
    ))
}

pub async fn save_configuration(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Json(input): Json<SaveAiConfiguration>,
) -> Result<impl IntoResponse, Response> {
    let configuration = if matches!(
        input.kind.to_ascii_lowercase().as_str(),
        "search_console" | "meta_insights_facebook" | "meta_insights_instagram"
    ) {
        let actor_id = require_store_owner_or_admin(&context, &claims).await?;
        context
            .service
            .save_with_actor(input, &actor_id)
            .await
            .map_err(map_error)?
    } else {
        context.service.save(input).await.map_err(map_error)?
    };
    Ok((
        StatusCode::OK,
        Json(json!({ "configuration": configuration })),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisconnectSearchConsoleRequest {
    property: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisconnectMetaInsightsRequest {
    channel: String,
    account_id: String,
}

pub async fn disconnect_meta_insights(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Json(input): Json<DisconnectMetaInsightsRequest>,
) -> Result<impl IntoResponse, Response> {
    let actor_id = require_store_owner_or_admin(&context, &claims).await?;
    context
        .service
        .disconnect_meta_insights_with_actor(&input.channel, &input.account_id, &actor_id)
        .await
        .map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartSearchConsoleOAuthRequest {
    property: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteSearchConsoleOAuthRequest {
    code: String,
    state: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchConsoleOAuthState {
    tenant_id: String,
    actor_id: String,
    property: String,
    nonce: String,
    issued_at: i64,
}

const SEARCH_CONSOLE_OAUTH_SCOPE: &str = "https://www.googleapis.com/auth/webmasters.readonly";

pub async fn disconnect_search_console(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Json(input): Json<DisconnectSearchConsoleRequest>,
) -> Result<impl IntoResponse, Response> {
    let actor_id = require_store_owner_or_admin(&context, &claims).await?;
    context
        .service
        .disconnect_search_console_with_actor(&input.property, &actor_id)
        .await
        .map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_search_console_oauth(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Json(input): Json<StartSearchConsoleOAuthRequest>,
) -> Result<impl IntoResponse, Response> {
    let actor_id = require_store_owner_or_admin(&context, &claims).await?;
    let property =
        stheno_ai::normalize_search_console_property(&input.property).map_err(map_error)?;
    let (client_id, client_secret, redirect_uri) = search_console_oauth_environment()?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let nonce = ulid::Ulid::new().to_string();
    context
        .service
        .create_search_console_oauth_state(&nonce, &property, &actor_id)
        .await
        .map_err(map_error)?;
    let state = sign_search_console_oauth_state(
        SearchConsoleOAuthState {
            tenant_id,
            actor_id,
            property,
            nonce,
            issued_at: chrono::Utc::now().timestamp(),
        },
        &client_secret,
    )
    .map_err(|_| oauth_configuration_error())?;
    let mut authorization_url = url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|_| oauth_configuration_error())?;
    authorization_url.query_pairs_mut().extend_pairs([
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", SEARCH_CONSOLE_OAUTH_SCOPE),
        ("access_type", "offline"),
        ("prompt", "consent"),
        ("state", state.as_str()),
    ]);
    Ok((
        StatusCode::OK,
        Json(json!({ "authorization_url": authorization_url.as_str() })),
    ))
}

pub async fn complete_search_console_oauth(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Json(input): Json<CompleteSearchConsoleOAuthRequest>,
) -> Result<impl IntoResponse, Response> {
    let actor_id = require_store_owner_or_admin(&context, &claims).await?;
    if input.code.is_empty() || input.code.len() > 4096 || input.state.len() > 8192 {
        return Err(oauth_state_error());
    }
    let (client_id, client_secret, redirect_uri) = search_console_oauth_environment()?;
    let state = verify_search_console_oauth_state(&input.state, &client_secret)
        .map_err(|_| oauth_state_error())?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let now = chrono::Utc::now().timestamp();
    if state.tenant_id != tenant_id
        || state.actor_id != actor_id
        || now.saturating_sub(state.issued_at) > 600
        || state.issued_at > now.saturating_add(30)
    {
        return Err(oauth_state_error());
    }
    let consumed = context
        .service
        .consume_search_console_oauth_state(&state.nonce, &state.property, &state.actor_id)
        .await
        .map_err(map_error)?;
    if !consumed {
        return Err(oauth_state_error());
    }
    let adapter = stheno_search_console::GoogleSearchConsoleAdapter::new()
        .map_err(search_console_error_response)?;
    let refresh_token = adapter
        .exchange_authorization_code(&input.code, &client_id, &client_secret, &redirect_uri)
        .await
        .map_err(search_console_error_response)?;
    let mut secrets = std::collections::BTreeMap::new();
    secrets.insert("client_id".to_string(), client_id);
    secrets.insert("client_secret".to_string(), client_secret);
    secrets.insert("refresh_token".to_string(), refresh_token);
    let configuration = context
        .service
        .save_with_actor(
            SaveAiConfiguration {
                kind: "search_console".to_string(),
                account_id: state.property.clone(),
                secrets,
                enabled: true,
            },
            &actor_id,
        )
        .await
        .map_err(map_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "configuration": configuration })),
    ))
}

pub async fn get_search_console_insights(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<SearchConsoleQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let start = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_window",
                "The Search Console date window is invalid.",
            )),
        )
            .into_response()
    })?;
    let end = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_window",
                "The Search Console date window is invalid.",
            )),
        )
            .into_response()
    })?;
    let configuration = context
        .service
        .load_search_console(&query.property)
        .await
        .map_err(map_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    "source_not_connected",
                    "That Search Console property is not connected.",
                )),
            )
                .into_response()
        })?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let adapter = stheno_search_console::GoogleSearchConsoleAdapter::new().map_err(|_| {
        search_console_error_response(stheno_search_console::SearchConsoleError::RequestFailed)
    })?;
    let report = adapter
        .collect(&configuration, &tenant_id, start, end)
        .await
        .map_err(search_console_error_response)?;
    let pool = context.pool.clone();
    let source_service = context.intelligence_sources;
    let report = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move {
            source_service
                .record_snapshot(
                    &mut conn,
                    report.source,
                    &report.property,
                    &report.observations,
                )
                .await?;
            Ok(report)
        })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "search_console": report }))))
}

pub async fn get_search_console_snapshots(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<SearchConsoleSnapshotQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let configuration = context
        .service
        .load_search_console(&query.property)
        .await
        .map_err(map_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    "source_not_connected",
                    "That Search Console property is not connected.",
                )),
            )
                .into_response()
        })?;
    let pool = context.pool.clone();
    let source_service = context.intelligence_sources;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let (snapshots, changes) = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move {
            let snapshots = source_service
                .list_recent(&mut conn, "search_console", &configuration.property)
                .await?;
            let changes = compare_adjacent_source_snapshots(&tenant_id, &snapshots);
            Ok((snapshots, changes))
        })
    })
    .await
    .map_err(map_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "snapshots": snapshots, "changes": changes })),
    ))
}

pub async fn get_search_console_audit(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<SearchConsoleSnapshotQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let history = context
        .service
        .list_search_console_audit(&query.property, 50)
        .await
        .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "history": history }))))
}

pub async fn get_meta_insights(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<MetaInsightsQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let (kind, source) = match query.channel.as_str() {
        "messenger" => (
            stheno_meta_insights::MetaInsightsKind::FacebookPage,
            "facebook_page",
        ),
        "instagram" => (
            stheno_meta_insights::MetaInsightsKind::InstagramAccount,
            "instagram_account",
        ),
        _ => {
            return Err(meta_insights_error_response(
                stheno_meta_insights::MetaInsightsError::InvalidRequest,
            ));
        }
    };
    let start = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d").map_err(|_| {
        meta_insights_error_response(stheno_meta_insights::MetaInsightsError::InvalidRequest)
    })?;
    let end = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d").map_err(|_| {
        meta_insights_error_response(stheno_meta_insights::MetaInsightsError::InvalidRequest)
    })?;
    let configuration = context
        .service
        .load_meta_insights(&query.channel, &query.account_id)
        .await
        .map_err(map_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    "source_not_connected",
                    "That Meta account is not connected.",
                )),
            )
                .into_response()
        })?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let graph_api_version =
        std::env::var("STHENO_META_INSIGHTS_API_VERSION").unwrap_or_else(|_| "v26.0".to_string());
    let adapter = stheno_meta_insights::MetaInsightsAdapter::new(&graph_api_version)
        .map_err(meta_insights_error_response)?;
    let report = adapter
        .collect(kind, &configuration, &tenant_id, start, end)
        .await
        .map_err(meta_insights_error_response)?;
    let pool = context.pool.clone();
    let source_service = context.intelligence_sources;
    let report = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move {
            source_service
                .record_snapshot(&mut conn, source, &report.account_id, &report.observations)
                .await?;
            Ok(report)
        })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "meta_insights": report }))))
}

pub async fn get_meta_insights_snapshots(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<MetaInsightsSnapshotQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let source = match query.channel.as_str() {
        "messenger" => "facebook_page",
        "instagram" => "instagram_account",
        _ => {
            return Err(meta_insights_error_response(
                stheno_meta_insights::MetaInsightsError::InvalidRequest,
            ));
        }
    };
    let pool = context.pool.clone();
    let source_service = context.intelligence_sources;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let (snapshots, changes) = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move {
            let snapshots = source_service
                .list_recent(&mut conn, source, &query.account_id)
                .await?;
            let changes = compare_adjacent_source_snapshots(&tenant_id, &snapshots);
            Ok((snapshots, changes))
        })
    })
    .await
    .map_err(map_error)?;
    Ok((
        StatusCode::OK,
        Json(json!({ "snapshots": snapshots, "changes": changes })),
    ))
}

pub async fn get_meta_insights_audit(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<MetaInsightsSnapshotQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let history = context
        .service
        .list_meta_insights_audit(&query.channel, &query.account_id, 50)
        .await
        .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "history": history }))))
}

pub async fn get_whatsapp_insights(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<WhatsAppInsightsQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let configured = context
        .service
        .list()
        .await
        .map_err(map_error)?
        .into_iter()
        .any(|configuration| {
            configuration.kind == "whatsapp"
                && configuration.account_id == query.account_id
                && configuration.enabled
                && configuration.configured
        });
    if !configured {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "source_not_connected",
                "That WhatsApp account is not connected.",
            )),
        )
            .into_response());
    }
    let start = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|_| whatsapp_insights_error())?;
    let end = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|_| whatsapp_insights_error())?;
    let tenant_id = stheno_db::tenant::current().map_err(map_error)?;
    let aggregate = stheno_query::query_consented_whatsapp_aggregate(
        &context.pool,
        &tenant_id,
        &query.account_id,
        start,
        end,
    )
    .await
    .map_err(map_error)?;
    let report = if let Some(aggregate) = aggregate {
        let coverage = stheno_ai::MetricCoverage::Partial;
        let sample_size = u64::try_from(aggregate.consented_conversations).ok();
        let observations = vec![
            stheno_ai::SourceMetricObservation {
                tenant_id: tenant_id.clone(),
                source: "whatsapp_consent".to_string(),
                account_id: aggregate.account_id.clone(),
                scope: "active_consent_population".to_string(),
                metric: "processed_inbound_messages".to_string(),
                definition: "whatsapp-consented-processed-inbound-v1".to_string(),
                unit: "count".to_string(),
                value: rust_decimal::Decimal::from(aggregate.inbound_events),
                denominator: None,
                sample_size,
                window_started_on: aggregate.window_started_on,
                window_ended_on: aggregate.window_ended_on,
                window_timezone: aggregate.window_timezone.to_string(),
                ingested_at: aggregate.ingested_at,
                coverage,
            },
            stheno_ai::SourceMetricObservation {
                tenant_id,
                source: "whatsapp_consent".to_string(),
                account_id: aggregate.account_id.clone(),
                scope: "active_consent_population".to_string(),
                metric: "consented_conversations".to_string(),
                definition: "whatsapp-consented-distinct-senders-v1".to_string(),
                unit: "count".to_string(),
                value: rust_decimal::Decimal::from(aggregate.consented_conversations),
                denominator: None,
                sample_size,
                window_started_on: aggregate.window_started_on,
                window_ended_on: aggregate.window_ended_on,
                window_timezone: aggregate.window_timezone.to_string(),
                ingested_at: aggregate.ingested_at,
                coverage,
            },
        ];
        let pool = context.pool.clone();
        let source_service = context.intelligence_sources;
        let report = WhatsAppInsightsReport {
            source: "whatsapp_consent",
            account_id: aggregate.account_id,
            window_started_on: aggregate.window_started_on,
            window_ended_on: aggregate.window_ended_on,
            window_timezone: aggregate.window_timezone,
            ingested_at: aggregate.ingested_at,
            coverage,
            rows_received: observations.len(),
            inbound_events: aggregate.inbound_events,
            consented_conversations: aggregate.consented_conversations,
        };
        stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move {
                source_service
                    .record_snapshot(
                        &mut conn,
                        "whatsapp_consent",
                        &report.account_id,
                        &observations,
                    )
                    .await?;
                Ok(report)
            })
        })
        .await
        .map_err(map_error)?
    } else {
        WhatsAppInsightsReport {
            source: "whatsapp_consent",
            account_id: query.account_id,
            window_started_on: start,
            window_ended_on: end
                .checked_add_days(chrono::Days::new(1))
                .ok_or_else(whatsapp_insights_error)?,
            window_timezone: "UTC",
            ingested_at: chrono::Utc::now(),
            coverage: stheno_ai::MetricCoverage::Unavailable,
            rows_received: 0,
            inbound_events: 0,
            consented_conversations: 0,
        }
    };
    Ok((StatusCode::OK, Json(json!({ "whatsapp_insights": report }))))
}

fn whatsapp_insights_error() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new(
            "invalid_whatsapp_insights_window",
            "The WhatsApp insights date window is invalid.",
        )),
    )
        .into_response()
}

fn meta_insights_error_response(error: stheno_meta_insights::MetaInsightsError) -> Response {
    let (status, kind, message) = match error {
        stheno_meta_insights::MetaInsightsError::InvalidRequest => (
            StatusCode::BAD_REQUEST,
            "invalid_window",
            "The Meta insights request is invalid.",
        ),
        stheno_meta_insights::MetaInsightsError::CredentialsRejected => (
            StatusCode::UNAUTHORIZED,
            "source_credentials_rejected",
            "The Meta account credentials were rejected.",
        ),
        stheno_meta_insights::MetaInsightsError::PermissionDenied => (
            StatusCode::FORBIDDEN,
            "source_permission_denied",
            "The connected Meta account lacks the required insights permission.",
        ),
        stheno_meta_insights::MetaInsightsError::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            "source_rate_limited",
            "Meta is rate limiting this insights request.",
        ),
        stheno_meta_insights::MetaInsightsError::ProviderUnavailable => (
            StatusCode::BAD_GATEWAY,
            "source_unavailable",
            "Meta insights are temporarily unavailable.",
        ),
        stheno_meta_insights::MetaInsightsError::MalformedResponse
        | stheno_meta_insights::MetaInsightsError::RequestFailed => (
            StatusCode::BAD_GATEWAY,
            "source_request_failed",
            "Meta returned an unusable insights response.",
        ),
    };
    (status, Json(ErrorResponse::new(kind, message))).into_response()
}

fn search_console_error_response(error: stheno_search_console::SearchConsoleError) -> Response {
    let (status, code, message) = match error {
        stheno_search_console::SearchConsoleError::InvalidRequest => (
            StatusCode::BAD_REQUEST,
            "invalid_window",
            "The Search Console date window is invalid.",
        ),
        stheno_search_console::SearchConsoleError::CredentialsRejected
        | stheno_search_console::SearchConsoleError::PermissionDenied => (
            StatusCode::CONFLICT,
            "source_authorization_required",
            "The saved Google authorization is expired or lacks read access to this property.",
        ),
        stheno_search_console::SearchConsoleError::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            "source_rate_limited",
            "Google Search Console is temporarily rate limiting this property.",
        ),
        stheno_search_console::SearchConsoleError::ProviderUnavailable => (
            StatusCode::BAD_GATEWAY,
            "source_unavailable",
            "Google Search Console is temporarily unavailable.",
        ),
        stheno_search_console::SearchConsoleError::MalformedResponse
        | stheno_search_console::SearchConsoleError::RequestFailed => (
            StatusCode::BAD_GATEWAY,
            "source_request_failed",
            "Search Console data could not be retrieved safely.",
        ),
    };
    (status, Json(ErrorResponse::new(code, message))).into_response()
}

pub async fn list_chat_offers(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let pool = context.pool.clone();
    let offers = context.chat_offers;
    let rows = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { offers.list(&mut conn).await })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "chat_offers": rows }))))
}

pub async fn save_chat_offer(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Json(input): Json<SaveChatOffer>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let currency = stheno_currency::CurrencyService::new(context.pool.clone())
        .decimal_digits(&input.currency_code)
        .await
        .map_err(map_error)?;
    let price = rust_decimal::Decimal::from_str(&input.chat_unit_price).map_err(|_| {
        map_error(stheno_types::Error::Invalid {
            code: "ai_chat_offer_invalid",
            message: "Chat price must be a positive decimal string.".to_string(),
            field: Some("chat_unit_price".to_string()),
        })
    })?;
    if price.normalize().scale() > currency as u32 {
        return Err(map_error(stheno_types::Error::Invalid {
            code: "ai_chat_offer_precision_invalid",
            message: format!(
                "Currency {} allows at most {currency} decimal places.",
                input.currency_code
            ),
            field: Some("chat_unit_price".to_string()),
        }));
    }
    let pool = context.pool.clone();
    let offers = context.chat_offers;
    let actor_id = claims.actor_id.clone().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "not_allowed",
                "Only a store owner or admin can manage this AI setting.",
            )),
        )
            .into_response()
    })?;
    let offer = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { offers.save_with_actor(&mut conn, input, &actor_id).await })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "chat_offer": offer }))))
}

pub async fn delete_chat_offer(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let pool = context.pool.clone();
    let offers = context.chat_offers;
    let actor_id = claims.actor_id.clone().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "not_allowed",
                "Only a store owner or admin can manage this AI setting.",
            )),
        )
            .into_response()
    })?;
    stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { offers.delete_with_actor(&mut conn, &id, &actor_id).await })
    })
    .await
    .map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Returns deterministic, privacy-safe product and category trends from this tenant's orders.
pub async fn get_store_intelligence(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Query(query): Query<StoreIntelligenceQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let generated_at = chrono::Utc::now();
    let signals = context
        .order_service
        .store_intelligence_order_signals(
            generated_at - chrono::Duration::days(7),
            generated_at - chrono::Duration::days(14),
            generated_at,
        )
        .await
        .map_err(map_error)?;
    let category_signals = context
        .order_service
        .store_intelligence_category_signals(
            generated_at - chrono::Duration::days(7),
            generated_at - chrono::Duration::days(14),
            generated_at,
        )
        .await
        .map_err(map_error)?;
    let mut report = stheno_ai::build_store_intelligence_report(
        signals
            .into_iter()
            .map(|signal| StoreOrderObservation {
                product_id: signal.product_id,
                product_name: signal.product_name,
                category: signal.category,
                current_units: signal.current_units,
                comparison_units: signal.comparison_units,
                current_orders: signal.current_orders,
                comparison_orders: signal.comparison_orders,
            })
            .collect(),
        category_signals
            .into_iter()
            .map(|signal| StoreCategoryObservation {
                category: signal.category,
                current_units: signal.current_units,
                comparison_units: signal.comparison_units,
                current_orders: signal.current_orders,
                comparison_orders: signal.comparison_orders,
            })
            .collect(),
        generated_at,
    );
    for recommendation in &mut report.recommendations {
        recommendation.id = format!(
            "{}:{}:{}:{}",
            report
                .current_window_started_at
                .timestamp()
                .div_euclid(86_400),
            recommendation.subject_type,
            recommendation.subject_id,
            recommendation.action,
        );
        let query: String = recommendation
            .subject_label
            .chars()
            .filter(|character| character.is_alphanumeric() || character.is_whitespace())
            .take(200)
            .collect();
        if query.trim().is_empty() {
            recommendation.knowledge_lookup = "no_search_terms";
            continue;
        }
        let knowledge = context.knowledge;
        let pool = context.pool.clone();
        let results = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
            Box::pin(async move { knowledge.search(&mut conn, &query, 3).await })
        })
        .await;
        match results {
            Ok(hits) => {
                recommendation.knowledge_lookup = if hits.is_empty() {
                    "no_match"
                } else {
                    "matched"
                };
                recommendation.knowledge_sources = hits
                    .iter()
                    .map(|hit| KnowledgeEvidenceReference {
                        entry_id: hit.entry.id.clone(),
                        title: hit.entry.title.clone(),
                        updated_at: hit.entry.updated_at,
                    })
                    .collect();
                recommendation.knowledge_evidence = hits
                    .into_iter()
                    .map(|hit| stheno_ai::KnowledgeEvidenceSnippet {
                        source_id: hit.entry.id,
                        excerpt: hit.entry.content.chars().take(1200).collect(),
                    })
                    .collect();
            }
            Err(error) => {
                tracing::warn!(error = %error, "Store intelligence knowledge retrieval unavailable");
                recommendation.knowledge_lookup = "unavailable";
            }
        }
    }
    let reviews = context.intelligence_reviews;
    let pool = context.pool.clone();
    let review_states = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { reviews.list(&mut conn).await })
    })
    .await
    .map_err(map_error)?;
    for recommendation in &mut report.recommendations {
        recommendation.review_decision = review_states
            .iter()
            .find(|review| review.recommendation_id == recommendation.id)
            .map(|review| review.decision.clone());
    }
    let model_ranking = if query.rank_local {
        match context.service.load_local_ranker().await {
            Ok(Some(config)) => {
                let provider_config = stheno_ai_openai_compatible::OpenAiCompatibleConfig {
                    endpoint: config.endpoint,
                    model: config.model.clone(),
                    api_key: config.api_key,
                };
                match stheno_ai_openai_compatible::OpenAiCompatibleProvider::new_local(
                    provider_config,
                ) {
                    Ok(provider) => match try_local_ranking_slot() {
                        Ok(_permit) => Some(
                            stheno_ai::rank_store_options(
                                &provider,
                                &config.model,
                                &report.recommendations,
                            )
                            .await,
                        ),
                        Err(_) => Some(StoreIntelligenceRanking {
                            status: "abstained",
                            ranked: Vec::new(),
                            evidence_citations: Vec::new(),
                            abstention_reason: Some("local_model_busy"),
                        }),
                    },
                    Err(error) => {
                        tracing::warn!(error = %error, "Local Store Intelligence model unavailable");
                        Some(StoreIntelligenceRanking {
                            status: "abstained",
                            ranked: Vec::new(),
                            evidence_citations: Vec::new(),
                            abstention_reason: Some("local_model_unavailable"),
                        })
                    }
                }
            }
            Ok(None) => Some(StoreIntelligenceRanking {
                status: "abstained",
                ranked: Vec::new(),
                evidence_citations: Vec::new(),
                abstention_reason: Some("local_model_not_configured"),
            }),
            Err(error) => {
                tracing::warn!(error = %error, "Local Store Intelligence model configuration unavailable");
                Some(StoreIntelligenceRanking {
                    status: "abstained",
                    ranked: Vec::new(),
                    evidence_citations: Vec::new(),
                    abstention_reason: Some("local_model_unavailable"),
                })
            }
        }
    } else {
        None
    };
    Ok((
        StatusCode::OK,
        Json(json!({ "store_intelligence": report, "model_ranking": model_ranking })),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRecommendationRequest {
    decision: String,
    request_id: String,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ReviewHistoryQuery {
    limit: Option<i32>,
}

/// Saves an owner review state without executing the recommended commerce action.
pub async fn review_store_intelligence_recommendation(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Path(recommendation_id): Path<String>,
    Json(input): Json<ReviewRecommendationRequest>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let actor_id = claims.actor_id.clone().ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "not_allowed",
                "A store owner or admin is required.",
            )),
        )
            .into_response()
    })?;
    let reviews = context.intelligence_reviews;
    let pool = context.pool.clone();
    let review = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move {
            reviews
                .record(
                    &mut conn,
                    &recommendation_id,
                    &input.decision,
                    &actor_id,
                    &input.request_id,
                )
                .await
        })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "review": review }))))
}

/// Returns a bounded, owner-only history of review decisions for a recommendation.
pub async fn list_store_intelligence_review_history(
    State(context): State<AiAdminContext>,
    Extension(claims): Extension<JwtClaims>,
    Path(recommendation_id): Path<String>,
    Query(query): Query<ReviewHistoryQuery>,
) -> Result<impl IntoResponse, Response> {
    require_store_owner_or_admin(&context, &claims).await?;
    let limit = query.limit.unwrap_or(20);
    if !(1..=50).contains(&limit) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_data",
                "Review history limit must be between 1 and 50.",
            )),
        )
            .into_response());
    }
    let reviews = context.intelligence_reviews;
    let pool = context.pool.clone();
    let history = stheno_db::tenant::with_tenant_conn(&pool, move |mut conn| {
        Box::pin(async move { reviews.history(&mut conn, &recommendation_id, limit).await })
    })
    .await
    .map_err(map_error)?;
    Ok((StatusCode::OK, Json(json!({ "history": history }))))
}

async fn require_store_owner_or_admin(
    context: &AiAdminContext,
    claims: &JwtClaims,
) -> Result<String, Response> {
    let Some(actor_id) = claims.actor_id.as_deref() else {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "not_allowed",
                "Only a store owner or admin can manage this AI setting.",
            )),
        )
            .into_response());
    };
    let user = context.users.get_user(actor_id).await.map_err(map_error)?;
    if !can_manage_store_ai(user.role.as_str()) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "not_allowed",
                "Only a store owner or admin can manage this AI setting.",
            )),
        )
            .into_response());
    }
    Ok(actor_id.to_string())
}

fn can_manage_store_ai(role: &str) -> bool {
    matches!(role, "owner" | "admin")
}

fn search_console_oauth_environment() -> Result<(String, String, String), Response> {
    let client_id = std::env::var("STHENO_SEARCH_CONSOLE_CLIENT_ID")
        .ok()
        .filter(|value| {
            !value.trim().is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
        })
        .ok_or_else(oauth_configuration_error)?;
    let client_secret = std::env::var("STHENO_SEARCH_CONSOLE_CLIENT_SECRET")
        .ok()
        .filter(|value| {
            !value.trim().is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
        })
        .ok_or_else(oauth_configuration_error)?;
    let redirect_uri = std::env::var("STHENO_SEARCH_CONSOLE_REDIRECT_URI")
        .ok()
        .filter(|value| valid_search_console_redirect_uri(value))
        .ok_or_else(oauth_configuration_error)?;
    Ok((client_id, client_secret, redirect_uri))
}

fn valid_search_console_redirect_uri(value: &str) -> bool {
    if value.len() > 2048 || value.chars().any(char::is_control) {
        return false;
    }
    url::Url::parse(value).is_ok_and(|url| {
        let secure = url.scheme() == "https";
        let local = url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
        (secure || local)
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}

fn sign_search_console_oauth_state(
    state: SearchConsoleOAuthState,
    secret: &str,
) -> Result<String, ()> {
    let payload = serde_json::to_vec(&state).map_err(|_| ())?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| ())?;
    mac.update(&payload);
    Ok(format!(
        "{}.{}",
        hex::encode(&payload),
        hex::encode(mac.finalize().into_bytes())
    ))
}

fn verify_search_console_oauth_state(
    state: &str,
    secret: &str,
) -> Result<SearchConsoleOAuthState, ()> {
    let (payload_hex, signature_hex) = state.split_once('.').ok_or(())?;
    let payload = hex::decode(payload_hex).map_err(|_| ())?;
    let signature = hex::decode(signature_hex).map_err(|_| ())?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| ())?;
    mac.update(&payload);
    mac.verify_slice(&signature).map_err(|_| ())?;
    let claims: SearchConsoleOAuthState = serde_json::from_slice(&payload).map_err(|_| ())?;
    if claims.nonce.len() != 26
        || claims.actor_id.is_empty()
        || claims.tenant_id.is_empty()
        || claims.property.is_empty()
    {
        return Err(());
    }
    Ok(claims)
}

fn oauth_configuration_error() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse::new(
            "search_console_oauth_unavailable",
            "Search Console OAuth is not configured on this deployment.",
        )),
    )
        .into_response()
}

fn oauth_state_error() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new(
            "search_console_oauth_state_invalid",
            "The Search Console authorization could not be verified. Start a new connection.",
        )),
    )
        .into_response()
}

fn map_error(error: stheno_types::Error) -> Response {
    let (status, kind) = match error {
        stheno_types::Error::Invalid { .. } => (StatusCode::BAD_REQUEST, "invalid_data"),
        stheno_types::Error::NotFound { .. } => (StatusCode::NOT_FOUND, "not_found"),
        stheno_types::Error::Conflict { .. } => (StatusCode::CONFLICT, "conflict"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "unexpected_state"),
    };
    (
        status,
        Json(ErrorResponse::new(kind, "AI configuration request failed.")),
    )
        .into_response()
}

#[cfg(test)]
mod authorization_tests {
    use super::{
        can_manage_store_ai, sign_search_console_oauth_state, try_local_ranking_slot,
        valid_search_console_redirect_uri, verify_search_console_oauth_state,
        SearchConsoleOAuthState, LOCAL_RANKING_SLOT,
    };

    #[test]
    fn store_intelligence_management_is_limited_to_owner_and_admin_roles() {
        for role in ["owner", "admin"] {
            assert!(can_manage_store_ai(role), "{role} should be allowed");
        }
        for role in ["staff", "manager", "operator", "customer", ""] {
            assert!(!can_manage_store_ai(role), "{role} should be denied");
        }
    }

    #[test]
    fn search_console_oauth_state_is_signed_and_redirect_uri_is_trusted() {
        let state = SearchConsoleOAuthState {
            tenant_id: "tenant-a".to_string(),
            actor_id: "user-a".to_string(),
            property: "sc-domain:shop.example.com".to_string(),
            nonce: "01JBCDEFGHJKMNPQRSTVWXYZ12".to_string(),
            issued_at: 1_798_000_000,
        };
        let token = sign_search_console_oauth_state(state, "test-client-secret").unwrap();
        assert_eq!(
            verify_search_console_oauth_state(&token, "test-client-secret")
                .unwrap()
                .tenant_id,
            "tenant-a"
        );
        assert!(verify_search_console_oauth_state(&token, "wrong-secret").is_err());
        assert!(valid_search_console_redirect_uri(
            "https://admin.example.com/app/ai"
        ));
        assert!(valid_search_console_redirect_uri(
            "http://localhost:9000/app/ai"
        ));
        assert!(!valid_search_console_redirect_uri(
            "https://admin.example.com/app/ai?next=x"
        ));
        assert!(!valid_search_console_redirect_uri(
            "http://attacker.example/app/ai"
        ));
    }

    #[test]
    fn concurrent_local_rankings_are_rejected_and_slot_recovers() {
        let semaphore = LOCAL_RANKING_SLOT.get_or_init(|| tokio::sync::Semaphore::new(1));
        let held = semaphore
            .try_acquire()
            .expect("first request reserves slot");
        assert!(try_local_ranking_slot().is_err());
        drop(held);
        assert!(try_local_ranking_slot().is_ok());
    }
}
