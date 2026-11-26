//! Tenant-scoped message event deduplication and out-of-order suppression.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use stheno_db::diesel::sql_types::{Bool, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{DbPool, Error, Result};

/// Durable event-key store. It retains no message content or direct sender identifier.
#[derive(Clone)]
pub struct WebhookEventService {
    pool: DbPool,
}

impl WebhookEventService {
    /// Creates the service over the application pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Atomically claims an in-order message event for processing.
    ///
    /// `sender_id` is hashed before persistence. Deduplication is tenant/account/channel
    /// scoped. Older messages arriving after a newer message for the same conversation are
    /// recorded for replay suppression but return `false` for processing. Failed or expired
    /// claims can be retried, while processed claims can never be claimed again.
    pub async fn claim_message(
        &self,
        tenant_id: &str,
        channel: &str,
        account_id: &str,
        event_id: &str,
        sender_id: &str,
        provider_created_at: DateTime<Utc>,
    ) -> Result<bool> {
        stheno_db::tenant::validate(tenant_id)?;
        if !matches!(channel, "whatsapp" | "messenger" | "instagram")
            || !bounded_id(account_id, 128)
            || !bounded_id(event_id, 256)
            || !bounded_id(sender_id, 256)
        {
            return Err(Error::Invalid {
                code: "invalid_meta_event",
                message: "Meta event identifiers are malformed or oversized.".to_string(),
                field: None,
            });
        }
        let tenant_id = tenant_id.to_string();
        let channel = channel.to_string();
        let account_id = account_id.to_string();
        let event_id = event_id.to_string();
        let conversation_key = format!("{:x}", Sha256::digest(sender_id.as_bytes()));
        let lock_key = format!("{tenant_id}|{channel}|{account_id}|{conversation_key}");
        let pool = self.pool.clone();

        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    #[derive(QueryableByName)]
                    struct Latest {
                        #[diesel(sql_type = Bool)]
                        is_latest: bool,
                    }
                    diesel::sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                        .bind::<Text, _>(&lock_key)
                        .execute(conn)
                        .await
                        .map_err(|error| Error::Database(error.to_string()))?;
                    let latest = diesel::sql_query(
                        "SELECT NOT EXISTS (SELECT 1 FROM ai.channel_event \
                          WHERE channel = $1 AND external_account_id = $2 \
                            AND conversation_key = $3 AND provider_created_at > $4) AS is_latest",
                    )
                    .bind::<Text, _>(&channel)
                    .bind::<Text, _>(&account_id)
                    .bind::<Text, _>(&conversation_key)
                    .bind::<Timestamptz, _>(provider_created_at)
                    .get_result::<Latest>(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    if !latest.is_latest {
                        diesel::sql_query(
                            "INSERT INTO ai.channel_event \
                             (id, channel, external_account_id, event_id, conversation_key, provider_created_at, state) \
                             VALUES ($1, $2, $3, $4, $5, $6, 'ignored') \
                             ON CONFLICT (tenant_id, channel, external_account_id, event_id) DO NOTHING",
                        )
                        .bind::<Text, _>(format!("aiev_{}", ulid::Ulid::new()))
                        .bind::<Text, _>(&channel)
                        .bind::<Text, _>(&account_id)
                        .bind::<Text, _>(&event_id)
                        .bind::<Text, _>(&conversation_key)
                        .bind::<Timestamptz, _>(provider_created_at)
                        .execute(conn)
                        .await
                        .map_err(|error| Error::Database(error.to_string()))?;
                        return Ok(false);
                    }
                    let claimed = diesel::sql_query(
                        "INSERT INTO ai.channel_event \
                         (id, channel, external_account_id, event_id, conversation_key, provider_created_at, state, processing_until) \
                         VALUES ($1, $2, $3, $4, $5, $6, 'processing', now() + interval '1 minute') \
                         ON CONFLICT (tenant_id, channel, external_account_id, event_id) \
                         DO UPDATE SET state = 'processing', processing_until = now() + interval '1 minute' \
                         WHERE ai.channel_event.state = 'failed' \
                            OR (ai.channel_event.state = 'processing' AND ai.channel_event.processing_until < now())",
                    )
                    .bind::<Text, _>(format!("aiev_{}", ulid::Ulid::new()))
                    .bind::<Text, _>(&channel)
                    .bind::<Text, _>(&account_id)
                    .bind::<Text, _>(&event_id)
                    .bind::<Text, _>(&conversation_key)
                    .bind::<Timestamptz, _>(provider_created_at)
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    Ok(claimed == 1)
                })
            })
            .await
        })
        .await
    }

    /// Marks a successful handoff so retries are acknowledged without reprocessing.
    pub async fn mark_processed(
        &self,
        tenant_id: &str,
        channel: &str,
        account_id: &str,
        event_id: &str,
    ) -> Result<()> {
        self.finish(tenant_id, channel, account_id, event_id, "processed")
            .await
    }

    /// Releases a failed handler claim so a provider retry can be processed again.
    pub async fn mark_failed(
        &self,
        tenant_id: &str,
        channel: &str,
        account_id: &str,
        event_id: &str,
    ) -> Result<()> {
        self.finish(tenant_id, channel, account_id, event_id, "failed")
            .await
    }

    async fn finish(
        &self,
        tenant_id: &str,
        channel: &str,
        account_id: &str,
        event_id: &str,
        state: &'static str,
    ) -> Result<()> {
        stheno_db::tenant::validate(tenant_id)?;
        let tenant_id = tenant_id.to_string();
        let channel = channel.to_string();
        let account_id = account_id.to_string();
        let event_id = event_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let changed = diesel::sql_query(
                        "UPDATE ai.channel_event SET state = $4, processing_until = NULL, \
                         processed_at = CASE WHEN $4 = 'processed' THEN now() ELSE processed_at END \
                         WHERE channel = $1 AND external_account_id = $2 AND event_id = $3 AND state = 'processing'",
                    )
                    .bind::<Text, _>(&channel)
                    .bind::<Text, _>(&account_id)
                    .bind::<Text, _>(&event_id)
                    .bind::<Text, _>(state)
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    if changed == 0 {
                        return Err(Error::NotFound {
                            entity: "ai_channel_event",
                            id: event_id,
                        });
                    }
                    Ok(())
                })
            })
            .await
        })
        .await
    }
}

fn bounded_id(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
