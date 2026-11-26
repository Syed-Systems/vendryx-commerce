//! Payment service with PostgreSQL persistence via PgCrudRepository (TASKS P13-12).
//! Enforces outbound idempotency, row-locked ledger invariants, webhook convergence,
//! and card-testing abuse limits.

#![allow(dead_code)]

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};
use stheno_db::diesel::{
    self,
    sql_types::{BigInt, Integer, Nullable, Text, Timestamptz},
    QueryableByName,
};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};
use tokio::sync::Mutex as AsyncMutex;

use crate::dto::*;
use crate::models::{
    AccountHolder, Capture, CourierSettlementCycle, CourierSettlementLine, Payment,
    PaymentCollection, PaymentProviderRequest, PaymentReconciliationRun, PaymentSession,
    PaymentWebhookEvent, Refund, RefundReason,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ProviderCacheKey {
    tenant_id: String,
    provider_id: String,
    credential_version: i32,
}

#[derive(QueryableByName)]
struct CredentialRow {
    #[diesel(sql_type = Text)]
    ciphertext_hex: String,
    #[diesel(sql_type = Text)]
    nonce_hex: String,
    #[diesel(sql_type = Integer)]
    credential_version: i32,
}

#[derive(QueryableByName)]
struct CredentialSummaryRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    provider_id: String,
    #[diesel(sql_type = Integer)]
    credential_version: i32,
    #[diesel(sql_type = Text)]
    webhook_slug: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

/// One row of the outbound ledger, as the oversight read selects it.
#[derive(QueryableByName)]
struct ProviderRequestOversightRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    provider_id: String,
    #[diesel(sql_type = Text)]
    operation: String,
    #[diesel(sql_type = Text)]
    resource_type: String,
    #[diesel(sql_type = Text)]
    resource_id: String,
    #[diesel(sql_type = Text)]
    state: String,
    #[diesel(sql_type = Integer)]
    attempts: i32,
    #[diesel(sql_type = Timestamptz)]
    first_sent_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    settled_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(QueryableByName)]
struct ProviderRequestStateRow {
    #[diesel(sql_type = Text)]
    state: String,
}

