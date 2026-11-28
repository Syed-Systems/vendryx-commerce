//! Tenant-owned encrypted provider and Meta channel configuration (ADR-032 phase 32.5).

use std::collections::BTreeMap;

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::diesel::result::OptionalExtension;
use stheno_db::diesel::sql_types::{Binary, Bool, Integer, Nullable, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{DbPool, Error, Result};

/// Allowlisted configuration update. Secret values are never serialized back to callers.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveAiConfiguration {
    pub kind: String,
    pub account_id: String,
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
    pub enabled: bool,
}

/// Redacted configuration status safe for admin API responses.
#[derive(Debug, Clone, Serialize)]
pub struct AiConfigurationStatus {
    pub id: String,
    pub kind: String,
    pub account_id: String,
    pub configured: bool,
    pub enabled: bool,
    pub version: i32,
    pub webhook_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Non-secret, tenant-scoped record of an owner-managed source connection change.
#[derive(Debug, Clone, Serialize)]
pub struct SourceConnectionAudit {
    pub action: String,
    pub actor_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct SourceConnectionAuditRow {
    #[diesel(sql_type = Text)]
    action: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

/// Runtime provider values; intentionally not `Debug` or serializable.
pub struct AiProviderRuntimeConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
}

/// Runtime Meta credentials returned only to the webhook/outbound adapter.
pub struct MetaRuntimeConfig {
    pub tenant_id: String,
    pub channel: String,
    pub account_id: String,
    pub webhook_id: String,
    pub app_secret: String,
    pub verify_token: String,
    pub access_token: String,
}

/// Minimum secret-bearing values needed by the outbound WhatsApp adapter.
pub struct WhatsAppSenderRuntimeConfig {
    pub phone_number_id: String,
    pub access_token: String,
}

/// Encrypted merchant-authorized Search Console credentials.
pub struct GoogleSearchConsoleRuntimeConfig {
    pub property: String,
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

#[derive(QueryableByName)]
struct ConfigurationRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    webhook_id: Option<String>,
    #[diesel(sql_type = Binary)]
    ciphertext: Vec<u8>,
    #[diesel(sql_type = Binary)]
    nonce: Vec<u8>,
    #[diesel(sql_type = Bool)]
    has_credentials: bool,
    #[diesel(sql_type = Integer)]
    version: i32,
    #[diesel(sql_type = Bool)]
    enabled: bool,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct WebhookOwnerRow {
    #[diesel(sql_type = Text)]
    tenant_id: String,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    account_id: String,
}

#[derive(QueryableByName)]
struct WebhookRegisteredRow {
    #[diesel(sql_type = Bool)]
    registered: bool,
}

/// Encrypted per-tenant AI and social channel configuration storage.
#[derive(Clone)]
pub struct AiConfigurationService {
    pool: DbPool,
}

impl AiConfigurationService {
    /// Creates the service over the application pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Records a one-time owner-authorized OAuth state with a ten-minute lifetime.
    pub async fn create_search_console_oauth_state(
        &self,
        nonce: &str,
        property: &str,
        actor_id: &str,
    ) -> Result<()> {
        if nonce.len() != 26
            || nonce.chars().any(char::is_control)
            || actor_id.trim().is_empty()
            || actor_id.len() > 128
            || actor_id.chars().any(char::is_control)
        {
            return Err(invalid("Search Console OAuth state is invalid."));
        }
        let tenant_id = stheno_db::tenant::current()?;
        let property = normalize_search_console_property(property)?;
        let nonce = nonce.to_string();
        let actor_id = actor_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "DELETE FROM ai.search_console_oauth_state WHERE expires_at <= now()",
                    )
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    diesel::sql_query(
                        "INSERT INTO ai.search_console_oauth_state (nonce, property, actor_id) \
                         VALUES ($1, $2, $3)",
                    )
                    .bind::<Text, _>(nonce)
                    .bind::<Text, _>(property)
                    .bind::<Text, _>(actor_id)
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

