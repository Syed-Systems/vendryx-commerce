//! Cross-module read model for tenant Store Intelligence exports (ADR-035).

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use stheno_db::diesel::sql_types::{BigInt, Bool, Date, Nullable, Numeric, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{DbPool, Error, Result};

/// Tenant-owned source data assembled across AI and notification schemas.
#[derive(Debug, Serialize)]
pub struct TenantStoreIntelligenceExport {
    pub connections: Vec<SourceConnectionExport>,
    pub observations: Vec<SourceObservationExport>,
    pub connection_audit: Vec<SourceConnectionAuditExport>,
    pub recommendation_reviews: Vec<RecommendationReviewExport>,
    pub recommendation_review_audit: Vec<RecommendationReviewAuditExport>,
    pub whatsapp_analytics_consents: Vec<WhatsAppAnalyticsConsentExport>,
}

#[derive(Debug, Serialize)]
pub struct SourceConnectionExport {
    pub source: String,
    pub account_id: String,
    pub configured: bool,
    pub enabled: bool,
    pub version: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SourceObservationExport {
    pub source: String,
    pub account_id: String,
    pub scope: String,
    pub metric: String,
    pub definition: String,
    pub unit: String,
    pub value: Decimal,
    pub denominator: Option<Decimal>,
    pub sample_size: Option<i64>,
    pub window_started_on: NaiveDate,
    pub window_ended_on: NaiveDate,
    pub window_timezone: String,
    pub ingested_at: DateTime<Utc>,
    pub coverage: String,
}

#[derive(Debug, Serialize)]
pub struct SourceConnectionAuditExport {
    pub source: String,
    pub account_id: String,
    pub action: String,
    pub actor_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationReviewExport {
    pub recommendation_id: String,
    pub decision: String,
    pub version: i32,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationReviewAuditExport {
    pub recommendation_id: String,
    pub request_id: String,
    pub decision: String,
    pub actor_id: String,
    pub evidence_source: String,
    pub rule_version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WhatsAppAnalyticsConsentExport {
    pub recipient_hash: String,
    pub customer_id: Option<String>,
    pub source_event_id: String,
    pub consented: bool,
    pub consent_source: String,
    pub consented_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(QueryableByName)]
struct ConnectionRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Bool)]
    has_credentials: bool,
    #[diesel(sql_type = Bool)]
    enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    version: i32,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct ObservationRow {
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    scope: String,
    #[diesel(sql_type = Text)]
    metric: String,
    #[diesel(sql_type = Text)]
    definition: String,
    #[diesel(sql_type = Text)]
    unit: String,
    #[diesel(sql_type = Numeric)]
    value: Decimal,
    #[diesel(sql_type = Nullable<Numeric>)]
    denominator: Option<Decimal>,
    #[diesel(sql_type = Nullable<BigInt>)]
    sample_size: Option<i64>,
    #[diesel(sql_type = Date)]
    window_started_on: NaiveDate,
    #[diesel(sql_type = Date)]
    window_ended_on: NaiveDate,
    #[diesel(sql_type = Text)]
    window_timezone: String,
    #[diesel(sql_type = Timestamptz)]
    ingested_at: DateTime<Utc>,
    #[diesel(sql_type = Text)]
    coverage: String,
}

#[derive(QueryableByName)]
struct AuditRow {
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    action: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct WhatsAppConsentRow {
    #[diesel(sql_type = Text)]
    recipient_hash: String,
    #[diesel(sql_type = Nullable<Text>)]
    customer_id: Option<String>,
    #[diesel(sql_type = Text)]
    source_event_id: String,
    #[diesel(sql_type = Bool)]
    consented: bool,
    #[diesel(sql_type = Text)]
    consent_source: String,
    #[diesel(sql_type = Timestamptz)]
    consented_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    revoked_at: Option<DateTime<Utc>>,
}

#[derive(QueryableByName)]
struct RecommendationReviewRow {
    #[diesel(sql_type = Text)]
    recommendation_id: String,
    #[diesel(sql_type = Text)]
    decision: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    version: i32,
    #[diesel(sql_type = Timestamptz)]
    changed_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct RecommendationReviewAuditRow {
    #[diesel(sql_type = Text)]
    recommendation_id: String,
    #[diesel(sql_type = Text)]
    request_id: String,
    #[diesel(sql_type = Text)]
    decision: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Text)]
    evidence_source: String,
    #[diesel(sql_type = Text)]
    rule_version: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

/// Reads one tenant's non-secret source data under its RLS role and assembles the export.
pub async fn query_tenant_store_intelligence_export(
    pool: &DbPool,
    tenant_id: &str,
) -> Result<TenantStoreIntelligenceExport> {
    stheno_db::tenant::validate(tenant_id)?;
    let pool = pool.clone();
    stheno_db::tenant::scope(tenant_id.to_string(), async move {
        stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                let connections = diesel::sql_query(
                    "SELECT kind, account_id, has_credentials, enabled, version, updated_at \
                     FROM ai.configuration \
                     WHERE kind IN ('search_console', 'meta_insights_facebook', 'meta_insights_instagram') \
                     ORDER BY kind, account_id",
                )
                .load::<ConnectionRow>(conn)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| SourceConnectionExport {
                    source: source_for_kind(&row.kind).to_string(),
                    account_id: row.account_id,
                    configured: row.has_credentials,
                    enabled: row.enabled,
                    version: row.version,
                    updated_at: row.updated_at,
                })
                .collect();

                let observations = diesel::sql_query(
                    "SELECT source, account_id, scope, metric, definition, unit, value, denominator, sample_size, \
                     window_started_on, window_ended_on, window_timezone, ingested_at, coverage \
                     FROM ai.store_intelligence_source_observation \
                     ORDER BY ingested_at DESC, source, metric",
                )
                .load::<ObservationRow>(conn)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| SourceObservationExport {
                    source: row.source,
                    account_id: row.account_id,
                    scope: row.scope,
                    metric: row.metric,
                    definition: row.definition,
                    unit: row.unit,
                    value: row.value,
                    denominator: row.denominator,
                    sample_size: row.sample_size,
                    window_started_on: row.window_started_on,
                    window_ended_on: row.window_ended_on,
                    window_timezone: row.window_timezone,
                    ingested_at: row.ingested_at,
                    coverage: row.coverage,
                })
                .collect();

                let connection_audit = diesel::sql_query(
                    "SELECT source, account_id, action, actor_id, created_at \
                     FROM ai.source_connection_audit ORDER BY created_at DESC",
                )
                .load::<AuditRow>(conn)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| SourceConnectionAuditExport {
                    source: row.source,
                    account_id: row.account_id,
                    action: row.action,
                    actor_id: row.actor_id,
                    created_at: row.created_at,
                })
                .collect();

                let recommendation_reviews = diesel::sql_query(
                    "SELECT recommendation_id, decision, version, changed_at \
                     FROM ai.store_intelligence_review ORDER BY changed_at DESC, recommendation_id",
                )
                .load::<RecommendationReviewRow>(conn)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| RecommendationReviewExport {
                    recommendation_id: row.recommendation_id,
                    decision: row.decision,
                    version: row.version,
                    changed_at: row.changed_at,
                })
                .collect();

                let recommendation_review_audit = diesel::sql_query(
                    "SELECT recommendation_id, request_id, decision, actor_id, evidence_source, rule_version, created_at \
                     FROM ai.store_intelligence_review_audit ORDER BY created_at DESC, id",
                )
                .load::<RecommendationReviewAuditRow>(conn)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| RecommendationReviewAuditExport {
                    recommendation_id: row.recommendation_id,
                    request_id: row.request_id,
                    decision: row.decision,
                    actor_id: row.actor_id,
                    evidence_source: row.evidence_source,
                    rule_version: row.rule_version,
                    created_at: row.created_at,
                })
                .collect();

                let whatsapp_analytics_consents = diesel::sql_query(
                    "SELECT recipient_hash, customer_id, source_event_id, consented, consent_source, \
                     consented_at, revoked_at FROM notification.whatsapp_analytics_consent \
                     ORDER BY consented_at DESC, recipient_hash",
                )
                .load::<WhatsAppConsentRow>(conn)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|row| WhatsAppAnalyticsConsentExport {
                    recipient_hash: row.recipient_hash,
                    customer_id: row.customer_id,
                    source_event_id: row.source_event_id,
                    consented: row.consented,
                    consent_source: row.consent_source,
                    consented_at: row.consented_at,
                    revoked_at: row.revoked_at,
                })
                .collect();

                Ok(TenantStoreIntelligenceExport {
                    connections,
                    observations,
                    connection_audit,
                    recommendation_reviews,
                    recommendation_review_audit,
                    whatsapp_analytics_consents,
                })
            })
        })
        .await
    })
    .await
}

fn source_for_kind(kind: &str) -> &str {
    match kind {
        "meta_insights_facebook" => "facebook_page",
        "meta_insights_instagram" => "instagram_account",
        _ => "search_console",
    }
}

fn database_error(error: diesel::result::Error) -> Error {
    Error::Database(error.to_string())
}
