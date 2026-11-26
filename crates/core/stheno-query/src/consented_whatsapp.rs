//! Cross-schema read for consent-filtered WhatsApp event totals (ADR-035).

use chrono::{DateTime, Days, NaiveDate, TimeZone, Utc};
use serde::Serialize;
use stheno_db::diesel::sql_types::{BigInt, Jsonb, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{DbPool, Error, Result};

const MAX_WINDOW_DAYS: i64 = 92;
const MAX_CONSENTED_RECIPIENTS: usize = 5000;

/// Aggregate totals for persisted inbound events linked to currently consented senders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentedWhatsAppAggregate {
    pub account_id: String,
    pub window_started_on: NaiveDate,
    pub window_ended_on: NaiveDate,
    pub window_timezone: &'static str,
    pub ingested_at: DateTime<Utc>,
    pub inbound_events: i64,
    pub consented_conversations: i64,
}

#[derive(QueryableByName)]
struct ConsentRow {
    #[diesel(sql_type = Text)]
    recipient_hash: String,
    #[diesel(sql_type = Timestamptz)]
    consented_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct AggregateRow {
    #[diesel(sql_type = BigInt)]
    inbound_events: i64,
    #[diesel(sql_type = BigInt)]
    consented_conversations: i64,
}

#[derive(Serialize)]
struct EligibleSender {
    conversation_hash: String,
    consented_at: DateTime<Utc>,
}

/// Counts processed WhatsApp inbound events only for senders whose tenant consent is active.
///
/// The query returns no phone number or sender hash. A caller must label non-empty results as
/// partial because the local event ledger cannot prove complete platform webhook delivery.
pub async fn query_consented_whatsapp_aggregate(
    pool: &DbPool,
    tenant_id: &str,
    account_id: &str,
    start_date: NaiveDate,
    end_date_inclusive: NaiveDate,
) -> Result<Option<ConsentedWhatsAppAggregate>> {
    stheno_db::tenant::validate(tenant_id)?;
    if account_id.trim().is_empty()
        || account_id.len() > 512
        || account_id.chars().any(char::is_control)
    {
        return Err(Error::Invalid {
            code: "invalid_whatsapp_insights_request",
            message: "WhatsApp account identifier is invalid.".to_string(),
            field: Some("account_id".to_string()),
        });
    }
    let day_count = end_date_inclusive
        .signed_duration_since(start_date)
        .num_days()
        .saturating_add(1);
    let today_utc = Utc::now().date_naive();
    let latest_end = today_utc
        .checked_add_days(Days::new(1))
        .ok_or_else(invalid_window)?;
    if day_count <= 0
        || day_count > MAX_WINDOW_DAYS
        || end_date_inclusive > today_utc
        || start_date < NaiveDate::from_ymd_opt(2000, 1, 1).ok_or_else(invalid_window)?
    {
        return Err(invalid_window());
    }
    let window_ended_on = end_date_inclusive
        .checked_add_days(Days::new(1))
        .filter(|date| *date <= latest_end)
        .ok_or_else(invalid_window)?;
    let start_at =
        Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).ok_or_else(invalid_window)?);
    let end_at = Utc.from_utc_datetime(
        &window_ended_on
            .and_hms_opt(0, 0, 0)
            .ok_or_else(invalid_window)?,
    );
    let tenant = tenant_id.to_string();
    let account_id = account_id.to_string();
    let pool = pool.clone();
    stheno_db::tenant::scope(tenant, async move {
        stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
            Box::pin(async move {
                let consent_rows = diesel::sql_query(
                    "SELECT recipient_hash, consented_at FROM notification.whatsapp_analytics_consent \
                     WHERE consented = true AND revoked_at IS NULL \
                     ORDER BY recipient_hash LIMIT $1",
                )
                .bind::<BigInt, _>((MAX_CONSENTED_RECIPIENTS + 1) as i64)
                .load::<ConsentRow>(conn)
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
                if consent_rows.len() > MAX_CONSENTED_RECIPIENTS {
                    return Err(Error::Invalid {
                        code: "whatsapp_insights_recipient_limit",
                        message: "Too many active consent records for bounded aggregation."
                            .to_string(),
                        field: None,
                    });
                }
                if consent_rows.is_empty() {
                    return Ok(None);
                }
                let eligible: Vec<EligibleSender> = consent_rows
                    .into_iter()
                    .map(|row| EligibleSender {
                        conversation_hash: row.recipient_hash,
                        consented_at: row.consented_at,
                    })
                    .collect();
                let eligible = serde_json::to_value(eligible)
                    .map_err(|_| Error::Invalid {
                        code: "whatsapp_insights_consent_invalid",
                        message: "Consent records could not be safely aggregated.".to_string(),
                        field: None,
                    })?;
                let totals = diesel::sql_query(
                    "SELECT count(*)::bigint AS inbound_events, \
                            count(DISTINCT event.conversation_key)::bigint AS consented_conversations \
                     FROM ai.channel_event AS event \
                     WHERE event.channel = 'whatsapp' AND event.external_account_id = $1 \
                       AND event.state = 'processed' \
                       AND event.provider_created_at >= $2 AND event.provider_created_at < $3 \
                       AND EXISTS ( \
                         SELECT 1 FROM jsonb_array_elements($4::jsonb) AS consent(value) \
                          WHERE consent.value->>'conversation_hash' = event.conversation_key \
                            AND event.provider_created_at >= (consent.value->>'consented_at')::timestamptz \
                       )",
                )
                .bind::<Text, _>(account_id.clone())
                .bind::<Timestamptz, _>(start_at)
                .bind::<Timestamptz, _>(end_at)
                .bind::<Jsonb, _>(eligible)
                .get_result::<AggregateRow>(conn)
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
                if totals.inbound_events <= 0 {
                    return Ok(None);
                }
                Ok(Some(ConsentedWhatsAppAggregate {
                    account_id,
                    window_started_on: start_date,
                    window_ended_on,
                    window_timezone: "UTC",
                    ingested_at: Utc::now(),
                    inbound_events: totals.inbound_events,
                    consented_conversations: totals.consented_conversations,
                }))
            })
        })
        .await
    })
    .await
}

fn invalid_window() -> Error {
    Error::Invalid {
        code: "invalid_whatsapp_insights_window",
        message: "WhatsApp insights date window must be a valid past range of at most 92 days."
            .to_string(),
        field: Some("date_window".to_string()),
    }
}
