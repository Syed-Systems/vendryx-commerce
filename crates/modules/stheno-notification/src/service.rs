use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::sql_types::{Bool, Date, Integer, Jsonb, Nullable, Text};
use diesel::{sql_query, QueryableByName};
use sha2::{Digest, Sha256};
use stheno_db::diesel::result::OptionalExtension;
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{
    Conn, CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository,
};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateTemplateInput, NotificationDTO, NotificationDeliveryDTO, NotificationTemplateDTO,
    SendNotificationInput,
};
use crate::models::{Notification, NotificationDelivery, NotificationTemplate};
use crate::provider::NotificationProvider;

#[derive(Default)]
struct ProviderRegistry(HashMap<String, Arc<dyn NotificationProvider>>);

#[derive(QueryableByName)]
struct OutboxDelivery {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    idempotency_key: String,
    #[diesel(sql_type = Text)]
    provider_id: String,
    #[diesel(sql_type = Text)]
    to: String,
    #[diesel(sql_type = Text)]
    channel: String,
    #[diesel(sql_type = Text)]
    tenant_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    subject: Option<String>,
    #[diesel(sql_type = Text)]
    body: String,
    #[diesel(sql_type = Nullable<Jsonb>)]
    data: Option<serde_json::Value>,
}

#[derive(QueryableByName)]
struct WhatsAppConsentRow {
    #[diesel(sql_type = Bool)]
    consented: bool,
}

#[derive(QueryableByName)]
struct QuotaPresence {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    present: i32,
}

#[derive(QueryableByName)]
struct QuotaReservationChannel {
    #[diesel(sql_type = Text)]
    channel: String,
}

#[derive(QueryableByName)]
struct DailyUsageRow {
    #[diesel(sql_type = Text)]
    channel: String,
    #[diesel(sql_type = Integer)]
    reserved_count: i32,
    #[diesel(sql_type = Integer)]
    sent_count: i32,
}

/// Usage and configured allowance for one notification channel on the current UTC day.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyNotificationChannelUsage {
    pub sent: u32,
    pub reserved: u32,
    /// `None` means the current plan has no daily cap for this channel.
    pub limit: Option<u32>,
    /// `None` means unlimited. Otherwise this excludes both sent and in-flight reservations.
    pub remaining: Option<u32>,
}

/// Tenant-scoped notification usage for the current UTC calendar day.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyNotificationUsage {
    pub usage_date_utc: NaiveDate,
    pub email: DailyNotificationChannelUsage,
    pub whatsapp: DailyNotificationChannelUsage,
}

#[async_trait]
pub trait NotificationQuotaPolicy: Send + Sync {
    /// Returns a per-day cap for a tenant/channel, or `None` for unlimited delivery.
    async fn daily_limit(&self, tenant_id: &str, channel: &str) -> Result<Option<i32>>;
}

#[derive(Clone)]
pub struct NotificationService {
    pool: DbPool,
    providers: Arc<RwLock<ProviderRegistry>>,
    quota_policy: Arc<RwLock<Option<Arc<dyn NotificationQuotaPolicy>>>>,
    templates: PgCrudRepository<NotificationTemplate>,
    notifications: PgCrudRepository<Notification>,
    deliveries: PgCrudRepository<NotificationDelivery>,
}

/// Read-only consent lookup for providers that must recheck before external dispatch.
#[derive(Clone)]
pub struct WhatsAppConsentChecker {
    pool: DbPool,
}