    /// Atomically consumes one unexpired OAuth state; every valid state succeeds at most once.
    pub async fn consume_search_console_oauth_state(
        &self,
        nonce: &str,
        property: &str,
        actor_id: &str,
    ) -> Result<bool> {
        if nonce.len() != 26 || nonce.chars().any(char::is_control) {
            return Ok(false);
        }
        let tenant_id = stheno_db::tenant::current()?;
        let property = normalize_search_console_property(property)?;
        let nonce = nonce.to_string();
        let actor_id = actor_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "DELETE FROM ai.search_console_oauth_state \
                         WHERE nonce = $1 AND property = $2 AND actor_id = $3 \
                           AND expires_at > now()",
                    )
                    .bind::<Text, _>(nonce)
                    .bind::<Text, _>(property)
                    .bind::<Text, _>(actor_id)
                    .execute(conn)
                    .await
                    .map(|count| count == 1)
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }

    /// Creates or rotates one allowlisted provider/channel configuration.
    /// Empty secret values preserve previously stored values during rotation updates.
    pub async fn save(&self, input: SaveAiConfiguration) -> Result<AiConfigurationStatus> {
        self.save_inner(input, None).await
    }

    /// Saves an owner-managed Search Console connection and audits it in the same transaction.
    pub async fn save_with_actor(
        &self,
        input: SaveAiConfiguration,
        actor_id: &str,
    ) -> Result<AiConfigurationStatus> {
        if actor_id.trim().is_empty()
            || actor_id.len() > 128
            || actor_id.chars().any(char::is_control)
        {
            return Err(invalid("The source configuration actor is invalid."));
        }
        if !matches!(
            input.kind.to_ascii_lowercase().as_str(),
            "search_console" | "meta_insights_facebook" | "meta_insights_instagram"
        ) {
            return Err(invalid(
                "Actor-audited save is reserved for external insight sources.",
            ));
        }
        self.save_inner(input, Some(actor_id.to_string())).await
    }