fn credential_dto(
    row: CredentialSummaryRow,
    schema: &[CredentialField],
    health: CredentialHealth,
) -> PaymentProviderCredentialDTO {
    PaymentProviderCredentialDTO {
        id: row.id,
        provider_id: row.provider_id,
        credential_version: row.credential_version,
        webhook_slug: row.webhook_slug,
        health,
        fields: schema
            .iter()
            .map(|field| CredentialFieldReadDTO {
                key: field.key.to_string(),
                label: field.label.to_string(),
                kind: field.kind,
                required: field.required,
                configured: matches!(health, CredentialHealth::Configured),
            })
            .collect(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

#[derive(Clone)]
pub struct PaymentService {
    pool: DbPool,
    collections: Arc<PgCrudRepository<PaymentCollection>>,
    sessions: Arc<PgCrudRepository<PaymentSession>>,
    payments: Arc<PgCrudRepository<Payment>>,
    captures: Arc<PgCrudRepository<Capture>>,
    refunds: Arc<PgCrudRepository<Refund>>,
    refund_reasons: Arc<PgCrudRepository<RefundReason>>,
    account_holders: Arc<PgCrudRepository<AccountHolder>>,
    outbound_requests: Arc<PgCrudRepository<PaymentProviderRequest>>,
    webhook_events: Arc<PgCrudRepository<PaymentWebhookEvent>>,
    reconciliation_runs: Arc<PgCrudRepository<PaymentReconciliationRun>>,
    settlement_cycles: Arc<PgCrudRepository<CourierSettlementCycle>>,
    settlement_lines: Arc<PgCrudRepository<CourierSettlementLine>>,
    payment_locks: Arc<RwLock<HashMap<String, Arc<AsyncMutex<()>>>>>,
    provider_cache: Arc<RwLock<HashMap<ProviderCacheKey, Arc<dyn PaymentProvider>>>>,
    factories: Arc<RwLock<HashMap<String, Arc<dyn PaymentProviderFactory>>>>,
    ip_session_attempts: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl PaymentService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            collections: Arc::new(PgCrudRepository::new(pool.clone())),
            sessions: Arc::new(PgCrudRepository::new(pool.clone())),
            payments: Arc::new(PgCrudRepository::new(pool.clone())),
            captures: Arc::new(PgCrudRepository::new(pool.clone())),
            refunds: Arc::new(PgCrudRepository::new(pool.clone())),
            refund_reasons: Arc::new(PgCrudRepository::new(pool.clone())),
            account_holders: Arc::new(PgCrudRepository::new(pool.clone())),
            outbound_requests: Arc::new(PgCrudRepository::new(pool.clone())),
            webhook_events: Arc::new(PgCrudRepository::new(pool.clone())),
            reconciliation_runs: Arc::new(PgCrudRepository::new(pool.clone())),
            settlement_cycles: Arc::new(PgCrudRepository::new(pool.clone())),
            settlement_lines: Arc::new(PgCrudRepository::new(pool.clone())),
            pool,
            payment_locks: Arc::new(RwLock::new(HashMap::new())),
            provider_cache: Arc::new(RwLock::new(HashMap::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
            ip_session_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a ready-made provider instance under its own id.
    ///
    /// This was an empty stub "kept temporarily" while resolution moved to factories and
    /// tenant-owned credentials, so every caller registered nothing and then failed much
    /// later with `NotFound { payment_provider_factory }` — a silent no-op is the worst
    /// shape for a migration shim. It is sugar for a factory that takes no credentials and
    /// always yields this instance, which is precisely what a provider with nothing to
    /// configure is. Gateways register a real factory and keep their credential
    /// requirement.
    pub fn register_provider(&self, provider: Arc<dyn PaymentProvider>) {
        self.register_factory(Arc::new(FixedProviderFactory { provider }));
    }

    pub fn register_factory(&self, factory: Arc<dyn PaymentProviderFactory>) {
        let mut map = self.factories.write().unwrap();
        map.insert(factory.provider_id().to_string(), factory);
    }

    pub fn get_factory(&self, id: &str) -> Result<Arc<dyn PaymentProviderFactory>> {
        let map = self.factories.read().unwrap();
        map.get(id).cloned().ok_or_else(|| Error::NotFound {
            entity: "payment_provider_factory",
            id: id.to_string(),
        })
    }

    pub fn list_factories(&self) -> Vec<Arc<dyn PaymentProviderFactory>> {
        let mut factories: Vec<_> = self.factories.read().unwrap().values().cloned().collect();
        factories.sort_by(|a, b| a.provider_id().cmp(b.provider_id()));
        factories
    }

    pub async fn get_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<PaymentProviderCredentialDTO>> {
        stheno_db::tenant::validate(tenant_id)?;
        let factory = self.get_factory(provider_id)?;
        let Some(row) = self.load_credential_summary(tenant_id, provider_id).await? else {
            return Ok(None);
        };
        let health = self
            .credential_health_from_last_outcome(tenant_id, provider_id)
            .await?;
        Ok(Some(credential_dto(
            row,
            factory.credential_schema(),
            health,
        )))
    }

    pub async fn upsert_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        creds: &CredentialMap,
    ) -> Result<PaymentProviderCredentialDTO> {
        stheno_db::tenant::validate(tenant_id)?;
        let factory = self.get_factory(provider_id)?;
        validate_credential_map(factory.credential_schema(), creds)?;
        let encrypted = encrypt_credential_map(tenant_id, provider_id, creds)?;
        let row = self
            .upsert_credential_row(tenant_id, provider_id, encrypted)
            .await?;
        let health = self
            .credential_health_from_last_outcome(tenant_id, provider_id)
            .await?;
        Ok(credential_dto(row, factory.credential_schema(), health))
    }

    pub async fn get_provider(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Arc<dyn PaymentProvider>> {
        stheno_db::tenant::validate(tenant_id)?;
        let factory = self.get_factory(provider_id)?;

        // A provider whose credential schema is empty has nothing to store and nothing to
        // encrypt, so a stored row is not what makes it resolvable. Requiring one made the
        // built-in offline/manual provider unusable until somebody saved an empty
        // credential for it — the failure surfaced far from its cause, as a checkout that
        // reached payment with no method, and as `NotFound { payment_provider_credential }`
        // in every test that used it.
        //
        // Gateways are unaffected: their schemas are non-empty, so a tenant without a
        // credential still fails closed, which is what keeps one tenant's keys from ever
        // building another tenant's provider.
        let row = match self.load_credential_row(tenant_id, provider_id).await {
            Ok(row) => Some(row),
            Err(Error::NotFound { .. }) if factory.credential_schema().is_empty() => None,
            Err(e) => return Err(e),
        };

        let cache_key = ProviderCacheKey {
            tenant_id: tenant_id.to_string(),
            provider_id: provider_id.to_string(),
            credential_version: row.as_ref().map(|r| r.credential_version).unwrap_or(0),
        };

        if let Some(provider) = self.provider_cache.read().unwrap().get(&cache_key).cloned() {
            return Ok(provider);
        }

        let creds = match row {
            Some(row) => {
                let encrypted = EncryptedCredential {
                    ciphertext: hex::decode(row.ciphertext_hex).map_err(|_| Error::Invalid {
                        code: "credential_ciphertext_invalid",
                        message: "Credential ciphertext is invalid".to_string(),
                        field: None,
                    })?,
                    nonce: hex::decode(row.nonce_hex).map_err(|_| Error::Invalid {
                        code: "credential_nonce_invalid",
                        message: "Credential nonce is invalid".to_string(),
                        field: None,
                    })?,
                };
                decrypt_credential_map(tenant_id, provider_id, &encrypted)?
            }
            None => CredentialMap::new(),
        };
        let provider = factory.build(&creds)?;

        self.provider_cache
            .write()
            .unwrap()
            .insert(cache_key, provider.clone());

        Ok(provider)
    }

    async fn load_credential_row(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<CredentialRow> {
        let tenant_id = tenant_id.to_string();
        let provider_id = provider_id.to_string();
        let pool = self.pool.clone();

        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let row = diesel::sql_query(
                        "SELECT encode(ciphertext, 'hex') AS ciphertext_hex,
                                encode(nonce, 'hex') AS nonce_hex,
                                credential_version
                           FROM payment.payment_provider_credential
                          WHERE provider_id = $1
                            AND deleted_at IS NULL",
                    )
                    .bind::<Text, _>(&provider_id)
                    .get_result(conn)
                    .await;

                    match row {
                        Ok(row) => Ok(row),
                        Err(diesel::result::Error::NotFound) => Err(Error::NotFound {
                            entity: "payment_provider_credential",
                            id: provider_id,
                        }),
                        Err(e) => Err(Error::Database(e.to_string())),
                    }
                })
            })
            .await
        })
        .await
    }

    async fn load_credential_summary(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<CredentialSummaryRow>> {
        let tenant_id = tenant_id.to_string();
        let provider_id = provider_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let rows: Vec<CredentialSummaryRow> = diesel::sql_query(
                        "SELECT id,
                                provider_id,
                                credential_version,
                                webhook_slug,
                                created_at,
                                updated_at
                           FROM payment.payment_provider_credential
                          WHERE provider_id = $1
                            AND deleted_at IS NULL
                          LIMIT 1",
                    )
                    .bind::<Text, _>(&provider_id)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
                    Ok(rows.into_iter().next())
                })
            })
            .await
        })
        .await
    }

    async fn upsert_credential_row(
        &self,
        tenant_id: &str,
        provider_id: &str,
        encrypted: EncryptedCredential,
    ) -> Result<CredentialSummaryRow> {
        let tenant_id = tenant_id.to_string();
        let provider_id = provider_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "INSERT INTO payment.payment_provider_credential (
                             id, provider_id, ciphertext, nonce, credential_version, webhook_slug
                         ) VALUES (
                             $1, $2, decode($3, 'hex'), decode($4, 'hex'), 1, $5
                         )
                         ON CONFLICT (tenant_id, provider_id) WHERE deleted_at IS NULL
                         DO UPDATE SET
                             ciphertext = EXCLUDED.ciphertext,
                             nonce = EXCLUDED.nonce,
                             credential_version = payment.payment_provider_credential.credential_version + 1,
                             updated_at = now()
                         RETURNING id,
                                   provider_id,
                                   credential_version,
                                   webhook_slug,
                                   created_at,
                                   updated_at",
                    )
                    .bind::<Text, _>(format!("ppcred_{tenant_id}_{provider_id}"))
                    .bind::<Text, _>(&provider_id)
                    .bind::<Text, _>(hex::encode(encrypted.ciphertext))
                    .bind::<Text, _>(hex::encode(encrypted.nonce))
                    .bind::<Text, _>(format!("wh_{}", ulid::Ulid::new()))
                    .get_result::<CredentialSummaryRow>(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))
                })
            })
            .await
        })
        .await
    }

    /// One tenant's outbound provider calls, newest first.
    ///
    /// The control plane's payments oversight needs this across every tenant, but these
    /// rows sit behind RLS and `PLATFORM_SCOPE` deliberately matches nothing in a
    /// tenant-scoped table — so the cross-tenant view is a fan-out over this, one scope at
    /// a time, rather than a widened read. Keeping the method tenant-scoped is what makes
    /// that safe: there is no call shape here that reads another tenant's ledger.
    pub async fn list_provider_requests(
        &self,
        tenant_id: &str,
        limit: i64,
    ) -> Result<Vec<ProviderRequestOversightDTO>> {
        stheno_db::tenant::validate(tenant_id)?;
        let tenant = tenant_id.to_string();
        let pool = self.pool.clone();

        stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let rows: Vec<ProviderRequestOversightRow> = diesel::sql_query(
                        "SELECT id, provider_id, operation, resource_type, resource_id,
                                state, attempts, first_sent_at, settled_at
                           FROM payment.payment_provider_request
                          WHERE deleted_at IS NULL
                          ORDER BY COALESCE(settled_at, first_sent_at) DESC
                          LIMIT $1",
                    )
                    .bind::<BigInt, _>(limit)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
                    Ok(rows)
                })
            })
            .await
        })
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| ProviderRequestOversightDTO {
                    tenant_id: tenant.clone(),
                    id: r.id,
                    provider_id: r.provider_id,
                    operation: r.operation,
                    resource_type: r.resource_type,
                    resource_id: r.resource_id,
                    state: r.state,
                    attempts: r.attempts,
                    first_sent_at: r.first_sent_at,
                    settled_at: r.settled_at,
                })
                .collect()
        })
    }

    async fn credential_health_from_last_outcome(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<CredentialHealth> {
        let tenant_id = tenant_id.to_string();
        let provider_id = provider_id.to_string();
        let pool = self.pool.clone();
        let state = stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let rows: Vec<ProviderRequestStateRow> = diesel::sql_query(
                        "SELECT state
                           FROM payment.payment_provider_request
                          WHERE provider_id = $1
                            AND deleted_at IS NULL
                          ORDER BY COALESCE(settled_at, first_sent_at) DESC
                          LIMIT 1",
                    )
                    .bind::<Text, _>(&provider_id)
                    .load(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
                    Ok(rows.into_iter().next().map(|row| row.state))
                })
            })
            .await
        })
        .await?;

        Ok(match state.as_deref() {
            Some("succeeded") => CredentialHealth::Configured,
            Some("configured_unverified") => CredentialHealth::ConfiguredUnverified,
            Some("failed") => CredentialHealth::Invalid,
            _ => CredentialHealth::Unconfigured,
        })
    }

    pub async fn verify_provider_credentials(
        &self,
        tenant_id: &str,
        provider_id: &str,
        input_creds: Option<&CredentialMap>,
    ) -> Result<ProviderVerificationResult> {
        stheno_db::tenant::validate(tenant_id)?;
        let factory = self.get_factory(provider_id)?;

        let provider = match input_creds {
            Some(creds) => {
                validate_credential_map(factory.credential_schema(), creds)?;
                factory.build(creds)?
            }
            None => self.get_provider(tenant_id, provider_id).await?,
        };

        let result = provider.verify_connection().await?;

        let state = match result.status {
            ProviderHealthStatus::Healthy => "succeeded",
            ProviderHealthStatus::Failed => "failed",
            ProviderHealthStatus::ConfiguredUnverified => "configured_unverified",
            ProviderHealthStatus::Degraded => "failed",
        };

        let req_id = format!("ppreq_v_{}", ulid::Ulid::new());
        let idem_key = format!("idem_v_{}", ulid::Ulid::new());
        let provider_id_str = provider_id.to_string();
        let state_str = state.to_string();
        let error_msg = if !result.verified || result.status == ProviderHealthStatus::Failed {
            Some(result.message.clone())
        } else {
            None
        };

        let pool = self.pool.clone();
        let tenant_str = tenant_id.to_string();
        stheno_db::tenant::scope(tenant_str, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "INSERT INTO payment.payment_provider_request (
                             id, idempotency_key, provider_id, operation, resource_type, resource_id,
                             request_hash, state, response, error, attempts, first_sent_at, settled_at, deleted_at
                         ) VALUES (
                             $1, $2, $3, 'verify_credentials', 'provider_verification', $1,
                             'verify_probe', $4, '{}'::jsonb, CASE WHEN $5 IS NULL THEN NULL ELSE jsonb_build_object('message', $5::text) END, 1, now(), now(), NULL
                         )",
                    )
                    .bind::<Text, _>(&req_id)
                    .bind::<Text, _>(&idem_key)
                    .bind::<Text, _>(&provider_id_str)
                    .bind::<Text, _>(&state_str)
                    .bind::<Nullable<Text>, _>(&error_msg)
                    .execute(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
                    Ok(())
                })
            })
            .await
        })
        .await?;

        Ok(result)
    }

    /// The read side of refund reasons.
    ///
    /// The repository has existed since the module was written and nothing ever read from
    /// it: `/admin/refund-reasons` answered with two hardcoded rows, and the create route
    /// echoed its input back with an invented id. So a merchant could add a reason, see it
    /// "saved", and never find it again.
    pub async fn list_refund_reasons(&self) -> Result<Vec<RefundReasonDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let rows = self.refund_reasons.list(None, &filter).await?;
        Ok(rows.iter().map(Self::to_refund_reason_dto).collect())
    }

    pub async fn get_refund_reason(&self, id: &str) -> Result<RefundReasonDTO> {
        let reason = self.refund_reasons.retrieve(None, id).await?;
        Ok(Self::to_refund_reason_dto(&reason))
    }

    /// Partial update for a refund reason.
    pub async fn update_refund_reason(
        &self,
        id: &str,
        input: UpdateRefundReasonInput,
    ) -> Result<RefundReasonDTO> {
        let mut reason = self.refund_reasons.retrieve(None, id).await?;
        if let Some(label) = input.label {
            reason.label = label;
        }
        if let Some(description) = input.description {
            reason.description = Some(description);
        }
        if let Some(metadata) = input.metadata {
            reason.metadata = Some(metadata);
        }
        reason.updated_at = Utc::now();

        let updated = self.refund_reasons.update(None, id, reason).await?;
        Ok(Self::to_refund_reason_dto(&updated))
    }

    pub async fn delete_refund_reason_by_id(&self, id: &str) -> Result<()> {
        self.refund_reasons.soft_delete(None, id).await?;
        Ok(())
    }

    fn to_refund_reason_dto(r: &RefundReason) -> RefundReasonDTO {
        RefundReasonDTO {
            id: r.id.to_string(),
            label: r.label.clone(),
            description: r.description.clone(),
            metadata: r.metadata.clone(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }

    pub async fn create_refund_reason(
        &self,
        label: &str,
        description: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<String> {
        let reason = RefundReason {
            id: Id::new(),
            label: label.to_string(),
            description,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        let created = self.refund_reasons.create(None, reason).await?;
        Ok(created.id.to_string())
    }

    // ── Payment Collection CRUD ──────────────────────────────────────────────

    pub async fn create_payment_collection(
        &self,
        input: CreatePaymentCollectionInput,
    ) -> Result<PaymentCollectionDTO> {
        let id: Id<PaymentCollection> = Id::new();
        let now = Utc::now();
        let col = PaymentCollection {
            id,
            currency_code: input.currency_code.to_lowercase(),
            amount: input.amount,
            authorized_amount: dec!(0),
            captured_amount: dec!(0),
            refunded_amount: dec!(0),
            status: "not_paid".to_string(),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.collections.create(None, col).await?;
        self.get_payment_collection(&created.id.to_string()).await
    }

    /// Every payment collection.
    ///
    /// `/admin/payment-collections` answered from an empty literal in `config.rs` while
    /// the table was being written to by checkout.
    pub async fn list_payment_collections(&self) -> Result<Vec<PaymentCollectionDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let rows = self.collections.list(None, &filter).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.get_payment_collection(&row.id.to_string()).await?);
        }
        Ok(out)
    }

    pub async fn get_payment_collection(&self, id: &str) -> Result<PaymentCollectionDTO> {
        let col = self.collections.retrieve(None, id).await?;
        if col.deleted_at.is_some() {
            return Err(Error::NotFound {
                entity: "payment_collection",
                id: id.to_string(),
            });
        }

        let col_id_str = col.id.to_string();
        let sessions = self.get_sessions_for_collection(&col_id_str).await?;
        let payments = self.get_payments_for_collection(&col_id_str).await?;

        Ok(PaymentCollectionDTO {
            id: col.id.to_string(),
            currency_code: col.currency_code,
            amount: col.amount,
            authorized_amount: col.authorized_amount,
            captured_amount: col.captured_amount,
            refunded_amount: col.refunded_amount,
            status: col.status,
            payment_sessions: sessions,
            payments,
            metadata: col.metadata,
            created_at: col.created_at,
            updated_at: col.updated_at,
        })
    }

    pub async fn mark_collection_paid(&self, id: &str) -> Result<PaymentCollectionDTO> {
        let mut col = self.collections.retrieve(None, id).await?;
        col.status = "paid".to_string();
        col.captured_amount = col.amount;
        col.updated_at = Utc::now();
        let _ = self.collections.update(None, id, col).await?;
        self.get_payment_collection(id).await
    }

    // ── Payment Session Lifecycle & Card-Testing Limits (P4-27) ──────────────

    pub async fn create_payment_session(
        &self,
        input: CreatePaymentSessionInput,
        client_ip: Option<&str>,
    ) -> Result<PaymentSessionDTO> {
        // Card testing rate limit (P4-27, F10)
        if let Some(ip) = client_ip {
            if !self.check_ip_session_rate_limit(ip, 10) {
                return Err(Error::Invalid {
                    code: "card_testing_limit_exceeded",
                    message: "Too many payment sessions created from this IP".to_string(),
                    field: None,
                });
            }
        }

        let col = self
            .collections
            .retrieve(None, &input.payment_collection_id)
            .await?;

        // Cancel previous active sessions on this collection (PAYMENTS §11, E30)
        let spec = FilterSpec::new()
            .with_predicate("payment_collection_id", FilterOp::Eq(col.id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let existing_sessions = self.sessions.list(None, &spec).await?;
        for mut s in existing_sessions {
            if s.status != "canceled" && s.status != "error" && s.status != "captured" {
                s.status = "canceled".to_string();
                s.updated_at = Utc::now();
                let s_id = s.id.to_string();
                let _ = self.sessions.update(None, &s_id, s).await;
            }
        }

        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &input.provider_id).await?;
        let init_out = provider
            .initiate(InitiateInput {
                amount: input.amount,
                currency_code: input.currency_code.clone(),
                context: input.context.clone(),
                metadata: None,
            })
            .await?;

        let id: Id<PaymentSession> = Id::new();
        let now = Utc::now();
        let session = PaymentSession {
            id,
            payment_collection_id: input.payment_collection_id,
            provider_id: input.provider_id,
            amount: input.amount,
            currency_code: input.currency_code.to_lowercase(),
            status: init_out.status.as_str().to_string(),
            data: Some(init_out.data),
            context: input.context,
            authorized_at: None,
            status_observed_at: now,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.sessions.create(None, session).await?;
        self.get_payment_session(&created.id.to_string()).await
    }

    pub async fn get_payment_session(&self, id: &str) -> Result<PaymentSessionDTO> {
        let s = self.sessions.retrieve(None, id).await?;
        if s.deleted_at.is_some() {
            return Err(Error::NotFound {
                entity: "payment_session",
                id: id.to_string(),
            });
        }

        let status =
            PaymentSessionStatus::from_str_name(&s.status).unwrap_or(PaymentSessionStatus::Pending);

        Ok(PaymentSessionDTO {
            id: s.id.to_string(),
            payment_collection_id: s.payment_collection_id,
            provider_id: s.provider_id,
            amount: s.amount,
            currency_code: s.currency_code,
            status,
            data: s.data,
            context: s.context,
            authorized_at: s.authorized_at,
            status_observed_at: s.status_observed_at,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
    }

    // ── Outbound Idempotency & Ambiguity Resolution (P4-17, P4-18, PAYMENTS §6) ──

    #[allow(clippy::too_many_arguments)]
    pub async fn idempotent_call<F, Fut, T>(
        &self,
        idempotency_key: &str,
        provider_id: &str,
        operation: &str,
        resource_type: &str,
        resource_id: &str,
        payload: &serde_json::Value,
        call_fn: F,
    ) -> Result<T>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<T>>,
        T: Serialize + for<'de> Deserialize<'de> + Clone,
    {
        self.idempotent_call_with_reconciliation(
            idempotency_key,
            provider_id,
            operation,
            resource_type,
            resource_id,
            payload,
            call_fn,
            |_| async { Ok(None) },
        )
        .await
    }

    /// Runs an outbound operation with a provider-specific read path for ambiguous results.
    #[allow(clippy::too_many_arguments)]
    pub async fn idempotent_call_with_reconciliation<F, Fut, RF, RFut, T>(
        &self,
        idempotency_key: &str,
        provider_id: &str,
        operation: &str,
        resource_type: &str,
        resource_id: &str,
        payload: &serde_json::Value,
        call_fn: F,
        reconcile_fn: RF,
    ) -> Result<T>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<T>>,
        RF: FnOnce(String) -> RFut,
        RFut: Future<Output = Result<Option<T>>>,
        T: Serialize + for<'de> Deserialize<'de> + Clone,
    {
        let request_hash = compute_sha256(payload);

        let spec = FilterSpec::new()
            .with_predicate("idempotency_key", FilterOp::Eq(idempotency_key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        let existing = self.outbound_requests.list(None, &spec).await?;

        if let Some(req) = existing.into_iter().next() {
            // Key reuse with changed payload (PAYMENTS §6, E4)
            if req.request_hash != request_hash {
                return Err(Error::Invalid {
                    code: "idempotency_key_payload_mismatch",
                    message: "Idempotency key reused with different request payload".to_string(),
                    field: None,
                });
            }

            if req.state == "succeeded" {
                if let Some(resp_val) = req.response {
                    let parsed: T = serde_json::from_value(resp_val).map_err(|e| {
                        Error::Internal(
                            format!("Failed to deserialize cached response: {e}").into(),
                        )
                    })?;
                    return Ok(parsed);
                }
            } else if req.state == "failed" {
                return Err(Error::Invalid {
                    code: "previous_request_failed",
                    message: format!("Previous provider request with key {idempotency_key} failed"),
                    field: None,
                });
            } else if req.state == "in_flight" || req.state == "unknown" {
                // Ambiguous timeout recovery path (P4-18, PAYMENTS §6 step 5).
                if operation == "refund" {
                    if let Ok(Some(output)) = reconcile_fn(idempotency_key.to_string()).await {
                        let response = serde_json::to_value(&output).map_err(|error| {
                            Error::Internal(
                                format!(
                                    "Could not serialize reconciled provider response: {error}"
                                )
                                .into(),
                            )
                        })?;
                        let req_id = req.id.to_string();
                        let mut updated_req = req;
                        updated_req.state = "succeeded".to_string();
                        updated_req.response = Some(response);
                        updated_req.settled_at = Some(Utc::now());
                        self.outbound_requests
                            .update(None, &req_id, updated_req)
                            .await?;
                        return Ok(output);
                    }
                    return Err(Error::Conflict {
                        code: "provider_request_unknown",
                        message: format!(
                            "Refund request {idempotency_key} has an unknown provider outcome and requires reconciliation"
                        ),
                    });
                }
                if let Ok(tenant_id) = stheno_db::tenant::current() {
                    if let Ok(provider) = self.get_provider(&tenant_id, provider_id).await {
                        let ret_res = provider
                            .retrieve(RetrieveInput {
                                reference_id: resource_id.to_string(),
                                data: None,
                            })
                            .await;

                        if let Ok(ret) = ret_res {
                            if ret.status == PaymentSessionStatus::Authorized
                                || ret.status == PaymentSessionStatus::Captured
                            {
                                // Succeeded on remote, backfill and return
                                let resp_json = serde_json::to_value(&ret.data).unwrap_or_default();
                                let req_id = req.id.to_string();
                                let mut updated_req = req;
                                updated_req.state = "succeeded".to_string();
                                updated_req.response = Some(resp_json.clone());
                                updated_req.settled_at = Some(Utc::now());
                                let _ = self
                                    .outbound_requests
                                    .update(None, &req_id, updated_req)
                                    .await;

                                if let Ok(parsed) = serde_json::from_value::<T>(resp_json) {
                                    return Ok(parsed);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Record in_flight request
        let req_id: Id<PaymentProviderRequest> = Id::new();
        let now = Utc::now();
        let in_flight_req = PaymentProviderRequest {
            id: req_id,
            idempotency_key: idempotency_key.to_string(),
            provider_id: provider_id.to_string(),
            operation: operation.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            request_hash,
            state: "in_flight".to_string(),
            response: None,
            error: None,
            attempts: 1,
            first_sent_at: now,
            settled_at: None,
            deleted_at: None,
        };

        let mut recorded = self.outbound_requests.create(None, in_flight_req).await?;

        // Execute provider call
        let result = call_fn(idempotency_key.to_string()).await;

        match &result {
            Ok(output) => {
                recorded.state = "succeeded".to_string();
                recorded.response = serde_json::to_value(output).ok();
                recorded.settled_at = Some(Utc::now());
            }
            Err(e) => {
                // Ambiguous timeout/failure -> set to 'unknown' to trigger retrieve() reconciliation on retry
                recorded.state = "unknown".to_string();
                recorded.error = Some(serde_json::json!({ "error": format!("{e:?}") }));
                recorded.settled_at = Some(Utc::now());
            }
        }

        let rec_id = recorded.id.to_string();
        let _ = self.outbound_requests.update(None, &rec_id, recorded).await;
        result
    }

    // ── Convergence for stranded provider requests ───────────────────────────
    //
    // `idempotent_call` converges an ambiguous request only when the *same key is
    // retried*. After a process dies mid-call nothing retries: the customer has gone,
    // and bKash-style gateways only send a webhook on success, so silence is not
    // evidence of failure. Worse, a crash leaves the row `in_flight` rather than
    // `unknown`, because the state is written after the call returns.
    //
    // This sweep is what closes that gap, and it is the difference between
    // timeout-safety and crash-safety. Without it a stranded row sits forever and the
    // money is found by a customer complaint or a month-end statement.

    /// Asks the provider what actually happened to one stranded request and settles it.
    /// Returns the state it reached.
    async fn converge_request(&self, req: &PaymentProviderRequest) -> Result<String> {
        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &req.provider_id).await?;

        let retrieved = provider
            .retrieve(RetrieveInput {
                reference_id: req.resource_id.clone(),
                data: None,
            })
            .await;

        let Ok(ret) = retrieved else {
            // The provider could not answer. Leave the row alone: guessing here is how
            // a real payment gets marked failed.
            return Ok(req.state.clone());
        };

        let mut updated = req.clone();
        let now = Utc::now();

        match ret.status {
            PaymentSessionStatus::Authorized | PaymentSessionStatus::Captured => {
                updated.state = "succeeded".to_string();
                updated.response = Some(serde_json::to_value(&ret.data).unwrap_or_default());
            }
            PaymentSessionStatus::Canceled | PaymentSessionStatus::Error => {
                updated.state = "failed".to_string();
                updated.error = Some(serde_json::json!({
                    "converged_from": req.state,
                    "provider_status": ret.status.as_str(),
                }));
            }
            // Still moving. Ask again on the next sweep rather than deciding.
            _ => return Ok(req.state.clone()),
        }

        updated.settled_at = Some(now);
        let state = updated.state.clone();
        let id = updated.id.to_string();
        self.outbound_requests.update(None, &id, updated).await?;
        Ok(state)
    }

    /// Sweeps provider requests stranded in `in_flight` or `unknown` and settles what
    /// the provider can confirm. Safe to run repeatedly; it only ever moves a request
    /// from ambiguous to decided.
    ///
    /// `min_age_seconds` keeps the sweep off requests that are simply still running.
    pub async fn reconcile_unsettled_requests(
        &self,
        min_age_seconds: i64,
        limit: usize,
    ) -> Result<UnsettledReconcileReport> {
        let spec = FilterSpec::new()
            .with_predicate(
                "state",
                FilterOp::In(vec!["in_flight".to_string(), "unknown".to_string()]),
            )
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, limit);

        let cutoff = Utc::now() - chrono::Duration::seconds(min_age_seconds);
        let candidates: Vec<PaymentProviderRequest> = self
            .outbound_requests
            .list(None, &spec)
            .await?
            .into_iter()
            .filter(|r| r.first_sent_at <= cutoff)
            .collect();

        let mut report = UnsettledReconcileReport {
            scanned: candidates.len(),
            ..Default::default()
        };

        for req in &candidates {
            match self.converge_request(req).await {
                Ok(state) if state == "succeeded" => report.converged += 1,
                Ok(state) if state == "failed" => report.resolved_failed += 1,
                _ => report.still_unsettled += 1,
            }
        }

        Ok(report)
    }

    // ── Authorization & Payment Creation ─────────────────────────────────────

    pub async fn authorize_payment_session(
        &self,
        input: AuthorizePaymentSessionInput,
    ) -> Result<PaymentDTO> {
        let session = self.get_payment_session(&input.session_id).await?;
        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &session.provider_id).await?;

        // Every money-moving call goes through the outbound ledger, so an ambiguous
        // timeout leaves a durable `unknown` row to converge from rather than a lost
        // payment. A session is authorized once, so the key derives from its id.
        let auth_input = AuthorizeInput {
            session_id: session.id.clone(),
            amount: session.amount,
            currency_code: session.currency_code.clone(),
            context: input.context.clone(),
            data: session.data.clone(),
        };
        let auth_payload = serde_json::to_value(&auth_input).unwrap_or_default();
        let auth_provider = provider.clone();
        let auth_out = self
            .idempotent_call(
                &format!("authorize:{}", session.id),
                &session.provider_id,
                "authorize",
                "payment_session",
                &session.id,
                &auth_payload,
                move |_key| async move { auth_provider.authorize(auth_input).await },
            )
            .await?;

        if auth_out.status != PaymentSessionStatus::Authorized {
            // Update session status and fail
            let mut s = self.sessions.retrieve(None, &session.id).await?;
            s.status = auth_out.status.as_str().to_string();
            s.data = Some(auth_out.data);
            s.updated_at = Utc::now();
            let _ = self.sessions.update(None, &session.id, s).await;

            return Err(Error::Invalid {
                code: "payment_authorization_failed",
                message: format!(
                    "Session authorization resulted in {}",
                    auth_out.status.as_str()
                ),
                field: None,
            });
        }

        // Update session to authorized
        let now = Utc::now();
        let mut s = self.sessions.retrieve(None, &session.id).await?;
        s.status = "authorized".to_string();
        s.authorized_at = Some(now);
        s.data = Some(auth_out.data.clone());
        s.updated_at = now;
        let _ = self.sessions.update(None, &session.id, s).await;

        // Create Payment entity ONLY on successful authorization (DATA_MODEL §2.7, PAYMENTS §2)
        let payment_id: Id<Payment> = Id::new();
        let payment = Payment {
            id: payment_id,
            payment_collection_id: session.payment_collection_id.clone(),
            payment_session_id: Some(session.id.clone()),
            provider_id: session.provider_id.clone(),
            currency_code: session.currency_code.clone(),
            amount: session.amount,
            captured_at: None,
            canceled_at: None,
            data: Some(auth_out.data),
            metadata: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created_payment = self.payments.create(None, payment).await?;

        // Update collection authorized_amount & status
        let mut col = self
            .collections
            .retrieve(None, &session.payment_collection_id)
            .await?;
        col.authorized_amount += session.amount;
        col.status = "authorized".to_string();
        col.updated_at = now;
        let _ = self
            .collections
            .update(None, &session.payment_collection_id, col)
            .await;

        self.get_payment(&created_payment.id.to_string()).await
    }

    /// Voids an authorised payment, releasing the hold with the provider.
    ///
    /// The complete-cart SAGA lists `cancel_authorization` as its compensation for the
    /// authorisation step. That compensation was a `tracing::info!` and nothing else, which
    /// was survivable only while the step it compensated also did nothing. Now that
    /// authorisation is real, a checkout that fails after it would leave a genuine hold on
    /// the buyer's money with no path to release it.
    pub async fn cancel_payment(&self, payment_id: &str) -> Result<PaymentDTO> {
        let payment = self.payments.retrieve(None, payment_id).await?;
        if payment.canceled_at.is_some() {
            // Compensation runs on an unwind path that may itself be retried; voiding twice
            // must not be an error.
            return self.get_payment(payment_id).await;
        }

        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &payment.provider_id).await?;
        // A payment is voided once; double-void is already treated as a no-op above.
        let cancel_input = CancelInput {
            payment_id: payment_id.to_string(),
            data: payment.data.clone(),
        };
        let cancel_payload = serde_json::to_value(&cancel_input).unwrap_or_default();
        let cancel_provider = provider.clone();
        let out = self
            .idempotent_call(
                &format!("cancel:{payment_id}"),
                &payment.provider_id,
                "cancel",
                "payment",
                payment_id,
                &cancel_payload,
                move |_key| async move { cancel_provider.cancel(cancel_input).await },
            )
            .await?;

        let now = Utc::now();
        let mut updated = payment.clone();
        updated.canceled_at = Some(now);
        updated.data = Some(out.data);
        updated.updated_at = now;
        let _ = self.payments.update(None, payment_id, updated).await;

        // The collection no longer holds this amount.
        if let Ok(mut col) = self
            .collections
            .retrieve(None, &payment.payment_collection_id)
            .await
        {
            col.authorized_amount -= payment.amount;
            col.status = if col.authorized_amount > dec!(0) {
                "authorized".to_string()
            } else {
                "canceled".to_string()
            };
            col.updated_at = now;
            let collection_id = payment.payment_collection_id.clone();
            let _ = self.collections.update(None, &collection_id, col).await;
        }

        self.get_payment(payment_id).await
    }

    pub async fn get_payment(&self, id: &str) -> Result<PaymentDTO> {
        let p = self.payments.retrieve(None, id).await?;
        if p.deleted_at.is_some() {
            return Err(Error::NotFound {
                entity: "payment",
                id: id.to_string(),
            });
        }

        let payment_id_str = p.id.to_string();
        let captures = self.get_captures_for_payment(&payment_id_str).await?;
        let refunds = self.get_refunds_for_payment(&payment_id_str).await?;

        let captured_amount: Decimal = captures.iter().map(|c| c.amount).sum();
        let refunded_amount: Decimal = refunds.iter().map(|r| r.amount).sum();

        Ok(PaymentDTO {
            id: p.id.to_string(),
            payment_collection_id: p.payment_collection_id,
            payment_session_id: p.payment_session_id,
            provider_id: p.provider_id,
            currency_code: p.currency_code,
            amount: p.amount,
            captured_at: p.captured_at,
            canceled_at: p.canceled_at,
            captures,
            refunds,
            captured_amount,
            refunded_amount,
            data: p.data,
            metadata: p.metadata,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
    }

    // ── Ledger Invariants: Capture & Refund under Row Lock (P4-21, P4-22, PAYMENTS §3.3) ──

    fn get_or_create_payment_lock(&self, payment_id: &str) -> Arc<AsyncMutex<()>> {
        let mut locks = self.payment_locks.write().unwrap();
        locks
            .entry(payment_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }

    pub async fn capture_payment(&self, input: CapturePaymentInput) -> Result<CaptureDTO> {
        let lock = self.get_or_create_payment_lock(&input.payment_id);
        let _guard = lock.lock().await;

        let payment = self.get_payment(&input.payment_id).await?;
        let current_captured = payment.captured_amount;
        let amount_to_capture = input.amount.unwrap_or(payment.amount - current_captured);

        // Ledger Invariant 1: 0 <= sum(captures) + amount <= payment.amount (PAYMENTS §3.3)
        if amount_to_capture <= dec!(0) {
            return Err(Error::Invalid {
                code: "invalid_capture_amount",
                message: "Capture amount must be greater than zero".to_string(),
                field: None,
            });
        }

        if current_captured + amount_to_capture > payment.amount {
            return Err(Error::Conflict {
                code: "over_capture_prevented",
                message: format!(
                    "Cannot capture {}: existing captured {} + {} exceeds authorized amount {}",
                    amount_to_capture, current_captured, amount_to_capture, payment.amount
                ),
            });
        }

        // Call provider capture through the outbound ledger.
        //
        // The key carries `current_captured` as a sequence. A retry after a crash sees
        // the same running total — the capture row was never written — so it dedupes to
        // the original request. A genuine second capture advances the total and gets its
        // own key. Keying on amount alone would silently merge two legitimate partial
        // captures of equal size into one.
        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &payment.provider_id).await?;
        let capture_input = CaptureInput {
            payment_id: payment.id.clone(),
            amount: amount_to_capture,
            currency_code: payment.currency_code.clone(),
            data: payment.data.clone(),
        };
        let capture_payload = serde_json::to_value(&capture_input).unwrap_or_default();
        let capture_provider = provider.clone();
        let _capture_out = self
            .idempotent_call(
                &format!(
                    "capture:{}:{}:{}",
                    payment.id, current_captured, amount_to_capture
                ),
                &payment.provider_id,
                "capture",
                "payment",
                &payment.id,
                &capture_payload,
                move |_key| async move { capture_provider.capture(capture_input).await },
            )
            .await?;

        let capture = Capture {
            id: Id::new(),
            payment_id: payment.id.clone(),
            amount: amount_to_capture,
            created_by: input.created_by.clone(),
            metadata: input.metadata.clone(),
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created_capture = self.captures.create(None, capture).await?;

        // Update payment collection captured_amount
        let mut col = self
            .collections
            .retrieve(None, &payment.payment_collection_id)
            .await?;
        col.captured_amount += amount_to_capture;
        if col.captured_amount >= col.amount {
            col.status = "captured".to_string();
        }
        col.updated_at = Utc::now();
        let _ = self
            .collections
            .update(None, &payment.payment_collection_id, col)
            .await;

        // Update payment captured_at if fully captured
        let mut p = self.payments.retrieve(None, &payment.id).await?;
        if current_captured + amount_to_capture >= payment.amount {
            p.captured_at = Some(Utc::now());
        }
        p.updated_at = Utc::now();
        let _ = self.payments.update(None, &payment.id, p).await;

        Ok(CaptureDTO {
            id: created_capture.id.to_string(),
            payment_id: created_capture.payment_id,
            amount: created_capture.amount,
            created_by: created_capture.created_by,
            metadata: created_capture.metadata,
            created_at: created_capture.created_at,
        })
    }

    pub async fn refund_payment(&self, input: RefundPaymentInput) -> Result<RefundDTO> {
        let lock = self.get_or_create_payment_lock(&input.payment_id);
        let _guard = lock.lock().await;

        let payment = self.get_payment(&input.payment_id).await?;
        let current_captured = payment.captured_amount;
        let current_refunded = payment.refunded_amount;

        // Ledger Invariant 2: 0 <= sum(refunds) + amount <= sum(captures) (PAYMENTS §3.3)
        if input.amount <= dec!(0) {
            return Err(Error::Invalid {
                code: "invalid_refund_amount",
                message: "Refund amount must be greater than zero".to_string(),
                field: None,
            });
        }

        if current_refunded + input.amount > current_captured {
            return Err(Error::Conflict {
                code: "over_refund_prevented",
                message: format!(
                    "Cannot refund {}: total refunded {} exceeds total captured {}",
                    input.amount,
                    current_refunded + input.amount,
                    current_captured
                ),
            });
        }

        // Same sequencing rule as capture: `current_refunded` distinguishes a retry
        // from a second, legitimate partial refund of the same amount.
        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &payment.provider_id).await?;
        let refund_input = RefundInput {
            payment_id: payment.id.clone(),
            amount: input.amount,
            currency_code: payment.currency_code.clone(),
            reason: input.refund_reason_id.clone(),
            note: input.note.clone(),
            data: payment.data.clone(),
        };
        let refund_payload = serde_json::to_value(&refund_input).unwrap_or_default();
        let refund_provider = provider.clone();
        let reconcile_provider = provider.clone();
        let reconcile_input = refund_input.clone();
        let idempotency_key = format!(
            "refund:{}:{}:{}",
            payment.id, current_refunded, input.amount
        );
        let _refund_out = self
            .idempotent_call_with_reconciliation(
                &idempotency_key,
                &payment.provider_id,
                "refund",
                "payment",
                &payment.id,
                &refund_payload,
                move |key| async move {
                    refund_provider
                        .refund_with_idempotency_key(refund_input, &key)
                        .await
                },
                move |key| async move {
                    reconcile_provider
                        .reconcile_refund(&reconcile_input, &key)
                        .await
                },
            )
            .await?;

        let refund = Refund {
            id: Id::new(),
            payment_id: payment.id.clone(),
            amount: input.amount,
            refund_reason_id: input.refund_reason_id,
            note: input.note,
            created_by: input.created_by,
            metadata: input.metadata,
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created_refund = self.refunds.create(None, refund).await?;

        // Update payment collection refunded_amount
        let mut col = self
            .collections
            .retrieve(None, &payment.payment_collection_id)
            .await?;
        col.refunded_amount += input.amount;
        col.updated_at = Utc::now();
        let _ = self
            .collections
            .update(None, &payment.payment_collection_id, col)
            .await;

        Ok(RefundDTO {
            id: created_refund.id.to_string(),
            payment_id: created_refund.payment_id,
            amount: created_refund.amount,
            refund_reason_id: created_refund.refund_reason_id,
            note: created_refund.note,
            created_by: created_refund.created_by,
            metadata: created_refund.metadata,
            created_at: created_refund.created_at,
        })
    }

    // ── Webhook Store & Convergence Reducer (P4-19, P4-20, PAYMENTS §7) ──────

    pub async fn receive_webhook(
        &self,
        provider_id: &str,
        raw_body: &[u8],
        signature_header: Option<&str>,
    ) -> Result<PaymentWebhookEventDTO> {
        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, provider_id).await?;
        let verified = provider.verify_webhook(raw_body, signature_header)?;

        // A parsed payload is not an authenticated one. Providers whose callbacks carry no
        // signature confirm against their own status API here; the default is a no-op for
        // the ones that really did verify. Refusing with a signature-shaped code means the
        // hooks route answers 401 and the gateway stops retrying a payload we will never
        // accept, rather than 400-looping.
        if !provider.confirm_webhook_event(&verified).await? {
            return Err(Error::Invalid {
                code: "webhook_signature_invalid",
                message: "The gateway did not confirm this event".to_string(),
                field: None,
            });
        }

        // Webhook replay guard (PAYMENTS §7.1, §7.3, E9): Unique constraint
        let spec = FilterSpec::new()
            .with_predicate("provider_id", FilterOp::Eq(provider_id.to_string()))
            .with_predicate(
                "provider_event_id",
                FilterOp::Eq(verified.provider_event_id.clone()),
            )
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.webhook_events.list(None, &spec).await?;
        if let Some(event) = existing.into_iter().next() {
            // Replay detected -> return 200 OK with existing event without processing
            return Ok(PaymentWebhookEventDTO {
                id: event.id.to_string(),
                provider_id: event.provider_id,
                provider_event_id: event.provider_event_id,
                event_type: event.event_type,
                payload: event.payload,
                signature_verified: event.signature_verified,
                provider_created_at: event.provider_created_at,
                state: "ignored".to_string(),
                processed_at: event.processed_at,
                error: event.error,
                created_at: event.created_at,
            });
        }

        let event = PaymentWebhookEvent {
            id: Id::new(),
            provider_id: provider_id.to_string(),
            provider_event_id: verified.provider_event_id.clone(),
            event_type: verified.event_type.clone(),
            payload: verified.payload.clone(),
            signature_verified: true,
            provider_created_at: verified.provider_created_at,
            state: "received".to_string(),
            processed_at: None,
            error: None,
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created_event = self.webhook_events.create(None, event).await?;

        // Apply convergence reducer
        let action = provider.map_webhook(&verified)?;
        self.apply_webhook_action(provider_id, &verified, action)
            .await;

        Ok(PaymentWebhookEventDTO {
            id: created_event.id.to_string(),
            provider_id: created_event.provider_id,
            provider_event_id: created_event.provider_event_id,
            event_type: created_event.event_type,
            payload: created_event.payload,
            signature_verified: true,
            provider_created_at: created_event.provider_created_at,
            state: "received".to_string(),
            processed_at: None,
            error: None,
            created_at: created_event.created_at,
        })
    }

    /// Convergence Reducer (PAYMENTS §7.2): Monotonic transitions + provider timestamp ordering
    async fn apply_webhook_action(
        &self,
        _provider_id: &str,
        event: &VerifiedEvent,
        action: WebhookAction,
    ) {
        if action == WebhookAction::NotSupported {
            return;
        }

        let target_status = match action {
            WebhookAction::Authorized => PaymentSessionStatus::Authorized,
            WebhookAction::Captured => PaymentSessionStatus::Captured,
            WebhookAction::Failed => PaymentSessionStatus::Error,
            WebhookAction::Pending => PaymentSessionStatus::Pending,
            WebhookAction::RequiresMore => PaymentSessionStatus::RequiresMore,
            WebhookAction::Canceled => PaymentSessionStatus::Canceled,
            WebhookAction::PendingAuthorization => PaymentSessionStatus::PendingAuthorization,
            WebhookAction::NotSupported => return,
        };

        // If event payload has a session_id, apply monotonic convergence
        if let Some(session_id) = event.payload.get("session_id").and_then(|v| v.as_str()) {
            if let Ok(mut session) = self.sessions.retrieve(None, session_id).await {
                let current_status = PaymentSessionStatus::from_str_name(&session.status)
                    .unwrap_or(PaymentSessionStatus::Pending);

                // Convergence Rule 1: Monotonic transition check
                if !current_status.can_transition_to(target_status) {
                    // Backwards transition dropped (PAYMENTS §7.2, E8)
                    return;
                }

                // Convergence Rule 2: Provider timestamp ordering
                if event.provider_created_at < session.status_observed_at {
                    // Stale observation dropped
                    return;
                }

                session.status = target_status.as_str().to_string();
                session.status_observed_at = event.provider_created_at;
                session.updated_at = Utc::now();
                let _ = self.sessions.update(None, session_id, session).await;
            }
        }
    }

    // ── Reconciliation Runs & Webhook Oversight (Platform) ───────────────────

    pub async fn list_reconciliation_runs(
        &self,
        tenant_id: &str,
        limit: usize,
    ) -> Result<Vec<PaymentReconciliationRunDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, limit);

        stheno_db::tenant::scope(tenant_id, async {
            let rows = self.reconciliation_runs.list(None, &spec).await?;
            Ok(rows
                .into_iter()
                .map(|r| PaymentReconciliationRunDTO {
                    id: r.id.to_string(),
                    provider_id: r.provider_id,
                    window_start: r.window_start,
                    window_end: r.window_end,
                    checked_count: r.checked_count,
                    drift_count: r.drift_count,
                    drifts: r.drifts,
                    state: r.state,
                    created_at: r.created_at,
                    completed_at: r.completed_at,
                })
                .collect())
        })
        .await
    }

    pub async fn list_webhook_events(
        &self,
        tenant_id: &str,
        limit: usize,
    ) -> Result<Vec<PaymentWebhookEventDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, limit);

        stheno_db::tenant::scope(tenant_id, async {
            let rows = self.webhook_events.list(None, &spec).await?;
            Ok(rows
                .into_iter()
                .map(|e| PaymentWebhookEventDTO {
                    id: e.id.to_string(),
                    provider_id: e.provider_id,
                    provider_event_id: e.provider_event_id,
                    event_type: e.event_type,
                    payload: e.payload,
                    signature_verified: e.signature_verified,
                    provider_created_at: e.provider_created_at,
                    state: e.state,
                    processed_at: e.processed_at,
                    error: e.error,
                    created_at: e.created_at,
                })
                .collect())
        })
        .await
    }

    pub async fn replay_webhook_event(
        &self,
        tenant_id: &str,
        webhook_id: &str,
    ) -> Result<PaymentWebhookEventDTO> {
        stheno_db::tenant::scope(tenant_id, async {
            let mut event = self.webhook_events.retrieve(None, webhook_id).await?;
            event.state = "processed".to_string();
            event.processed_at = Some(Utc::now());
            let updated = self.webhook_events.update(None, webhook_id, event).await?;
            Ok(PaymentWebhookEventDTO {
                id: updated.id.to_string(),
                provider_id: updated.provider_id,
                provider_event_id: updated.provider_event_id,
                event_type: updated.event_type,
                payload: updated.payload,
                signature_verified: updated.signature_verified,
                provider_created_at: updated.provider_created_at,
                state: updated.state,
                processed_at: updated.processed_at,
                error: updated.error,
                created_at: updated.created_at,
            })
        })
        .await
    }

    // ── Card Testing Abuse Limits (P4-27, F10) ────────────────────────────────

    pub fn check_ip_session_rate_limit(&self, ip: &str, max_per_minute: usize) -> bool {
        let mut map = self.ip_session_attempts.write().unwrap();
        let one_minute_ago = Utc::now() - Duration::minutes(1);
        let timestamps = map.entry(ip.to_string()).or_default();
        timestamps.retain(|t| *t > one_minute_ago);

        if timestamps.len() >= max_per_minute {
            false
        } else {
            timestamps.push(Utc::now());
            true
        }
    }

    // ── Internal Helpers ─────────────────────────────────────────────────────

    async fn get_sessions_for_collection(
        &self,
        collection_id: &str,
    ) -> Result<Vec<PaymentSessionDTO>> {
        let spec = FilterSpec::new()
            .with_predicate(
                "payment_collection_id",
                FilterOp::Eq(collection_id.to_string()),
            )
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let sessions = self.sessions.list(None, &spec).await?;

        Ok(sessions
            .into_iter()
            .map(|s| {
                let status = PaymentSessionStatus::from_str_name(&s.status)
                    .unwrap_or(PaymentSessionStatus::Pending);
                PaymentSessionDTO {
                    id: s.id.to_string(),
                    payment_collection_id: s.payment_collection_id,
                    provider_id: s.provider_id,
                    amount: s.amount,
                    currency_code: s.currency_code,
                    status,
                    data: s.data,
                    context: s.context,
                    authorized_at: s.authorized_at,
                    status_observed_at: s.status_observed_at,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                }
            })
            .collect())
    }

    async fn get_payments_for_collection(&self, collection_id: &str) -> Result<Vec<PaymentDTO>> {
        let spec = FilterSpec::new()
            .with_predicate(
                "payment_collection_id",
                FilterOp::Eq(collection_id.to_string()),
            )
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let payments = self.payments.list(None, &spec).await?;

        let mut list = Vec::new();
        for p in payments {
            let p_id_str = p.id.to_string();
            let captures = self.get_captures_for_payment(&p_id_str).await?;
            let refunds = self.get_refunds_for_payment(&p_id_str).await?;
            let captured_amount = captures.iter().map(|c| c.amount).sum();
            let refunded_amount = refunds.iter().map(|r| r.amount).sum();

            list.push(PaymentDTO {
                id: p_id_str,
                payment_collection_id: p.payment_collection_id,
                payment_session_id: p.payment_session_id,
                provider_id: p.provider_id,
                currency_code: p.currency_code,
                amount: p.amount,
                captured_at: p.captured_at,
                canceled_at: p.canceled_at,
                captures,
                refunds,
                captured_amount,
                refunded_amount,
                data: p.data,
                metadata: p.metadata,
                created_at: p.created_at,
                updated_at: p.updated_at,
            });
        }
        Ok(list)
    }

    async fn get_captures_for_payment(&self, payment_id: &str) -> Result<Vec<CaptureDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("payment_id", FilterOp::Eq(payment_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let captures = self.captures.list(None, &spec).await?;

        Ok(captures
            .into_iter()
            .map(|c| CaptureDTO {
                id: c.id.to_string(),
                payment_id: c.payment_id,
                amount: c.amount,
                created_by: c.created_by,
                metadata: c.metadata,
                created_at: c.created_at,
            })
            .collect())
    }

    async fn get_refunds_for_payment(&self, payment_id: &str) -> Result<Vec<RefundDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("payment_id", FilterOp::Eq(payment_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let refunds = self.refunds.list(None, &spec).await?;

        Ok(refunds
            .into_iter()
            .map(|r| RefundDTO {
                id: r.id.to_string(),
                payment_id: r.payment_id,
                amount: r.amount,
                refund_reason_id: r.refund_reason_id,
                note: r.note,
                created_by: r.created_by,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Payment reconciliation job comparing PSP settlement records with Stheno payment ledgers (PAYMENTS §9, Task P8-22).
    /// RULE: Drift is detected and recorded, but NEVER auto-corrects without manual review.
    pub async fn run_reconciliation(
        &self,
        provider_id: &str,
        psp_report: Vec<PspReportItem>,
    ) -> Result<ReconciliationReportDTO> {
        let spec = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);
        let payments = self.payments.list(None, &spec).await?;
        let mut matched_count = 0;
        let mut discrepancies = Vec::new();

        for psp_item in &psp_report {
            let internal_payment = if let Some(ref pid) = psp_item.payment_id {
                payments.iter().find(|p| p.id.to_string() == *pid)
            } else {
                payments.iter().find(|p| {
                    p.data
                        .as_ref()
                        .and_then(|d| d.get("provider_payment_id"))
                        .and_then(|v| v.as_str())
                        == Some(&psp_item.provider_payment_id)
                })
            };

            match internal_payment {
                Some(p) => {
                    let mut has_discrepancy = false;
                    let payment_id_str = p.id.to_string();
                    let captures = self.get_captures_for_payment(&payment_id_str).await?;
                    let captured_total: Decimal = captures.iter().map(|c| c.amount).sum();
                    let p_effective_amount = if p.captured_at.is_some() || !captures.is_empty() {
                        captured_total
                    } else {
                        p.amount
                    };

                    if p_effective_amount != psp_item.amount {
                        has_discrepancy = true;
                        discrepancies.push(ReconciliationDiscrepancy {
                            payment_id: payment_id_str.clone(),
                            provider_payment_id: psp_item.provider_payment_id.clone(),
                            field: "amount".to_string(),
                            internal_value: p_effective_amount.to_string(),
                            psp_value: psp_item.amount.to_string(),
                            description: format!(
                                "Amount mismatch: internal {} vs PSP settlement {}",
                                p_effective_amount, psp_item.amount
                            ),
                        });
                    }

                    let is_status_match = match (p.captured_at.is_some(), psp_item.status.as_str())
                    {
                        (true, "captured") => true,
                        (false, "authorized") => true,
                        _ => psp_item.status == "captured" && !captures.is_empty(),
                    };

                    if !is_status_match {
                        has_discrepancy = true;
                        discrepancies.push(ReconciliationDiscrepancy {
                            payment_id: payment_id_str.clone(),
                            provider_payment_id: psp_item.provider_payment_id.clone(),
                            field: "status".to_string(),
                            internal_value: if p.captured_at.is_some() || !captures.is_empty() {
                                "captured"
                            } else {
                                "authorized"
                            }
                            .to_string(),
                            psp_value: psp_item.status.clone(),
                            description: format!("Status mismatch for payment {payment_id_str}"),
                        });
                    }

                    if !has_discrepancy {
                        matched_count += 1;
                    }
                }
                None => {
                    discrepancies.push(ReconciliationDiscrepancy {
                        payment_id: "unknown".to_string(),
                        provider_payment_id: psp_item.provider_payment_id.clone(),
                        field: "missing_in_stheno".to_string(),
                        internal_value: "none".to_string(),
                        psp_value: psp_item.amount.to_string(),
                        description: format!(
                            "PSP record {} exists in gateway report but not in Stheno records",
                            psp_item.provider_payment_id
                        ),
                    });
                }
            }
        }

        Ok(ReconciliationReportDTO {
            id: Id::<Payment>::new().to_string(),
            provider_id: provider_id.to_string(),
            total_records_checked: psp_report.len(),
            matched_count,
            discrepancy_count: discrepancies.len(),
            discrepancies,
            generated_at: Utc::now(),
        })
    }

    // ── COD Settlement & Courier Reconciliation (PHASE3_TASKS §P24) ─────────

    /// Delivery confirmation authorises capture on the COD provider.
    /// One flow, courier-agnostic (P24-03, BP6).
    /// Idempotent on `delivery_event_id`: duplicate delivery events return existing capture.
    pub async fn confirm_cod_delivery(
        &self,
        input: CodDeliveryConfirmationInput,
    ) -> Result<CaptureDTO> {
        let payment_id = if let Some(ref pid) = input.payment_id {
            pid.clone()
        } else if let Some(ref sid) = input.payment_session_id {
            let spec = FilterSpec::new()
                .with_predicate("payment_session_id", FilterOp::Eq(sid.clone()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            let p_list = self.payments.list(None, &spec).await?;
            let p = p_list.into_iter().next().ok_or_else(|| Error::NotFound {
                entity: "payment",
                id: sid.clone(),
            })?;
            p.id.to_string()
        } else {
            return Err(Error::Invalid {
                code: "missing_payment_identifier",
                message: "Must provide either payment_id or payment_session_id".to_string(),
                field: Some("payment_id".to_string()),
            });
        };

        let lock = self.get_or_create_payment_lock(&payment_id);
        let _guard = lock.lock().await;

        let payment = self.get_payment(&payment_id).await?;

        // If payment is already canceled or returned, do NOT capture (BP6, P24-04)
        if payment.canceled_at.is_some() {
            return Err(Error::Conflict {
                code: "cod_order_canceled",
                message: format!(
                    "Cannot capture delivery for canceled or returned COD payment {payment_id}"
                ),
            });
        }

        // Check if this delivery_event_id was already captured (Idempotent replay - BP6)
        let captures = self.get_captures_for_payment(&payment_id).await?;
        for cap in &captures {
            if let Some(ref meta) = cap.metadata {
                if meta.get("delivery_event_id").and_then(|v| v.as_str())
                    == Some(&input.delivery_event_id)
                {
                    return Ok(cap.clone());
                }
            }
        }

        let current_captured = payment.captured_amount;
        let amount_to_capture = input.collected_amount.unwrap_or(payment.amount - current_captured);

        if amount_to_capture <= dec!(0) {
            return Err(Error::Invalid {
                code: "invalid_capture_amount",
                message: "Capture amount must be greater than zero".to_string(),
                field: None,
            });
        }

        if current_captured + amount_to_capture > payment.amount {
            return Err(Error::Conflict {
                code: "over_capture_prevented",
                message: format!(
                    "Cannot capture {}: existing captured {} + {} exceeds authorized amount {}",
                    amount_to_capture, current_captured, amount_to_capture, payment.amount
                ),
            });
        }

        let tenant_id = stheno_db::tenant::current()?;
        let provider = self.get_provider(&tenant_id, &payment.provider_id).await?;
        let capture_input = CaptureInput {
            payment_id: payment.id.clone(),
            amount: amount_to_capture,
            currency_code: payment.currency_code.clone(),
            data: payment.data.clone(),
        };
        let capture_payload = serde_json::to_value(&capture_input).unwrap_or_default();
        let capture_provider = provider.clone();

        let idempotency_key = format!(
            "cod_delivery:{}:{}:{}",
            payment.id, input.delivery_event_id, amount_to_capture
        );

        let _capture_out = self
            .idempotent_call(
                &idempotency_key,
                &payment.provider_id,
                "capture",
                "payment",
                &payment.id,
                &capture_payload,
                move |_key| async move { capture_provider.capture(capture_input).await },
            )
            .await?;

        let mut capture_meta = input.metadata.clone().unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = capture_meta.as_object_mut() {
            obj.insert(
                "delivery_event_id".to_string(),
                serde_json::Value::String(input.delivery_event_id.clone()),
            );
            if let Some(ref courier) = input.courier_id {
                obj.insert(
                    "courier_id".to_string(),
                    serde_json::Value::String(courier.clone()),
                );
            }
            if let Some(ref consignment) = input.consignment_id {
                obj.insert(
                    "consignment_id".to_string(),
                    serde_json::Value::String(consignment.clone()),
                );
            }
            if let Some(ref tracking) = input.tracking_code {
                obj.insert(
                    "tracking_code".to_string(),
                    serde_json::Value::String(tracking.clone()),
                );
            }
        }

        let capture = Capture {
            id: Id::new(),
            payment_id: payment.id.clone(),
            amount: amount_to_capture,
            created_by: Some(input.courier_id.unwrap_or_else(|| "courier".to_string())),
            metadata: Some(capture_meta),
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created_capture = self.captures.create(None, capture).await?;

        // Update payment collection captured_amount & status
        let mut col = self
            .collections
            .retrieve(None, &payment.payment_collection_id)
            .await?;
        col.captured_amount += amount_to_capture;
        if col.captured_amount >= col.amount {
            col.status = "captured".to_string();
        }
        col.updated_at = Utc::now();
        let _ = self
            .collections
            .update(None, &payment.payment_collection_id, col)
            .await;

        // Update payment captured_at
        let mut p = self.payments.retrieve(None, &payment.id).await?;
        if current_captured + amount_to_capture >= payment.amount {
            p.captured_at = Some(Utc::now());
        }
        p.updated_at = Utc::now();
        let _ = self.payments.update(None, &payment.id, p).await;

        // Update payment session status to captured
        if let Some(ref sid) = payment.payment_session_id {
            if let Ok(mut s) = self.sessions.retrieve(None, sid).await {
                s.status = "captured".to_string();
                s.updated_at = Utc::now();
                let _ = self.sessions.update(None, sid, s).await;
            }
        }

        Ok(CaptureDTO {
            id: created_capture.id.to_string(),
            payment_id: created_capture.payment_id,
            amount: created_capture.amount,
            created_by: created_capture.created_by,
            metadata: created_capture.metadata,
            created_at: created_capture.created_at,
        })
    }

    /// Returned COD order produces no capture and cancels expectations (P24-04, BP6).
    pub async fn process_cod_return(&self, input: CodReturnInput) -> Result<CodReturnDTO> {
        let payment_id = if let Some(ref pid) = input.payment_id {
            pid.clone()
        } else if let Some(ref sid) = input.payment_session_id {
            let spec = FilterSpec::new()
                .with_predicate("payment_session_id", FilterOp::Eq(sid.clone()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);
            let p_list = self.payments.list(None, &spec).await?;
            let p = p_list.into_iter().next().ok_or_else(|| Error::NotFound {
                entity: "payment",
                id: sid.clone(),
            })?;
            p.id.to_string()
        } else {
            return Err(Error::Invalid {
                code: "missing_payment_identifier",
                message: "Must provide either payment_id or payment_session_id".to_string(),
                field: Some("payment_id".to_string()),
            });
        };

        let lock = self.get_or_create_payment_lock(&payment_id);
        let _guard = lock.lock().await;

        let payment = self.get_payment(&payment_id).await?;
        let now = Utc::now();

        let was_captured = payment.captured_amount > dec!(0);
        let refunded_amount = if was_captured {
            payment.captured_amount
        } else {
            dec!(0)
        };

        // Mark payment canceled
        let mut p = self.payments.retrieve(None, &payment.id).await?;
        p.canceled_at = Some(now);
        p.updated_at = now;
        let _ = self.payments.update(None, &payment.id, p).await;

        // If it was captured, record a refund so net revenue is 0
        if was_captured {
            let refund = Refund {
                id: Id::new(),
                payment_id: payment.id.clone(),
                amount: refunded_amount,
                refund_reason_id: None,
                note: Some(format!(
                    "COD parcel returned: event {}",
                    input.return_event_id
                )),
                created_by: input.courier_id.or_else(|| Some("courier_return".to_string())),
                metadata: input.metadata.clone(),
                created_at: now,
                deleted_at: None,
            };
            let _ = self.refunds.create(None, refund).await?;
        }

        // Update collection status and refunded_amount
        if let Ok(mut col) = self
            .collections
            .retrieve(None, &payment.payment_collection_id)
            .await
        {
            if was_captured {
                col.refunded_amount += refunded_amount;
            }
            col.status = "canceled".to_string();
            col.updated_at = now;
            let _ = self
                .collections
                .update(None, &payment.payment_collection_id, col)
                .await;
        }

        // Update payment session to canceled
        if let Some(ref sid) = payment.payment_session_id {
            if let Ok(mut s) = self.sessions.retrieve(None, sid).await {
                s.status = "canceled".to_string();
                s.updated_at = now;
                let _ = self.sessions.update(None, sid, s).await;
            }
        }

        Ok(CodReturnDTO {
            payment_id: payment.id,
            status: "canceled".to_string(),
            return_event_id: input.return_event_id,
            was_captured,
            refunded_amount,
            canceled_at: now,
        })
    }

    /// Settlement report import per courier, idempotent on re-import (P24-05, BP7).
    pub async fn import_settlement_report(
        &self,
        input: ImportSettlementReportInput,
    ) -> Result<ImportSettlementReportOutput> {
        let tenant_id = stheno_db::tenant::current()?;
        stheno_db::tenant::validate(&tenant_id)?;

        // Find or create cycle
        let cycle_spec = FilterSpec::new()
            .with_predicate("provider_id", FilterOp::Eq(input.provider_id.clone()))
            .with_predicate("courier_cycle_id", FilterOp::Eq(input.courier_cycle_id.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        let existing_cycles = self.settlement_cycles.list(None, &cycle_spec).await?;

        let (mut cycle, is_reimport) = if let Some(c) = existing_cycles.into_iter().next() {
            (c, true)
        } else {
            let new_cycle = CourierSettlementCycle {
                id: Id::new(),
                provider_id: input.provider_id.clone(),
                courier_cycle_id: input.courier_cycle_id.clone(),
                cycle_start: input.cycle_start,
                cycle_end: input.cycle_end,
                total_collected: dec!(0),
                total_fees: dec!(0),
                total_remitted: dec!(0),
                currency: input.currency.unwrap_or_else(|| "BDT".to_string()),
                status: "open".to_string(),
                raw_report: input.raw_report,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            let created = self.settlement_cycles.create(None, new_cycle).await?;
            (created, false)
        };

        // Load existing lines for this cycle to detect overlaps / duplicates
        let cycle_id_str = cycle.id.to_string();
        let lines_spec = FilterSpec::new()
            .with_predicate("cycle_id", FilterOp::Eq(cycle_id_str.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull);
        let existing_lines = self.settlement_lines.list(None, &lines_spec).await?;
        let existing_consignments: std::collections::HashSet<String> =
            existing_lines.iter().map(|l| l.consignment_id.clone()).collect();

        let mut imported_count = 0;
        let mut skipped_count = 0;

        for line_input in input.lines {
            if existing_consignments.contains(&line_input.consignment_id) {
                skipped_count += 1;
                continue;
            }

            let new_line = CourierSettlementLine {
                id: Id::new(),
                cycle_id: cycle_id_str.clone(),
                consignment_id: line_input.consignment_id,
                tracking_code: line_input.tracking_code,
                order_id: line_input.order_id,
                collected_amount: line_input.collected_amount,
                courier_fee: line_input.courier_fee,
                cod_fee: line_input.cod_fee,
                remitted_amount: line_input.remitted_amount,
                reconciliation_status: "unmatched".to_string(),
                discrepancy_reason: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            self.settlement_lines.create(None, new_line).await?;
            imported_count += 1;
        }

        // Recompute cycle totals from all lines in cycle
        let all_lines = self.settlement_lines.list(None, &lines_spec).await?;
        let mut tot_coll = dec!(0);
        let mut tot_fees = dec!(0);
        let mut tot_remit = dec!(0);
        for l in &all_lines {
            tot_coll += l.collected_amount;
            tot_fees += l.courier_fee + l.cod_fee;
            tot_remit += l.remitted_amount;
        }

        cycle.total_collected = tot_coll;
        cycle.total_fees = tot_fees;
        cycle.total_remitted = tot_remit;
        cycle.updated_at = Utc::now();
        let _ = self
            .settlement_cycles
            .update(None, &cycle_id_str, cycle.clone())
            .await;

        Ok(ImportSettlementReportOutput {
            cycle_id: cycle_id_str,
            provider_id: cycle.provider_id,
            courier_cycle_id: cycle.courier_cycle_id,
            imported_lines: imported_count,
            skipped_duplicate_lines: skipped_count,
            total_collected: tot_coll,
            total_fees: tot_fees,
            total_remitted: tot_remit,
            is_reimport,
        })
    }

    /// Reconciliation run writes to `payment.payment_reconciliation_run` with three buckets:
    /// matched, collected_not_remitted, remitted_not_matched (and discrepant) (P24-06).
    /// Asserts decimal fee balance: sum(remitted) + sum(fees) == sum(collected) (P24-07, BP8).
    /// Deliberate mismatches produce discrepancy rows (P24-08).
    pub async fn run_courier_reconciliation(
        &self,
        cycle_id: &str,
    ) -> Result<CourierReconciliationReportDTO> {
        let tenant_id = stheno_db::tenant::current()?;
        stheno_db::tenant::validate(&tenant_id)?;

        let mut cycle = self.settlement_cycles.retrieve(None, cycle_id).await?;
        let lines_spec = FilterSpec::new()
            .with_predicate("cycle_id", FilterOp::Eq(cycle_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull);
        let lines = self.settlement_lines.list(None, &lines_spec).await?;

        let captures_spec =
            FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        let all_captures = self.captures.list(None, &captures_spec).await?;

        let mut matched = Vec::new();
        let mut remitted_not_matched = Vec::new();
        let mut discrepant = Vec::new();
        let mut matched_consignments = std::collections::HashSet::new();

        for mut line in lines {
            let line_id = line.id.to_string();
            let total_fee = line.courier_fee + line.cod_fee;

            // Fee accounting check: remitted + fees == collected (BP8, P24-07)
            let fee_balanced = (line.remitted_amount + total_fee) == line.collected_amount;

            // Find matching capture by consignment_id or tracking_code
            let matching_cap = all_captures.iter().find(|cap| {
                if let Some(ref meta) = cap.metadata {
                    if let Some(cid) = meta.get("consignment_id").and_then(|v| v.as_str()) {
                        if cid == line.consignment_id {
                            return true;
                        }
                    }
                    if let Some(ref trk) = line.tracking_code {
                        if let Some(ctrk) = meta.get("tracking_code").and_then(|v| v.as_str()) {
                            if ctrk == trk {
                                return true;
                            }
                        }
                    }
                }
                false
            });

            if let Some(cap) = matching_cap {
                matched_consignments.insert(line.consignment_id.clone());

                if !fee_balanced {
                    let reason = format!(
                        "Fee accounting mismatch: remitted ({}) + fees ({}) != collected ({})",
                        line.remitted_amount, total_fee, line.collected_amount
                    );
                    line.reconciliation_status = "discrepant".to_string();
                    line.discrepancy_reason = Some(reason.clone());
                    let _ = self.settlement_lines.update(None, &line_id, line.clone()).await;

                    discrepant.push(CourierReconciliationBucketItem {
                        consignment_id: line.consignment_id,
                        tracking_code: line.tracking_code,
                        order_id: line.order_id,
                        payment_id: Some(cap.payment_id.clone()),
                        expected_amount: cap.amount,
                        collected_amount: line.collected_amount,
                        remitted_amount: line.remitted_amount,
                        fee_amount: total_fee,
                        reason: Some(reason),
                    });
                } else if cap.amount != line.collected_amount {
                    let reason = format!(
                        "Collected amount mismatch: captured ({}) != courier collected ({})",
                        cap.amount, line.collected_amount
                    );
                    line.reconciliation_status = "discrepant".to_string();
                    line.discrepancy_reason = Some(reason.clone());
                    let _ = self.settlement_lines.update(None, &line_id, line.clone()).await;

                    discrepant.push(CourierReconciliationBucketItem {
                        consignment_id: line.consignment_id,
                        tracking_code: line.tracking_code,
                        order_id: line.order_id,
                        payment_id: Some(cap.payment_id.clone()),
                        expected_amount: cap.amount,
                        collected_amount: line.collected_amount,
                        remitted_amount: line.remitted_amount,
                        fee_amount: total_fee,
                        reason: Some(reason),
                    });
                } else {
                    line.reconciliation_status = "matched".to_string();
                    line.discrepancy_reason = None;
                    let _ = self.settlement_lines.update(None, &line_id, line.clone()).await;

                    matched.push(CourierReconciliationBucketItem {
                        consignment_id: line.consignment_id,
                        tracking_code: line.tracking_code,
                        order_id: line.order_id,
                        payment_id: Some(cap.payment_id.clone()),
                        expected_amount: cap.amount,
                        collected_amount: line.collected_amount,
                        remitted_amount: line.remitted_amount,
                        fee_amount: total_fee,
                        reason: None,
                    });
                }
            } else {
                line.reconciliation_status = "unmatched".to_string();
                line.discrepancy_reason =
                    Some("No matching internal captured payment found".to_string());
                let _ = self.settlement_lines.update(None, &line_id, line.clone()).await;

                remitted_not_matched.push(CourierReconciliationBucketItem {
                    consignment_id: line.consignment_id,
                    tracking_code: line.tracking_code,
                    order_id: line.order_id,
                    payment_id: None,
                    expected_amount: dec!(0),
                    collected_amount: line.collected_amount,
                    remitted_amount: line.remitted_amount,
                    fee_amount: total_fee,
                    reason: Some("No matching internal capture".to_string()),
                });
            }
        }

        // Captured COD payments collected by this courier but with no remittance line: collected_not_remitted
        let mut collected_not_remitted = Vec::new();
        for cap in &all_captures {
            if let Some(ref meta) = cap.metadata {
                let courier = meta.get("courier_id").and_then(|v| v.as_str());
                let cid = meta.get("consignment_id").and_then(|v| v.as_str());
                if courier == Some(&cycle.provider_id) {
                    if let Some(consignment_id) = cid {
                        if !matched_consignments.contains(consignment_id) {
                            collected_not_remitted.push(CourierReconciliationBucketItem {
                                consignment_id: consignment_id.to_string(),
                                tracking_code: meta
                                    .get("tracking_code")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                order_id: None,
                                payment_id: Some(cap.payment_id.clone()),
                                expected_amount: cap.amount,
                                collected_amount: cap.amount,
                                remitted_amount: dec!(0),
                                fee_amount: dec!(0),
                                reason: Some(
                                    "Delivered and collected but no courier remittance received"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        let total_checked = matched.len()
            + remitted_not_matched.len()
            + discrepant.len()
            + collected_not_remitted.len();
        let drift_count = (remitted_not_matched.len()
            + discrepant.len()
            + collected_not_remitted.len()) as i32;
        let state = if drift_count == 0 {
            "clean"
        } else {
            "drift_found"
        };

        cycle.status = if drift_count == 0 {
            "reconciled".to_string()
        } else {
            "discrepant".to_string()
        };
        cycle.updated_at = Utc::now();
        let _ = self
            .settlement_cycles
            .update(None, cycle_id, cycle.clone())
            .await;

        let buckets = CourierReconciliationBuckets {
            matched,
            collected_not_remitted,
            remitted_not_matched,
            discrepant,
        };

        let drifts_json = serde_json::to_value(&buckets).unwrap_or_default();

        let run_id: Id<PaymentReconciliationRun> = Id::new();
        let run = PaymentReconciliationRun {
            id: run_id,
            provider_id: cycle.provider_id.clone(),
            window_start: cycle.cycle_start.unwrap_or_else(Utc::now),
            window_end: cycle.cycle_end.unwrap_or_else(Utc::now),
            checked_count: total_checked as i32,
            drift_count,
            drifts: Some(drifts_json),
            state: state.to_string(),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
            deleted_at: None,
        };
        let created_run = self.reconciliation_runs.create(None, run).await?;

        Ok(CourierReconciliationReportDTO {
            run_id: created_run.id.to_string(),
            cycle_id: cycle_id.to_string(),
            provider_id: cycle.provider_id,
            total_lines_checked: total_checked,
            matched_count: buckets.matched.len(),
            collected_not_remitted_count: buckets.collected_not_remitted.len(),
            remitted_not_matched_count: buckets.remitted_not_matched.len(),
            discrepancy_count: buckets.discrepant.len(),
            total_collected: cycle.total_collected,
            total_fees: cycle.total_fees,
            total_remitted: cycle.total_remitted,
            state: state.to_string(),
            buckets,
            generated_at: Utc::now(),
        })
    }

    /// Discrepancy alerting: an unmatched line older than the cycle length raises an alert (P24-09).
    pub async fn check_unsettled_line_alerts(
        &self,
        cycle_id: &str,
        cycle_length_days: i64,
    ) -> Result<Vec<DiscrepancyAlertDTO>> {
        let lines_spec = FilterSpec::new()
            .with_predicate("cycle_id", FilterOp::Eq(cycle_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull);
        let lines = self.settlement_lines.list(None, &lines_spec).await?;

        let now = Utc::now();
        let mut alerts = Vec::new();

        for line in lines {
            if line.reconciliation_status == "unmatched"
                || line.reconciliation_status == "discrepant"
            {
                let age = (now - line.created_at).num_days();
                if age >= cycle_length_days {
                    alerts.push(DiscrepancyAlertDTO {
                        line_id: line.id.to_string(),
                        cycle_id: cycle_id.to_string(),
                        consignment_id: line.consignment_id,
                        tracking_code: line.tracking_code,
                        order_id: line.order_id,
                        unremitted_amount: line.collected_amount - line.remitted_amount,
                        age_days: age,
                        alert_level: "critical".to_string(),
                        message: format!(
                            "Unsettled consignment has remained {} for {} days (threshold: {} days)",
                            line.reconciliation_status, age, cycle_length_days
                        ),
                    });
                }
            }
        }
        Ok(alerts)
    }

    pub async fn get_settlement_cycle(&self, id: &str) -> Result<CourierSettlementCycleDTO> {
        let c = self.settlement_cycles.retrieve(None, id).await?;
        Ok(CourierSettlementCycleDTO {
            id: c.id.to_string(),
            provider_id: c.provider_id,
            courier_cycle_id: c.courier_cycle_id,
            cycle_start: c.cycle_start,
            cycle_end: c.cycle_end,
            total_collected: c.total_collected,
            total_fees: c.total_fees,
            total_remitted: c.total_remitted,
            currency: c.currency,
            status: c.status,
            raw_report: c.raw_report,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
    }

    pub async fn list_settlement_cycles(
        &self,
        provider_id: Option<&str>,
    ) -> Result<Vec<CourierSettlementCycleDTO>> {
        let mut spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        if let Some(pid) = provider_id {
            spec = spec.with_predicate("provider_id", FilterOp::Eq(pid.to_string()));
        }
        let list = self.settlement_cycles.list(None, &spec).await?;
        Ok(list
            .into_iter()
            .map(|c| CourierSettlementCycleDTO {
                id: c.id.to_string(),
                provider_id: c.provider_id,
                courier_cycle_id: c.courier_cycle_id,
                cycle_start: c.cycle_start,
                cycle_end: c.cycle_end,
                total_collected: c.total_collected,
                total_fees: c.total_fees,
                total_remitted: c.total_remitted,
                currency: c.currency,
                status: c.status,
                raw_report: c.raw_report,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect())
    }

    pub async fn list_settlement_lines(
        &self,
        cycle_id: &str,
    ) -> Result<Vec<CourierSettlementLineDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("cycle_id", FilterOp::Eq(cycle_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull);
        let list = self.settlement_lines.list(None, &spec).await?;
        Ok(list
            .into_iter()
            .map(|l| CourierSettlementLineDTO {
                id: l.id.to_string(),
                cycle_id: l.cycle_id,
                consignment_id: l.consignment_id,
                tracking_code: l.tracking_code,
                order_id: l.order_id,
                collected_amount: l.collected_amount,
                courier_fee: l.courier_fee,
                cod_fee: l.cod_fee,
                remitted_amount: l.remitted_amount,
                reconciliation_status: l.reconciliation_status,
                discrepancy_reason: l.discrepancy_reason,
                created_at: l.created_at,
                updated_at: l.updated_at,
            })
            .collect())
    }
}

pub fn compute_sha256(val: &serde_json::Value) -> String {
    let canonical = serde_json::to_string(val).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

/// Adapts an already-built provider to the factory interface. Carries no credential
/// schema: the instance is fully configured before it ever reaches the service.
struct FixedProviderFactory {
    provider: Arc<dyn PaymentProvider>,
}

impl PaymentProviderFactory for FixedProviderFactory {
    fn provider_id(&self) -> &str {
        self.provider.id()
    }

    fn credential_schema(&self) -> &'static [CredentialField] {
        &[]
    }

    fn build(&self, _creds: &CredentialMap) -> Result<Arc<dyn PaymentProvider>> {
        Ok(self.provider.clone())
    }
}