impl NotificationService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool: pool.clone(),
            providers: Arc::new(RwLock::new(ProviderRegistry(HashMap::new()))),
            quota_policy: Arc::new(RwLock::new(None)),
            templates: PgCrudRepository::new(pool.clone()),
            notifications: PgCrudRepository::new(pool.clone()),
            deliveries: PgCrudRepository::new(pool),
        }
    }

    pub fn register_provider(&self, provider: Arc<dyn NotificationProvider>) {
        let mut reg = self.providers.write().unwrap();
        reg.0.insert(provider.provider_id().to_string(), provider);
    }

    pub fn register_quota_policy(&self, policy: Arc<dyn NotificationQuotaPolicy>) {
        *self.quota_policy.write().unwrap() = Some(policy);
    }

    pub fn whatsapp_consent_checker(&self) -> WhatsAppConsentChecker {
        WhatsAppConsentChecker {
            pool: self.pool.clone(),
        }
    }

    /// Removes the tenant's analytics opt-in ledger during tenant erasure.
    pub async fn erase_whatsapp_analytics_consents(&self, conn: &mut Conn<'_>) -> Result<()> {
        sql_query("DELETE FROM notification.whatsapp_analytics_consent")
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|error| Error::Database(error.to_string()))
    }

    /// Returns live tenant usage and allowance for the current UTC date.
    pub async fn get_daily_usage(&self) -> Result<DailyNotificationUsage> {
        let tenant_id = stheno_db::tenant::current()?;
        stheno_db::tenant::validate(&tenant_id)?;
        let usage_date = Utc::now().date_naive();
        let pool = self.pool.clone();
        let query_tenant = tenant_id.clone();
        let rows = stheno_db::tenant::scope(tenant_id.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    sql_query(
                        "SELECT channel, reserved_count, sent_count \
                           FROM notification.daily_notification_usage \
                          WHERE tenant_id = $1 AND usage_date = $2",
                    )
                    .bind::<Text, _>(&query_tenant)
                    .bind::<Date, _>(usage_date)
                    .load::<DailyUsageRow>(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await?;

        let policy = self
            .quota_policy
            .read()
            .map_err(|_| Error::Database("notification quota policy lock poisoned".to_string()))?
            .clone()
            .ok_or_else(|| {
                Error::Database("notification quota policy is unavailable".to_string())
            })?;
        let email_limit = policy.daily_limit(&tenant_id, "email").await?;
        let whatsapp_limit = policy.daily_limit(&tenant_id, "whatsapp").await?;
        let email_counts = rows
            .iter()
            .find(|row| row.channel == "email")
            .map(|row| (row.sent_count, row.reserved_count))
            .unwrap_or_default();
        let whatsapp_counts = rows
            .iter()
            .find(|row| row.channel == "whatsapp")
            .map(|row| (row.sent_count, row.reserved_count))
            .unwrap_or_default();

        let channel_usage = |(sent, reserved): (i32, i32), limit: Option<i32>| {
            let sent = sent.max(0) as u32;
            let reserved = reserved.max(0) as u32;
            let limit = limit.map(|value| value.max(0) as u32);
            let remaining = limit.map(|value| value.saturating_sub(sent.saturating_add(reserved)));
            DailyNotificationChannelUsage {
                sent,
                reserved,
                limit,
                remaining,
            }
        };

        Ok(DailyNotificationUsage {
            usage_date_utc: usage_date,
            email: channel_usage(email_counts, email_limit),
            whatsapp: channel_usage(whatsapp_counts, whatsapp_limit),
        })
    }

    fn find_provider_for_channel(&self, channel: &str) -> Result<Arc<dyn NotificationProvider>> {
        let reg = self.providers.read().unwrap();
        for provider in reg.0.values() {
            if provider.supported_channels().contains(&channel) {
                return Ok(provider.clone());
            }
        }
        Err(Error::NotFound {
            entity: "notification_provider_for_channel",
            id: channel.to_string(),
        })
    }

    // ==========================================
    // 1. Templates
    // ==========================================

    pub async fn create_template(
        &self,
        input: CreateTemplateInput,
    ) -> Result<NotificationTemplateDTO> {
        let id = Id::new();
        let now = Utc::now();

        let template = NotificationTemplate {
            id,
            name: input.name,
            channel: input.channel,
            subject_template: input.subject_template,
            body_template: input.body_template,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.templates.create(None, template).await?;

        Ok(NotificationTemplateDTO {
            id: created.id.to_string(),
            name: created.name,
            channel: created.channel,
            subject_template: created.subject_template,
            body_template: created.body_template,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn get_template_by_name(&self, name: &str) -> Result<NotificationTemplateDTO> {
        let filter = FilterSpec::new()
            .with_predicate("name", FilterOp::Eq(name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.templates.list(None, &filter).await?;
        let t = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "notification_template",
            id: name.to_string(),
        })?;

        Ok(NotificationTemplateDTO {
            id: t.id.to_string(),
            name: t.name,
            channel: t.channel,
            subject_template: t.subject_template,
            body_template: t.body_template,
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
    }

    fn render_text(template: &str, data: Option<&serde_json::Value>) -> String {
        let mut rendered = template.to_string();
        if let Some(serde_json::Value::Object(map)) = data {
            for (k, v) in map {
                let placeholder = format!("{{{{{k}}}}}");
                let val_str = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                rendered = rendered.replace(&placeholder, &val_str);
            }
        }
        rendered
    }

    // ==========================================
    // 2. Notifications & Delivery
    // ==========================================

    pub async fn send_notification(&self, input: SendNotificationInput) -> Result<NotificationDTO> {
        self.send_notification_idempotent(input, None).await
    }

    /// Dispatches through the registered provider with an optional stable retry key.
    pub async fn send_notification_idempotent(
        &self,
        input: SendNotificationInput,
        idempotency_key: Option<&str>,
    ) -> Result<NotificationDTO> {
        if input.channel.trim().eq_ignore_ascii_case("sms") {
            return Err(Error::Invalid {
                code: "sms_notifications_disabled",
                message: "SMS delivery is disabled; use an explicitly configured email or WhatsApp channel.".to_string(),
                field: Some("channel".to_string()),
            });
        }
        let provider = self.find_provider_for_channel(&input.channel)?;

        // Render subject and body
        let (subject, body) = if let Some(ref template_name) = input.template {
            let tmpl = self.get_template_by_name(template_name).await?;
            let subj = tmpl
                .subject_template
                .as_ref()
                .map(|s| Self::render_text(s, input.data.as_ref()));
            let b = Self::render_text(&tmpl.body_template, input.data.as_ref());
            (subj, b)
        } else {
            (
                input.subject.clone(),
                input.body.clone().unwrap_or_default(),
            )
        };

        let notification_id = Id::new();
        let now = Utc::now();

        let quota_policy = self.quota_policy.read().unwrap().clone();
        let mut direct_quota = None;
        if let Some(policy) = quota_policy {
            let tenant_id = stheno_db::tenant::current()?;
            if let Some(limit) = policy.daily_limit(&tenant_id, &input.channel).await? {
                let usage_date = Utc::now().date_naive();
                let reservation_id = format!("direct:{}", ulid::Ulid::new());
                if !self
                    .reserve_daily_quota(
                        &tenant_id,
                        &reservation_id,
                        &input.channel,
                        usage_date,
                        limit,
                    )
                    .await?
                {
                    return Err(Error::Invalid {
                        code: "daily_notification_limit_reached",
                        message: "The daily notification limit for this channel has been reached."
                            .to_string(),
                        field: Some("channel".to_string()),
                    });
                }
                direct_quota = Some((tenant_id, reservation_id, usage_date));
            }
        }

        // Attempt dispatch via provider
        let send_res = provider
            .send_idempotent(
                &input.to,
                &input.channel,
                subject.as_deref(),
                &body,
                input.data.as_ref(),
                idempotency_key,
            )
            .await;

        if send_res.is_ok() {
            if let Some((tenant_id, reservation_id, usage_date)) = direct_quota {
                self.settle_daily_quota(&tenant_id, &reservation_id, usage_date)
                    .await?;
            }
        }

        let (status, error_msg, meta) = match send_res {
            Ok(res) => ("sent".to_string(), None, res.metadata),
            Err(e) => ("failed".to_string(), Some(e.to_string()), None),
        };

        let notification = Notification {
            id: notification_id,
            to: input.to,
            channel: input.channel,
            template: input.template,
            data: input.data,
            trigger_type: input.trigger_type,
            resource_id: input.resource_id,
            resource_type: input.resource_type,
            receiver_id: input.receiver_id,
            status: status.clone(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let delivery = NotificationDelivery {
            id: Id::new(),
            notification_id: notification_id.to_string(),
            provider_id: provider.provider_id().to_string(),
            delivered_at: if status == "sent" {
                Some(Utc::now())
            } else {
                None
            },
            error_message: error_msg,
            metadata: meta,
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.notifications.create(None, notification).await?;
        self.deliveries.create(None, delivery).await?;

        Ok(NotificationDTO {
            id: created.id.to_string(),
            to: created.to,
            channel: created.channel,
            template: created.template,
            data: created.data,
            trigger_type: created.trigger_type,
            resource_id: created.resource_id,
            resource_type: created.resource_type,
            receiver_id: created.receiver_id,
            status: created.status,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    /// Persists a notification intent before delivery. Repeated calls with the same key reuse
    /// the existing durable intent, so scheduler retries cannot create duplicate messages.
    pub async fn enqueue_notification_idempotent(
        &self,
        input: SendNotificationInput,
        idempotency_key: &str,
    ) -> Result<bool> {
        let tenant_id = stheno_db::tenant::current()?;
        self.enqueue_notification_idempotent_for_tenant(input, idempotency_key, &tenant_id)
            .await
    }

    /// Persists an intent for a known tenant when a platform worker runs outside tenant scope.
    pub async fn enqueue_notification_idempotent_for_tenant(
        &self,
        input: SendNotificationInput,
        idempotency_key: &str,
        tenant_id: &str,
    ) -> Result<bool> {
        stheno_db::tenant::validate(tenant_id)?;
        if input.channel.trim().eq_ignore_ascii_case("sms") {
            return Err(Error::Invalid {
                code: "sms_notifications_disabled",
                message: "SMS delivery is disabled; use an explicitly configured email or WhatsApp channel.".to_string(),
                field: Some("channel".to_string()),
            });
        }
        if !matches!(input.channel.as_str(), "email" | "whatsapp")
            || idempotency_key.is_empty()
            || idempotency_key.len() > 240
            || idempotency_key.chars().any(char::is_control)
        {
            return Err(Error::Invalid {
                code: "invalid_notification_intent",
                message: "Notification channel or idempotency key is invalid.".to_string(),
                field: None,
            });
        }
        if input.channel == "whatsapp" && !self.has_whatsapp_consent(tenant_id, &input.to).await? {
            return Err(Error::Invalid {
                code: "whatsapp_consent_required",
                message: "The recipient has not opted in to WhatsApp notifications.".to_string(),
                field: Some("to".to_string()),
            });
        }
        let provider = self.find_provider_for_channel(&input.channel)?;
        let (subject, body) = if let Some(ref template_name) = input.template {
            let template = self.get_template_by_name(template_name).await?;
            (
                template
                    .subject_template
                    .as_deref()
                    .map(|value| Self::render_text(value, input.data.as_ref())),
                Self::render_text(&template.body_template, input.data.as_ref()),
            )
        } else {
            (
                input.subject.clone(),
                input.body.clone().unwrap_or_default(),
            )
        };
        if input.to.is_empty() || input.to.len() > 320 || body.len() > 32_000 {
            return Err(Error::Invalid {
                code: "invalid_notification_intent",
                message: "Notification recipient or body is outside the allowed bounds."
                    .to_string(),
                field: None,
            });
        }

        let notification_id = Id::<Notification>::new().to_string();
        let outbox_id = format!("nob_{}", ulid::Ulid::new());
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|error| Error::Database(error.to_string()))?;
        let idempotency_key = idempotency_key.to_string();
        let provider_id = provider.provider_id().to_string();
        let tenant_id = tenant_id.to_string();
        let inserted = stheno_db::transaction::with_transaction(&mut conn, move |conn| {
            Box::pin(async move {
                sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                    .bind::<Text, _>(&idempotency_key)
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                let inserted = sql_query(
                    r#"WITH created AS (
                        INSERT INTO notification.notification
                            (id, "to", channel, template, data, trigger_type, resource_id, resource_type, receiver_id, status, tenant_id)
                        SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $15
                        WHERE NOT EXISTS (SELECT 1 FROM notification.notification_outbox WHERE idempotency_key = $10)
                        RETURNING id
                     )
                     INSERT INTO notification.notification_outbox
                        (id, notification_id, idempotency_key, provider_id, "to", channel, subject, body, data, trigger_type, resource_id, resource_type, tenant_id)
                     SELECT $11, created.id, $10, $12, $2, $3, $13, $14, $5, $6, $7, $8, $15 FROM created WHERE true
                     ON CONFLICT (idempotency_key) DO NOTHING"#,
                )
                .bind::<Text, _>(&notification_id)
                .bind::<Text, _>(&input.to)
                .bind::<Text, _>(&input.channel)
                .bind::<Nullable<Text>, _>(input.template.as_deref())
                .bind::<Nullable<Jsonb>, _>(input.data.as_ref())
                .bind::<Nullable<Text>, _>(input.trigger_type.as_deref())
                .bind::<Nullable<Text>, _>(input.resource_id.as_deref())
                .bind::<Nullable<Text>, _>(input.resource_type.as_deref())
                .bind::<Nullable<Text>, _>(input.receiver_id.as_deref())
                .bind::<Text, _>(&idempotency_key)
                .bind::<Text, _>(&outbox_id)
                .bind::<Text, _>(&provider_id)
                .bind::<Nullable<Text>, _>(subject.as_deref())
                .bind::<Text, _>(&body)
                .bind::<Text, _>(&tenant_id)
                .execute(conn)
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
                Ok(inserted == 1)
            })
        })
        .await?;
        Ok(inserted)
    }

    /// Records affirmative consent after a trusted checkout, chat, or customer-settings action.
    pub async fn record_whatsapp_consent(
        &self,
        tenant_id: &str,
        recipient: &str,
        source: &str,
        customer_id: Option<&str>,
    ) -> Result<()> {
        stheno_db::tenant::validate(tenant_id)?;
        if !matches!(source, "checkout" | "chat" | "customer_settings") {
            return Err(Error::Invalid {
                code: "invalid_whatsapp_consent_source",
                message: "WhatsApp consent source is invalid.".to_string(),
                field: Some("source".to_string()),
            });
        }
        let recipient = normalize_e164(recipient)?;
        let source = source.to_string();
        let tenant = tenant_id.to_string();
        let customer_id = customer_id.map(str::to_string);
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    sql_query(
                        "INSERT INTO notification.whatsapp_consent \
                            (tenant_id, recipient_e164, customer_id, consented, consent_source, consented_at, revoked_at) \
                         VALUES ($1, $2, $3, true, $4, now(), NULL) \
                         ON CONFLICT (tenant_id, recipient_e164) DO UPDATE SET \
                            consented = true, consent_source = EXCLUDED.consent_source, \
                            customer_id = COALESCE(EXCLUDED.customer_id, notification.whatsapp_consent.customer_id), \
                            consented_at = now(), revoked_at = NULL, updated_at = now()",
                    )
                    .bind::<Text, _>(tenant)
                    .bind::<Text, _>(recipient)
                    .bind::<Nullable<Text>, _>(customer_id)
                    .bind::<Text, _>(source)
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    Ok(())
                })
            })
            .await
        })
        .await
    }

    /// Records explicit consent for anonymous WhatsApp analytics, separate from message opt-in.
    pub async fn record_whatsapp_analytics_consent(
        &self,
        tenant_id: &str,
        recipient: &str,
        customer_id: Option<&str>,
        source_event_id: &str,
    ) -> Result<()> {
        stheno_db::tenant::validate(tenant_id)?;
        let recipient_hash = whatsapp_analytics_recipient_hash(recipient)?;
        if customer_id.is_some_and(|id| id.trim().is_empty() || id.len() > 128)
            || source_event_id.trim().is_empty()
            || source_event_id.len() > 128
            || source_event_id.chars().any(char::is_control)
        {
            return Err(Error::Invalid {
                code: "invalid_whatsapp_analytics_consent",
                message: "WhatsApp analytics consent identity is invalid.".to_string(),
                field: Some("source_event_id".to_string()),
            });
        }
        let tenant = tenant_id.to_string();
        let customer_id = customer_id.map(str::to_string);
        let source_event_id = source_event_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    sql_query(
                        "INSERT INTO notification.whatsapp_analytics_consent \
                         (tenant_id, recipient_hash, customer_id, source_event_id, consent_source) \
                         VALUES ($1, $2, $3, $4, 'checkout') \
                         ON CONFLICT (tenant_id, recipient_hash) DO UPDATE SET \
                           customer_id = COALESCE(EXCLUDED.customer_id, notification.whatsapp_analytics_consent.customer_id), \
                           source_event_id = EXCLUDED.source_event_id, consented = true, \
                           consent_source = 'checkout', consented_at = now(), revoked_at = NULL \
                         WHERE notification.whatsapp_analytics_consent.source_event_id <> EXCLUDED.source_event_id",
                    )
                    .bind::<Text, _>(tenant)
                    .bind::<Text, _>(recipient_hash)
                    .bind::<Nullable<Text>, _>(customer_id)
                    .bind::<Text, _>(source_event_id)
                    .execute(conn)
                    .await
                    .map(|_| ())
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }

    /// Revokes every tenant-scoped WhatsApp consent associated with this authenticated customer.
    ///
    /// Keeping the customer association on the consent row lets a customer revoke consent for
    /// a phone number they no longer have on their profile.
    pub async fn revoke_whatsapp_consent_for_customer(
        &self,
        tenant_id: &str,
        customer_id: &str,
    ) -> Result<bool> {
        stheno_db::tenant::validate(tenant_id)?;
        if customer_id.is_empty() || customer_id.len() > 128 {
            return Err(Error::Invalid {
                code: "invalid_customer_id",
                message: "Customer id is invalid.".to_string(),
                field: Some("customer_id".to_string()),
            });
        }
        let tenant = tenant_id.to_string();
        let customer_id = customer_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    stheno_db::transaction::with_transaction(conn, move |conn| {
                        Box::pin(async move {
                            let notifications = sql_query(
                                "UPDATE notification.whatsapp_consent \
                                    SET consented = false, revoked_at = now(), updated_at = now() \
                                  WHERE tenant_id = $1 AND customer_id = $2 AND consented = true",
                            )
                            .bind::<Text, _>(&tenant)
                            .bind::<Text, _>(&customer_id)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            let analytics = sql_query(
                                "UPDATE notification.whatsapp_analytics_consent \
                                    SET consented = false, revoked_at = now() \
                                  WHERE tenant_id = $1 AND customer_id = $2 AND consented = true",
                            )
                            .bind::<Text, _>(tenant)
                            .bind::<Text, _>(customer_id)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            Ok(notifications > 0 || analytics > 0)
                        })
                    })
                    .await
                })
            })
            .await
        })
        .await
    }

    /// Revokes tenant-scoped WhatsApp consent; queued messages are checked again before sending.
    pub async fn revoke_whatsapp_consent(&self, tenant_id: &str, recipient: &str) -> Result<bool> {
        stheno_db::tenant::validate(tenant_id)?;
        let recipient = normalize_e164(recipient)?;
        let recipient_hash = whatsapp_analytics_recipient_hash(&recipient)?;
        let tenant = tenant_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    stheno_db::transaction::with_transaction(conn, move |conn| {
                        Box::pin(async move {
                            let notifications = sql_query(
                                "UPDATE notification.whatsapp_consent \
                                    SET consented = false, revoked_at = now(), updated_at = now() \
                                  WHERE tenant_id = $1 AND recipient_e164 = $2 AND consented = true",
                            )
                            .bind::<Text, _>(&tenant)
                            .bind::<Text, _>(recipient)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            let analytics = sql_query(
                                "UPDATE notification.whatsapp_analytics_consent \
                                    SET consented = false, revoked_at = now() \
                                  WHERE tenant_id = $1 AND recipient_hash = $2 AND consented = true",
                            )
                            .bind::<Text, _>(tenant)
                            .bind::<Text, _>(recipient_hash)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            Ok(notifications > 0 || analytics > 0)
                        })
                    })
                    .await
                })
            })
            .await
        })
        .await
    }

    /// Checks current consent inside the tenant's RLS scope at dispatch time.
    pub async fn has_whatsapp_consent(&self, tenant_id: &str, recipient: &str) -> Result<bool> {
        self.whatsapp_consent_checker()
            .has_consent(tenant_id, recipient)
            .await
    }

    /// Delivers due intents with a lease so multiple server instances cannot send the same
    /// row concurrently. Expired leases are reclaimed after process crashes.
    pub async fn deliver_due_outbox(&self, limit: i64) -> Result<(usize, usize)> {
        let limit = limit.clamp(1, 100);
        let mut delivered = 0usize;
        let mut failed = 0usize;
        {
            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
            sql_query(
                r#"WITH exhausted AS (
                    UPDATE notification.notification_outbox
                       SET status = 'dead', locked_until = NULL,
                           last_error = 'delivery lease expired at retry limit', updated_at = now()
                     WHERE status = 'sending' AND locked_until < now() AND attempts >= 12
                     RETURNING notification_id
                 )
                 UPDATE notification.notification n SET status = 'failed', updated_at = now()
                   FROM exhausted e WHERE n.id = e.notification_id"#,
            )
            .execute(&mut conn)
            .await
            .map_err(|error| Error::Database(error.to_string()))?;
        }
        for _ in 0..limit {
            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
            let row = sql_query(
                r#"UPDATE notification.notification_outbox SET status = 'sending',
                    locked_until = now() + interval '2 minutes', attempts = attempts + 1, updated_at = now()
                 WHERE id = (SELECT id FROM notification.notification_outbox
                    WHERE ((status = 'pending' AND available_at <= now())
                       OR (status = 'sending' AND locked_until < now()))
                      AND attempts < 12
                    ORDER BY available_at, created_at FOR UPDATE SKIP LOCKED LIMIT 1)
                 RETURNING id, idempotency_key, provider_id, "to", channel, tenant_id, subject, body, data"#,
            )
            .get_result::<OutboxDelivery>(&mut conn)
            .await;
            let row = match row {
                Ok(row) => row,
                Err(diesel::result::Error::NotFound) => break,
                Err(error) => return Err(Error::Database(error.to_string())),
            };
            drop(conn);

            let provider = {
                let registry = self.providers.read().unwrap();
                registry.0.get(&row.provider_id).cloned()
            };
            let supports_idempotency = provider
                .as_ref()
                .is_some_and(|provider| provider.supports_idempotency());

            let quota_policy = self.quota_policy.read().unwrap().clone();
            let mut reserved_quota_date = None;
            if let Some(policy) = quota_policy {
                match policy.daily_limit(&row.tenant_id, &row.channel).await {
                    Ok(Some(limit)) => {
                        let usage_date = Utc::now().date_naive();
                        match self
                            .reserve_daily_quota(
                                &row.tenant_id,
                                &row.id,
                                &row.channel,
                                usage_date,
                                limit,
                            )
                            .await
                        {
                            Ok(true) => reserved_quota_date = Some(usage_date),
                            Ok(false) => {
                                self.defer_outbox_until_utc_day(&row.id).await?;
                                continue;
                            }
                            Err(_) => {
                                self.defer_outbox_after_quota_error(&row.id).await?;
                                continue;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {
                        // A plan lookup failure must never bypass a configured spending cap.
                        self.defer_outbox_after_quota_error(&row.id).await?;
                        continue;
                    }
                }
            }
            let result = match provider {
                Some(provider) => {
                    match tokio::time::timeout(
                        Duration::from_secs(30),
                        provider.send_for_tenant(
                            &row.tenant_id,
                            &row.to,
                            &row.channel,
                            row.subject.as_deref(),
                            &row.body,
                            row.data.as_ref(),
                            Some(&row.idempotency_key),
                        ),
                    )
                    .await
                    {
                        Ok(result) => {
                            result.map(|result| (provider.provider_id().to_string(), result))
                        }
                        Err(_) => Err(Error::Database(
                            "notification provider timed out".to_string(),
                        )),
                    }
                }
                None => Err(Error::NotFound {
                    entity: "notification_provider",
                    id: row.provider_id.clone(),
                }),
            };

            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|error| Error::Database(error.to_string()))?;
            match result {
                Ok((provider_id, result)) => {
                    if let Some(usage_date) = reserved_quota_date {
                        self.settle_daily_quota(&row.tenant_id, &row.id, usage_date)
                            .await?;
                    }
                    sql_query(
                        r#"WITH updated_outbox AS (
                            UPDATE notification.notification_outbox
                               SET status = 'sent', sent_at = now(), locked_until = NULL,
                                   last_error = NULL, updated_at = now()
                             WHERE id = $1 RETURNING notification_id
                         ), updated_notification AS (
                            UPDATE notification.notification n SET status = 'sent', updated_at = now()
                              FROM updated_outbox o WHERE n.id = o.notification_id RETURNING n.id
                         )
                         INSERT INTO notification.notification_delivery
                            (id, notification_id, provider_id, delivered_at, metadata, tenant_id)
                         SELECT $2, id, $3, now(), $4, $5 FROM updated_notification"#,
                    )
                    .bind::<Text, _>(&row.id)
                    .bind::<Text, _>(format!("notidel_{}", ulid::Ulid::new()))
                    .bind::<Text, _>(&provider_id)
                    .bind::<Nullable<Jsonb>, _>(result.metadata.as_ref())
                    .bind::<Text, _>(&row.tenant_id)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    delivered += 1;
                }
                Err(_) => {
                    // Provider errors can contain tokens, phone numbers, or message content.
                    // Persist only a fixed, non-sensitive failure class.
                    sql_query(
                        r#"WITH failed_outbox AS (
                            UPDATE notification.notification_outbox
                               SET status = CASE WHEN attempts >= 12 OR NOT $2 THEN 'dead' ELSE 'pending' END,
                                   locked_until = NULL,
                                   available_at = now() + LEAST(interval '6 hours', interval '30 seconds' * power(2, LEAST(attempts, 10))),
                                   last_error = 'provider delivery failed', updated_at = now()
                             WHERE id = $1 RETURNING notification_id, status
                         )
                         UPDATE notification.notification n
                            SET status = CASE WHEN f.status = 'dead' THEN 'failed' ELSE 'pending' END, updated_at = now()
                           FROM failed_outbox f WHERE n.id = f.notification_id"#,
                    )
                    .bind::<Text, _>(&row.id)
                    .bind::<diesel::sql_types::Bool, _>(supports_idempotency)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    failed += 1;
                }
            }
        }
        Ok((delivered, failed))
    }

    async fn reserve_daily_quota(
        &self,
        tenant_id: &str,
        outbox_id: &str,
        channel: &str,
        usage_date: NaiveDate,
        limit: i32,
    ) -> Result<bool> {
        stheno_db::tenant::validate(tenant_id)?;
        let tenant = tenant_id.to_string();
        let scope_tenant = tenant.clone();
        let outbox_id = outbox_id.to_string();
        let channel = channel.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(scope_tenant, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let result = stheno_db::transaction::with_transaction(conn, move |conn| {
                        Box::pin(async move {
                            sql_query(
                                "DELETE FROM notification.daily_notification_usage \
                                  WHERE tenant_id = $1 AND usage_date < $2 - 35",
                            )
                            .bind::<Text, _>(&tenant)
                            .bind::<Date, _>(usage_date)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            sql_query(
                                "INSERT INTO notification.daily_notification_usage \
                                    (tenant_id, channel, usage_date) VALUES ($1, $2, $3) \
                                 ON CONFLICT (tenant_id, channel, usage_date) DO NOTHING",
                            )
                            .bind::<Text, _>(&tenant)
                            .bind::<Text, _>(&channel)
                            .bind::<Date, _>(usage_date)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            sql_query(
                                "SELECT pg_advisory_xact_lock(hashtextextended($1 || ':' || $2, 0))",
                            )
                            .bind::<Text, _>(&tenant)
                            .bind::<Text, _>(&outbox_id)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            let existing = sql_query(
                                "SELECT 1 AS present FROM notification.daily_notification_reservation \
                                  WHERE outbox_id = $1 AND tenant_id = $2 AND usage_date = $3 LIMIT 1",
                            )
                            .bind::<Text, _>(&outbox_id)
                            .bind::<Text, _>(&tenant)
                            .bind::<Date, _>(usage_date)
                            .get_result::<QuotaPresence>(conn)
                            .await
                            .optional()
                            .map_err(|error| Error::Database(error.to_string()))?;
                            if existing.map(|row| row.present == 1).unwrap_or(false) {
                                return Ok(true);
                            }
                            let reserved = sql_query(
                                "UPDATE notification.daily_notification_usage \
                                    SET reserved_count = reserved_count + 1 \
                                  WHERE tenant_id = $1 AND channel = $2 AND usage_date = $3 \
                                    AND reserved_count + sent_count < $4",
                            )
                            .bind::<Text, _>(&tenant)
                            .bind::<Text, _>(&channel)
                            .bind::<Date, _>(usage_date)
                            .bind::<diesel::sql_types::Integer, _>(limit.max(0))
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            if reserved == 0 {
                                return Ok(false);
                            }
                            sql_query(
                                "INSERT INTO notification.daily_notification_reservation \
                                    (outbox_id, tenant_id, channel, usage_date) VALUES ($1, $2, $3, $4)",
                            )
                            .bind::<Text, _>(&outbox_id)
                            .bind::<Text, _>(&tenant)
                            .bind::<Text, _>(&channel)
                            .bind::<Date, _>(usage_date)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            Ok(true)
                        })
                    })
                    .await?;
                    Ok(result)
                })
            })
            .await
        })
        .await
    }

    async fn settle_daily_quota(
        &self,
        tenant_id: &str,
        outbox_id: &str,
        usage_date: NaiveDate,
    ) -> Result<()> {
        stheno_db::tenant::validate(tenant_id)?;
        let tenant = tenant_id.to_string();
        let scope_tenant = tenant.clone();
        let outbox_id = outbox_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(scope_tenant, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    stheno_db::transaction::with_transaction(conn, move |conn| {
                        Box::pin(async move {
                            let reservation = sql_query(
                                "SELECT channel FROM notification.daily_notification_reservation \
                                  WHERE outbox_id = $1 AND tenant_id = $2 AND usage_date = $3 \
                                    AND status = 'reserved' FOR UPDATE",
                            )
                            .bind::<Text, _>(&outbox_id)
                            .bind::<Text, _>(&tenant)
                            .bind::<Date, _>(usage_date)
                            .get_result::<QuotaReservationChannel>(conn)
                            .await
                            .optional()
                            .map_err(|error| Error::Database(error.to_string()))?;
                            if let Some(reservation) = reservation {
                                let updated_usage = sql_query(
                                    "UPDATE notification.daily_notification_usage \
                                        SET reserved_count = reserved_count - 1, sent_count = sent_count + 1 \
                                      WHERE tenant_id = $1 AND channel = $2 AND usage_date = $3 \
                                        AND reserved_count > 0",
                                )
                                .bind::<Text, _>(&tenant)
                                .bind::<Text, _>(&reservation.channel)
                                .bind::<Date, _>(usage_date)
                                .execute(conn)
                                .await
                                .map_err(|error| Error::Database(error.to_string()))?;
                                if updated_usage != 1 {
                                    return Err(Error::Database(
                                        "notification quota reservation is inconsistent".to_string(),
                                    ));
                                }
                                sql_query(
                                    "UPDATE notification.daily_notification_reservation \
                                        SET status = 'sent', sent_at = now() \
                                      WHERE outbox_id = $1 AND tenant_id = $2 AND usage_date = $3 \
                                        AND status = 'reserved'",
                                )
                                .bind::<Text, _>(&outbox_id)
                                .bind::<Text, _>(&tenant)
                                .bind::<Date, _>(usage_date)
                                .execute(conn)
                                .await
                                .map_err(|error| Error::Database(error.to_string()))?;
                            }
                            Ok(())
                        })
                    })
                    .await
                })
            })
            .await
        })
        .await
    }

    async fn defer_outbox_until_utc_day(&self, outbox_id: &str) -> Result<()> {
        sql_query(
            "WITH deferred AS (\
                UPDATE notification.notification_outbox SET status = 'pending', locked_until = NULL,\
                    attempts = GREATEST(attempts - 1, 0),\
                    available_at = (date_trunc('day', now() AT TIME ZONE 'UTC') + interval '1 day') AT TIME ZONE 'UTC',\
                    last_error = 'daily notification limit reached', updated_at = now()\
                 WHERE id = $1 RETURNING notification_id\
             ) UPDATE notification.notification n SET status = 'pending', updated_at = now()\
               FROM deferred d WHERE n.id = d.notification_id",
        )
        .bind::<Text, _>(outbox_id)
        .execute(&mut self.pool.get().await.map_err(|error| Error::Database(error.to_string()))?)
        .await
        .map_err(|error| Error::Database(error.to_string()))?;
        Ok(())
    }

    async fn defer_outbox_after_quota_error(&self, outbox_id: &str) -> Result<()> {
        sql_query(
            "WITH deferred AS (\
                UPDATE notification.notification_outbox SET status = 'pending', locked_until = NULL,\
                    attempts = GREATEST(attempts - 1, 0), available_at = now() + interval '5 minutes',\
                    last_error = 'notification quota policy unavailable', updated_at = now()\
                 WHERE id = $1 RETURNING notification_id\
             ) UPDATE notification.notification n SET status = 'pending', updated_at = now()\
               FROM deferred d WHERE n.id = d.notification_id",
        )
        .bind::<Text, _>(outbox_id)
        .execute(&mut self.pool.get().await.map_err(|error| Error::Database(error.to_string()))?)
        .await
        .map_err(|error| Error::Database(error.to_string()))?;
        Ok(())
    }

    pub async fn get_notification(&self, id: &str) -> Result<NotificationDTO> {
        let n = self.notifications.retrieve(None, id).await?;

        Ok(NotificationDTO {
            id: n.id.to_string(),
            to: n.to,
            channel: n.channel,
            template: n.template,
            data: n.data,
            trigger_type: n.trigger_type,
            resource_id: n.resource_id,
            resource_type: n.resource_type,
            receiver_id: n.receiver_id,
            status: n.status,
            created_at: n.created_at,
            updated_at: n.updated_at,
        })
    }

    /// Returns whether a successful notification already exists for one stable lifecycle event.
    pub async fn has_sent_for_resource(
        &self,
        trigger_type: &str,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<bool> {
        let filter = FilterSpec::new()
            .with_predicate("trigger_type", FilterOp::Eq(trigger_type.to_string()))
            .with_predicate("resource_type", FilterOp::Eq(resource_type.to_string()))
            .with_predicate("resource_id", FilterOp::Eq(resource_id.to_string()))
            .with_predicate("status", FilterOp::Eq("sent".to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        Ok(!self.notifications.list(None, &filter).await?.is_empty())
    }

    pub async fn list_deliveries(
        &self,
        notification_id: &str,
    ) -> Result<Vec<NotificationDeliveryDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("notification_id", FilterOp::Eq(notification_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.deliveries.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|d| NotificationDeliveryDTO {
                id: d.id.to_string(),
                notification_id: d.notification_id,
                provider_id: d.provider_id,
                delivered_at: d.delivered_at,
                error_message: d.error_message,
                metadata: d.metadata,
                created_at: d.created_at,
            })
            .collect())
    }
}

impl WhatsAppConsentChecker {
    /// Checks current consent without retaining the notification provider registry.
    pub async fn has_consent(&self, tenant_id: &str, recipient: &str) -> Result<bool> {
        stheno_db::tenant::validate(tenant_id)?;
        let recipient = normalize_e164(recipient)?;
        let tenant = tenant_id.to_string();
        let scope_tenant = tenant.clone();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(scope_tenant, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let row = sql_query(
                        "SELECT consented FROM notification.whatsapp_consent \
                          WHERE tenant_id = $1 AND recipient_e164 = $2 LIMIT 1",
                    )
                    .bind::<Text, _>(tenant)
                    .bind::<Text, _>(recipient)
                    .get_result::<WhatsAppConsentRow>(conn)
                    .await
                    .optional()
                    .map_err(|error| Error::Database(error.to_string()))?;
                    Ok(row.is_some_and(|row| row.consented))
                })
            })
            .await
        })
        .await
    }
}

fn normalize_e164(value: &str) -> Result<String> {
    let normalized: String = value
        .trim()
        .chars()
        .filter(|character| !matches!(character, ' ' | '-' | '(' | ')'))
        .collect();
    let digits = normalized.strip_prefix('+').unwrap_or(&normalized);
    if normalized.starts_with('+')
        && (8..=15).contains(&digits.len())
        && digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        Ok(normalized)
    } else {
        Err(Error::Invalid {
            code: "invalid_whatsapp_recipient",
            message: "WhatsApp recipients must be valid E.164 phone numbers.".to_string(),
            field: Some("to".to_string()),
        })
    }
}

fn whatsapp_analytics_recipient_hash(recipient: &str) -> Result<String> {
    let normalized = normalize_e164(recipient)?;
    Ok(format!(
        "{:x}",
        Sha256::digest(normalized.trim_start_matches('+').as_bytes())
    ))
}