    async fn save_inner(
        &self,
        input: SaveAiConfiguration,
        audit_actor: Option<String>,
    ) -> Result<AiConfigurationStatus> {
        let tenant_id = stheno_db::tenant::current()?;
        let kind = input.kind.to_ascii_lowercase();
        let account_id = normalize_account(&kind, &input.account_id)?;
        let allowed = allowed_secret_keys(&kind)?;
        let pool = self.pool.clone();
        let tenant_scope = tenant_id.clone();
        let row = stheno_db::tenant::scope(tenant_scope, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    let lock_key = format!("{tenant_id}|{kind}|{account_id}");
                    diesel::sql_query(
                        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
                    )
                    .bind::<Text, _>(&lock_key)
                    .execute(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;

                    let current = diesel::sql_query(
                        "SELECT id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, version, enabled, updated_at \
                           FROM ai.configuration WHERE kind = $1 AND account_id = $2 LIMIT 1",
                    )
                    .bind::<Text, _>(&kind)
                    .bind::<Text, _>(&account_id)
                    .get_result::<ConfigurationRow>(conn)
                    .await
                    .optional()
                    .map_err(|error| Error::Database(error.to_string()))?;
                    let mut secrets = match current.as_ref() {
                        Some(row) => decrypt_values(&tenant_id, &kind, &account_id, row)?,
                        None => BTreeMap::new(),
                    };
                    for (key, value) in input.secrets {
                        if !allowed.iter().any(|allowed_key| *allowed_key == key) {
                            return Err(invalid("Unknown secret field for this configuration."));
                        }
                        if value.len() > 8192 || value.chars().any(char::is_control) {
                            return Err(invalid(
                                "Secret values are malformed or exceed the size limit.",
                            ));
                        }
                        if !value.trim().is_empty() {
                            secrets.insert(key, value);
                        }
                    }
                    validate_required_values(&kind, &secrets, input.enabled)?;
                    let webhook_id = match kind.as_str() {
                        "llm" | "search_console" | "meta_insights_facebook" | "meta_insights_instagram" => None,
                        _ => current
                            .as_ref()
                            .and_then(|row| row.webhook_id.clone())
                            .or_else(|| Some(ulid::Ulid::new().to_string())),
                    };
                    let registry_values = webhook_id.as_ref().map(|id| {
                        (id.clone(), kind.clone(), account_id.clone(), input.enabled)
                    });
                    let id = current
                        .as_ref()
                        .map(|row| row.id.clone())
                        .unwrap_or_else(|| format!("aicfg_{}", ulid::Ulid::new()));
                    let (ciphertext, nonce) =
                        encrypt_values(&tenant_id, &kind, &account_id, &secrets)?;
                    let row = diesel::sql_query(
                        "INSERT INTO ai.configuration \
                         (id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, enabled) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                         ON CONFLICT (tenant_id, kind, account_id) DO UPDATE SET \
                           webhook_id = COALESCE(ai.configuration.webhook_id, EXCLUDED.webhook_id), \
                           ciphertext = EXCLUDED.ciphertext, nonce = EXCLUDED.nonce, \
                           has_credentials = EXCLUDED.has_credentials, \
                           enabled = EXCLUDED.enabled, version = ai.configuration.version + 1, updated_at = now() \
                         RETURNING id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, version, enabled, updated_at",
                    )
                    .bind::<Text, _>(id)
                    .bind::<Text, _>(&kind)
                    .bind::<Text, _>(&account_id)
                    .bind::<Nullable<Text>, _>(webhook_id)
                    .bind::<Binary, _>(ciphertext)
                    .bind::<Binary, _>(nonce)
                    .bind::<Bool, _>(!secrets.is_empty())
                    .bind::<Bool, _>(input.enabled)
                    .get_result::<ConfigurationRow>(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))?;
                    if let Some((webhook_id, kind, account_id, enabled)) = registry_values {
                        let registered = diesel::sql_query(
                            "SELECT ai.register_webhook($1, $2, $3, $4, $5) AS registered",
                        )
                        .bind::<Text, _>(webhook_id)
                        .bind::<Text, _>(&tenant_id)
                        .bind::<Text, _>(kind)
                        .bind::<Text, _>(account_id)
                        .bind::<Bool, _>(enabled)
                        .get_result::<WebhookRegisteredRow>(conn)
                        .await
                        .map_err(|error| Error::Database(error.to_string()))?
                        .registered;
                        if !registered {
                            return Err(Error::Conflict {
                                code: "ai_webhook_slug_conflict",
                                message: "Could not allocate a unique webhook identifier.".to_string(),
                            });
                        }
                    }
                    let audit_source = match kind.as_str() {
                        "search_console" => Some("search_console"),
                        "meta_insights_facebook" => Some("facebook_page"),
                        "meta_insights_instagram" => Some("instagram_account"),
                        _ => None,
                    };
                    if let (Some(source), Some(actor_id)) = (audit_source, audit_actor) {
                            let action = if !input.enabled {
                                "disabled"
                            } else if current.is_none() {
                                "connected"
                            } else {
                                "updated"
                            };
                            diesel::sql_query(
                                "INSERT INTO ai.source_connection_audit \
                                 (id, source, account_id, action, actor_id) VALUES ($1, $2, $3, $4, $5)",
                            )
                            .bind::<Text, _>(format!("srcaudit_{}", ulid::Ulid::new()))
                            .bind::<Text, _>(source)
                            .bind::<Text, _>(&account_id)
                            .bind::<Text, _>(action)
                            .bind::<Text, _>(actor_id)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                    }
                    Ok(row)
                })
            })
            .await
        })
        .await?;
        Ok(status(row))
    }

    /// Lists redacted statuses only; ciphertext and secret values never leave the service.
    pub async fn list(&self) -> Result<Vec<AiConfigurationStatus>> {
        let tenant_id = stheno_db::tenant::current()?;
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, version, enabled, updated_at \
                           FROM ai.configuration ORDER BY kind, account_id LIMIT 32",
                    )
                    .load::<ConfigurationRow>(conn)
                    .await
                    .map(|rows| rows.into_iter().map(status).collect())
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }

    /// Loads the configured LLM provider for the active tenant, if enabled.
    pub async fn load_provider(&self) -> Result<Option<AiProviderRuntimeConfig>> {
        let tenant_id = stheno_db::tenant::current()?;
        let Some(row) = self.load_row(&tenant_id, "llm", "default").await? else {
            return Ok(None);
        };
        if !row.enabled {
            return Ok(None);
        }
        let values = decrypt_values(&tenant_id, "llm", "default", &row)?;
        let provider = required_value(&values, "provider")?;
        let endpoint = match provider {
            "deepseek" => "https://api.deepseek.com/v1/chat/completions",
            "qwen" => "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
            // Fixed loopback only: merchants cannot submit arbitrary model endpoints.
            "ollama" => "http://127.0.0.1:11434/v1/chat/completions",
            _ => return Err(invalid("Configured AI provider is unsupported.")),
        };
        Ok(Some(AiProviderRuntimeConfig {
            provider: provider.to_string(),
            endpoint: endpoint.to_string(),
            model: required_value(&values, "model")?.to_string(),
            api_key: required_value(&values, "api_key")?.to_string(),
        }))
    }

    /// Loads only the explicitly configured local model for token-free ranking requests.
    pub async fn load_local_ranker(&self) -> Result<Option<AiProviderRuntimeConfig>> {
        Ok(self
            .load_provider()
            .await?
            .filter(|provider| provider.provider == "ollama"))
    }

    /// Loads an enabled, merchant-authorized Search Console property.
    pub async fn load_search_console(
        &self,
        property: &str,
    ) -> Result<Option<GoogleSearchConsoleRuntimeConfig>> {
        let tenant_id = stheno_db::tenant::current()?;
        let property = normalize_search_console_property(property)?;
        let Some(row) = self
            .load_row(&tenant_id, "search_console", &property)
            .await?
        else {
            return Ok(None);
        };
        if !row.enabled {
            return Ok(None);
        }
        let values = decrypt_values(&tenant_id, "search_console", &property, &row)?;
        Ok(Some(GoogleSearchConsoleRuntimeConfig {
            property,
            client_id: required_value(&values, "client_id")?.to_string(),
            client_secret: required_value(&values, "client_secret")?.to_string(),
            refresh_token: required_value(&values, "refresh_token")?.to_string(),
        }))
    }

    /// Loads configured Meta account credentials for read-only insights collection.
    pub async fn load_meta_insights(
        &self,
        channel: &str,
        account_id: &str,
    ) -> Result<Option<MetaRuntimeConfig>> {
        let tenant_id = stheno_db::tenant::current()?;
        let channel = channel.to_ascii_lowercase();
        let config_kind = match channel.as_str() {
            "messenger" => "meta_insights_facebook",
            "instagram" => "meta_insights_instagram",
            _ => return Err(invalid("Meta insights channel is unsupported.")),
        };
        let account_id = normalize_account(config_kind, account_id)?;
        let Some(row) = self.load_row(&tenant_id, config_kind, &account_id).await? else {
            return Ok(None);
        };
        if !row.enabled || !row.has_credentials {
            return Ok(None);
        }
        let values = decrypt_values(&tenant_id, config_kind, &account_id, &row)?;
        Ok(Some(MetaRuntimeConfig {
            tenant_id,
            channel,
            account_id,
            webhook_id: String::new(),
            app_secret: required_value(&values, "app_secret")?.to_string(),
            verify_token: String::new(),
            access_token: required_value(&values, "access_token")?.to_string(),
        }))
    }

    /// Lists a bounded, non-secret actor history for one Search Console property.
    pub async fn list_search_console_audit(
        &self,
        property: &str,
        limit: i32,
    ) -> Result<Vec<SourceConnectionAudit>> {
        if !(1..=50).contains(&limit) {
            return Err(invalid("Audit history limit must be between 1 and 50."));
        }
        let tenant_id = stheno_db::tenant::current()?;
        let property = normalize_search_console_property(property)?;
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT action, actor_id, created_at FROM ai.source_connection_audit \
                         WHERE source = 'search_console' AND account_id = $1 \
                         ORDER BY created_at DESC LIMIT $2",
                    )
                    .bind::<Text, _>(property)
                    .bind::<Integer, _>(limit)
                    .load::<SourceConnectionAuditRow>(conn)
                    .await
                    .map(|rows| {
                        rows.into_iter()
                            .map(|row| SourceConnectionAudit {
                                action: row.action,
                                actor_id: row.actor_id,
                                created_at: row.created_at,
                            })
                            .collect()
                    })
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }

    /// Permanently removes one Search Console connection and its encrypted credentials.
    pub async fn disconnect_search_console(&self, property: &str) -> Result<()> {
        self.disconnect_search_console_inner(property, None).await
    }

    /// Permanently removes one Search Console connection and audits the owner action atomically.
    pub async fn disconnect_search_console_with_actor(
        &self,
        property: &str,
        actor_id: &str,
    ) -> Result<()> {
        if actor_id.trim().is_empty()
            || actor_id.len() > 128
            || actor_id.chars().any(char::is_control)
        {
            return Err(invalid("The source configuration actor is invalid."));
        }
        self.disconnect_search_console_inner(property, Some(actor_id.to_string()))
            .await
    }

    async fn disconnect_search_console_inner(
        &self,
        property: &str,
        audit_actor: Option<String>,
    ) -> Result<()> {
        let tenant_id = stheno_db::tenant::current()?;
        let property = normalize_search_console_property(property)?;
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    stheno_db::transaction::with_transaction(conn, move |conn| {
                        Box::pin(async move {
                            diesel::sql_query(
                                "DELETE FROM ai.store_intelligence_source_observation \
                                 WHERE source = 'search_console' AND account_id = $1",
                            )
                            .bind::<Text, _>(&property)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            let removed = diesel::sql_query(
                                "DELETE FROM ai.configuration WHERE kind = 'search_console' AND account_id = $1",
                            )
                            .bind::<Text, _>(&property)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            if removed > 0 {
                                if let Some(actor_id) = audit_actor {
                                    diesel::sql_query(
                                        "INSERT INTO ai.source_connection_audit \
                                         (id, source, account_id, action, actor_id) VALUES ($1, 'search_console', $2, 'disconnected', $3)",
                                    )
                                    .bind::<Text, _>(format!("scaudit_{}", ulid::Ulid::new()))
                                    .bind::<Text, _>(&property)
                                    .bind::<Text, _>(actor_id)
                                    .execute(conn)
                                    .await
                                    .map_err(|error| Error::Database(error.to_string()))?;
                                }
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

    /// Permanently disconnects one insights account, removing credentials and observations atomically.
    pub async fn disconnect_meta_insights_with_actor(
        &self,
        channel: &str,
        account_id: &str,
        actor_id: &str,
    ) -> Result<()> {
        if actor_id.trim().is_empty()
            || actor_id.len() > 128
            || actor_id.chars().any(char::is_control)
        {
            return Err(invalid("The source configuration actor is invalid."));
        }
        let (kind, source) = meta_insights_source(channel)?;
        let account_id = normalize_account(kind, account_id)?;
        let actor_id = actor_id.to_string();
        let tenant_id = stheno_db::tenant::current()?;
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    stheno_db::transaction::with_transaction(conn, move |conn| {
                        Box::pin(async move {
                            diesel::sql_query(
                                "DELETE FROM ai.store_intelligence_source_observation \
                                 WHERE source = $1 AND account_id = $2",
                            )
                            .bind::<Text, _>(source)
                            .bind::<Text, _>(&account_id)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            let removed = diesel::sql_query(
                                "DELETE FROM ai.configuration WHERE kind = $1 AND account_id = $2",
                            )
                            .bind::<Text, _>(kind)
                            .bind::<Text, _>(&account_id)
                            .execute(conn)
                            .await
                            .map_err(|error| Error::Database(error.to_string()))?;
                            if removed > 0 {
                                diesel::sql_query(
                                    "INSERT INTO ai.source_connection_audit \
                                     (id, source, account_id, action, actor_id) \
                                     VALUES ($1, $2, $3, 'disconnected', $4)",
                                )
                                .bind::<Text, _>(format!("srcaudit_{}", ulid::Ulid::new()))
                                .bind::<Text, _>(source)
                                .bind::<Text, _>(&account_id)
                                .bind::<Text, _>(actor_id)
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

    /// Lists recent actor-attributed connection changes for one Meta insights account.
    pub async fn list_meta_insights_audit(
        &self,
        channel: &str,
        account_id: &str,
        limit: i32,
    ) -> Result<Vec<SourceConnectionAudit>> {
        if !(1..=50).contains(&limit) {
            return Err(invalid("Audit history limit must be between 1 and 50."));
        }
        let (kind, source) = meta_insights_source(channel)?;
        let account_id = normalize_account(kind, account_id)?;
        let tenant_id = stheno_db::tenant::current()?;
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant_id, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT action, actor_id, created_at FROM ai.source_connection_audit \
                         WHERE source = $1 AND account_id = $2 \
                         ORDER BY created_at DESC LIMIT $3",
                    )
                    .bind::<Text, _>(source)
                    .bind::<Text, _>(account_id)
                    .bind::<Integer, _>(limit)
                    .load::<SourceConnectionAuditRow>(conn)
                    .await
                    .map(|rows| {
                        rows.into_iter()
                            .map(|row| SourceConnectionAudit {
                                action: row.action,
                                actor_id: row.actor_id,
                                created_at: row.created_at,
                            })
                            .collect()
                    })
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }

    /// Resolves encrypted channel secrets by tenant and opaque webhook ID.
    pub async fn load_meta_webhook(
        &self,
        tenant_id: &str,
        webhook_id: &str,
    ) -> Result<Option<MetaRuntimeConfig>> {
        stheno_db::tenant::validate(tenant_id)?;
        let tenant = tenant_id.to_string();
        let hook = webhook_id.to_string();
        let pool = self.pool.clone();
        let row = stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, version, enabled, updated_at \
                           FROM ai.configuration WHERE webhook_id = $1 LIMIT 1",
                    )
                    .bind::<Text, _>(hook)
                    .get_result::<ConfigurationRow>(conn)
                    .await
                    .optional()
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await?;
        let Some(row) = row.filter(|row| row.enabled && row.kind != "llm") else {
            return Ok(None);
        };
        let values = decrypt_values(&tenant, &row.kind, &row.account_id, &row)?;
        Ok(Some(MetaRuntimeConfig {
            tenant_id: tenant,
            channel: row.kind,
            account_id: row.account_id,
            webhook_id: row.webhook_id.unwrap_or_default(),
            app_secret: required_value(&values, "app_secret")?.to_string(),
            verify_token: required_value(&values, "verify_token")?.to_string(),
            access_token: required_value(&values, "access_token")?.to_string(),
        }))
    }

    /// Loads the sole enabled WhatsApp account for a tenant for outbound notifications.
    /// Multiple active senders are treated as ambiguous and fail closed.
    pub async fn load_enabled_whatsapp_sender(
        &self,
        tenant_id: &str,
    ) -> Result<Option<WhatsAppSenderRuntimeConfig>> {
        stheno_db::tenant::validate(tenant_id)?;
        let tenant = tenant_id.to_string();
        let lookup_tenant = tenant.clone();
        let pool = self.pool.clone();
        let rows = stheno_db::tenant::scope(tenant.clone(), async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, version, enabled, updated_at \
                           FROM ai.configuration \
                          WHERE kind = 'whatsapp' AND enabled = true AND has_credentials = true \
                          ORDER BY updated_at DESC LIMIT 2",
                    )
                    .load::<ConfigurationRow>(conn)
                    .await
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await?;
        if rows.len() > 1 {
            return Err(invalid(
                "Disable extra WhatsApp accounts before choosing an outbound sender.",
            ));
        }
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let values = decrypt_values(&lookup_tenant, "whatsapp", &row.account_id, &row)?;
        Ok(Some(WhatsAppSenderRuntimeConfig {
            phone_number_id: row.account_id,
            access_token: required_value(&values, "access_token")?.to_string(),
        }))
    }

    /// Resolves a random public webhook ID before entering the owning tenant scope.
    pub async fn resolve_webhook_owner(
        &self,
        webhook_id: &str,
    ) -> Result<Option<(String, String, String)>> {
        if webhook_id.len() < 16
            || webhook_id.len() > 64
            || !webhook_id.bytes().all(|byte| byte.is_ascii_alphanumeric())
        {
            return Ok(None);
        }
        let hook = webhook_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope("system_webhook_lookup", async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT tenant_id, kind, account_id FROM ai.resolve_webhook($1)",
                    )
                    .bind::<Text, _>(hook)
                    .get_result::<WebhookOwnerRow>(conn)
                    .await
                    .optional()
                    .map(|row| row.map(|row| (row.tenant_id, row.kind, row.account_id)))
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }

    async fn load_row(
        &self,
        tenant_id: &str,
        kind: &str,
        account_id: &str,
    ) -> Result<Option<ConfigurationRow>> {
        let tenant = tenant_id.to_string();
        let kind = kind.to_string();
        let account = account_id.to_string();
        let pool = self.pool.clone();
        stheno_db::tenant::scope(tenant, async move {
            stheno_db::tenant::with_tenant_conn(&pool, move |conn| {
                Box::pin(async move {
                    diesel::sql_query(
                        "SELECT id, kind, account_id, webhook_id, ciphertext, nonce, has_credentials, version, enabled, updated_at \
                           FROM ai.configuration WHERE kind = $1 AND account_id = $2 LIMIT 1",
                    )
                    .bind::<Text, _>(kind)
                    .bind::<Text, _>(account)
                    .get_result::<ConfigurationRow>(conn)
                    .await
                    .optional()
                    .map_err(|error| Error::Database(error.to_string()))
                })
            })
            .await
        })
        .await
    }
}

fn status(row: ConfigurationRow) -> AiConfigurationStatus {
    AiConfigurationStatus {
        id: row.id,
        kind: row.kind,
        account_id: row.account_id,
        configured: row.has_credentials && row.nonce.len() == 12,
        enabled: row.enabled,
        version: row.version,
        webhook_id: row.webhook_id,
        updated_at: row.updated_at,
    }
}

fn normalize_account(kind: &str, account_id: &str) -> Result<String> {
    if kind == "llm" {
        if account_id != "default" {
            return Err(invalid(
                "LLM configuration must use the default account key.",
            ));
        }
        return Ok("default".to_string());
    }
    if kind == "search_console" {
        return normalize_search_console_property(account_id);
    }
    if !matches!(
        kind,
        "whatsapp"
            | "messenger"
            | "instagram"
            | "meta_insights_facebook"
            | "meta_insights_instagram"
    ) || account_id.trim().is_empty()
        || account_id.len() > 128
        || account_id.chars().any(char::is_control)
    {
        return Err(invalid("Channel and account ID are invalid."));
    }
    Ok(account_id.trim().to_string())
}

fn meta_insights_source(channel: &str) -> Result<(&'static str, &'static str)> {
    match channel.to_ascii_lowercase().as_str() {
        "messenger" => Ok(("meta_insights_facebook", "facebook_page")),
        "instagram" => Ok(("meta_insights_instagram", "instagram_account")),
        _ => Err(invalid("Meta insights channel is unsupported.")),
    }
}

/// Validates and normalizes a Search Console property identifier or URL.
pub fn normalize_search_console_property(property: &str) -> Result<String> {
    let property = property.trim();
    let valid_domain = |domain: &str| {
        !domain.is_empty()
            && domain.len() <= 253
            && domain.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                    && label
                        .as_bytes()
                        .first()
                        .is_some_and(u8::is_ascii_alphanumeric)
                    && label
                        .as_bytes()
                        .last()
                        .is_some_and(u8::is_ascii_alphanumeric)
            })
    };
    let valid = if let Some(domain) = property.strip_prefix("sc-domain:") {
        valid_domain(domain)
    } else if property.len() <= 512 && !property.chars().any(char::is_control) {
        url::Url::parse(property).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
        })
    } else {
        false
    };
    if !valid {
        return Err(invalid("Search Console property is invalid."));
    }
    Ok(property.to_string())
}

fn allowed_secret_keys(kind: &str) -> Result<&'static [&'static str]> {
    match kind {
        "llm" => Ok(&["provider", "model", "api_key"]),
        "search_console" => Ok(&["client_id", "client_secret", "refresh_token"]),
        "whatsapp" | "messenger" | "instagram" => {
            Ok(&["app_secret", "verify_token", "access_token"])
        }
        "meta_insights_facebook" | "meta_insights_instagram" => Ok(&["app_secret", "access_token"]),
        _ => Err(invalid("AI configuration kind is unsupported.")),
    }
}

fn validate_required_values(
    kind: &str,
    values: &BTreeMap<String, String>,
    enabled: bool,
) -> Result<()> {
    if !enabled {
        return Ok(());
    }
    let required: &[&str] = match kind {
        "llm" => &["provider", "model", "api_key"],
        "search_console" => &["client_id", "client_secret", "refresh_token"],
        "meta_insights_facebook" | "meta_insights_instagram" => &["app_secret", "access_token"],
        _ => &["app_secret", "verify_token", "access_token"],
    };
    if required.iter().any(|key| !values.contains_key(*key)) {
        return Err(invalid(
            "Enabled AI configuration is missing required secrets.",
        ));
    }
    if kind == "llm" {
        if !matches!(
            values.get("provider").map(String::as_str),
            Some("deepseek" | "qwen" | "ollama")
        ) {
            return Err(invalid(
                "AI provider must be deepseek, qwen, or the local Ollama service.",
            ));
        }
        let model = required_value(values, "model")?;
        if model.trim().is_empty() || model.len() > 128 {
            return Err(invalid("AI model name is invalid."));
        }
    }
    Ok(())
}

fn required_value<'a>(values: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    values
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid("Encrypted AI configuration is incomplete."))
}

fn key_from_env() -> Result<Aes256Gcm> {
    let raw = std::env::var("STHENO_CREDENTIAL_KEY").map_err(|_| Error::Invalid {
        code: "credential_key_missing",
        message: "STHENO_CREDENTIAL_KEY is required".to_string(),
        field: None,
    })?;
    let key = if raw.len() == 64 && raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        hex::decode(raw).map_err(|_| invalid("Credential encryption key is invalid."))?
    } else {
        raw.into_bytes()
    };
    if key.len() != 32 {
        return Err(invalid("STHENO_CREDENTIAL_KEY must contain 32 bytes."));
    }
    Aes256Gcm::new_from_slice(&key).map_err(|_| invalid("Credential encryption key is invalid."))
}

fn associated_data(tenant_id: &str, kind: &str, account_id: &str) -> Vec<u8> {
    format!("{tenant_id}\0{kind}\0{account_id}").into_bytes()
}

fn encrypt_values(
    tenant_id: &str,
    kind: &str,
    account_id: &str,
    values: &BTreeMap<String, String>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let plaintext = serde_json::to_vec(values)
        .map_err(|_| invalid("AI configuration could not be serialized."))?;
    encrypt_payload(&associated_data(tenant_id, kind, account_id), &plaintext)
}

pub(crate) fn encrypt_payload(aad: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = key_from_env()?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| invalid("AI configuration encryption failed."))?;
    Ok((ciphertext, nonce.to_vec()))
}

fn decrypt_values(
    tenant_id: &str,
    kind: &str,
    account_id: &str,
    row: &ConfigurationRow,
) -> Result<BTreeMap<String, String>> {
    if row.nonce.len() != 12 {
        return Err(invalid("Encrypted AI configuration nonce is invalid."));
    }
    let plaintext = decrypt_payload(
        &associated_data(tenant_id, kind, account_id),
        &row.ciphertext,
        &row.nonce,
    )?;
    serde_json::from_slice(&plaintext).map_err(|_| invalid("AI configuration payload is invalid."))
}

pub(crate) fn decrypt_payload(aad: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    if nonce.len() != 12 {
        return Err(invalid("Encrypted AI configuration nonce is invalid."));
    }
    key_from_env()?
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| invalid("Encrypted AI payload could not be decrypted."))
}

fn invalid(message: &'static str) -> Error {
    Error::Invalid {
        code: "ai_configuration_invalid",
        message: message.to_string(),
        field: None,
    }
}

#[cfg(test)]
mod search_console_configuration_tests {
    use super::{allowed_secret_keys, normalize_account, normalize_search_console_property};

    #[test]
    fn only_search_console_properties_and_read_credentials_are_allowed() {
        assert_eq!(
            normalize_search_console_property("sc-domain:shop.example.com").unwrap(),
            "sc-domain:shop.example.com"
        );
        assert_eq!(
            normalize_search_console_property("https://shop.example.com/").unwrap(),
            "https://shop.example.com/"
        );
        assert!(normalize_search_console_property("javascript:alert(1)").is_err());
        assert!(normalize_search_console_property("https://user@shop.example.com/").is_err());
        assert!(normalize_search_console_property("sc-domain:-bad.example").is_err());
        assert_eq!(
            normalize_account("search_console", "https://shop.example.com/").unwrap(),
            "https://shop.example.com/"
        );
        assert_eq!(
            allowed_secret_keys("search_console").unwrap(),
            &["client_id", "client_secret", "refresh_token"]
        );
    }
}
