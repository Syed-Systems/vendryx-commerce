//! Module `stheno-locking`: Distributed locking with Postgres advisory locks on pinned connections (P14-16, P14-17).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::sql_types::{Bool, Text};
use diesel::QueryableByName;
use serde::{Deserialize, Serialize};
use stheno_db::{DbConnection, DbPool};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockToken {
    pub resource: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait LockProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn try_acquire(&self, resource: &str, ttl_seconds: u64) -> Result<Option<LockToken>>;
    async fn release(&self, resource: &str, token: &str) -> Result<bool>;
    async fn is_healthy(&self) -> bool {
        true
    }
}

/// Namespaces a lock key by the calling tenant.
///
/// Postgres advisory locks are a single global 64-bit key space shared by every session
/// on the database, and row-level security does not apply to them. This is the only
/// tenant boundary in the engine enforced entirely in application code, so it fails
/// loudly when no tenant is in scope rather than falling back to a bare key.
fn tenant_key(resource: &str) -> Result<String> {
    let tenant = stheno_db::tenant::current()?;
    Ok(format!("{tenant}:{resource}"))
}

#[derive(QueryableByName)]
struct BoolRow {
    #[diesel(sql_type = Bool)]
    value: bool,
}

// ==========================================
// 1. In-Memory Lock Provider
// ==========================================

#[derive(Clone)]
struct LockEntry {
    token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct InMemoryLockProvider {
    locks: Arc<RwLock<HashMap<String, LockEntry>>>,
}

impl InMemoryLockProvider {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LockProvider for InMemoryLockProvider {
    fn provider_id(&self) -> &'static str {
        "in_memory"
    }

    async fn try_acquire(&self, resource: &str, ttl_seconds: u64) -> Result<Option<LockToken>> {
        let now = Utc::now();
        let mut map = self.locks.write().unwrap();

        if let Some(entry) = map.get(resource) {
            if entry.expires_at > now {
                return Ok(None); // Locked by someone else
            }
        }

        let token = ulid::Ulid::new().to_string();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

        map.insert(
            resource.to_string(),
            LockEntry {
                token: token.clone(),
                expires_at,
            },
        );

        Ok(Some(LockToken {
            resource: resource.to_string(),
            token,
            expires_at,
        }))
    }

    async fn release(&self, resource: &str, token: &str) -> Result<bool> {
        let mut map = self.locks.write().unwrap();
        if let Some(entry) = map.get(resource) {
            if entry.token == token {
                map.remove(resource);
                return Ok(true);
            }
        }
        Ok(false)
    }
}

// ==========================================
// 2. Postgres Advisory Lock Provider (P14-16, P14-17)
// ==========================================

pub struct PostgresAdvisoryLockProvider {
    pool: DbPool,
    pinned_connections: Arc<Mutex<HashMap<String, DbConnection>>>,
}

impl PostgresAdvisoryLockProvider {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            pinned_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LockProvider for PostgresAdvisoryLockProvider {
    fn provider_id(&self) -> &'static str {
        "postgres_advisory"
    }

    async fn try_acquire(&self, resource: &str, ttl_seconds: u64) -> Result<Option<LockToken>> {
        use diesel_async::RunQueryDsl;

        let mut conn = self.pool.get().await.map_err(|e| {
            Error::Database(format!(
                "Failed to acquire connection for advisory lock: {e}"
            ))
        })?;

        // Try acquire Postgres session-level advisory lock using 64-bit hashtext.
        //
        // The key is namespaced by tenant (ADR-012). Advisory locks live in a GLOBAL
        // 64-bit key space that row-level security does not touch — it is the one place
        // in the engine with no database backstop. Without the namespace, tenant A
        // locking "cart_01H…" would block tenant B's identically-named cart, which is a
        // cross-tenant denial of service that would look like an intermittent hang.
        let key = tenant_key(resource)?;
        let rows: Vec<BoolRow> =
            diesel::sql_query("SELECT pg_try_advisory_lock(hashtext($1)) as value")
                .bind::<Text, _>(&key)
                .load(&mut conn)
                .await
                .map_err(|e| Error::Database(format!("Failed pg_try_advisory_lock: {e}")))?;

        let acquired = rows.as_slice().first().map(|r| r.value).unwrap_or(false);

        if acquired {
            let token = ulid::Ulid::new().to_string();
            let expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);

            {
                let mut map = self.pinned_connections.lock().unwrap();
                map.insert(token.clone(), conn);
            }

            Ok(Some(LockToken {
                resource: resource.to_string(),
                token,
                expires_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn release(&self, resource: &str, token: &str) -> Result<bool> {
        use diesel_async::RunQueryDsl;

        let mut conn = {
            let mut map = self.pinned_connections.lock().unwrap();
            map.remove(token)
        };

        if let Some(ref mut c) = conn {
            // Must namespace identically to `try_acquire`, or the unlock targets a
            // different key and the lock is held until the session ends.
            let key = tenant_key(resource)?;
            let rows: Vec<BoolRow> =
                diesel::sql_query("SELECT pg_advisory_unlock(hashtext($1)) as value")
                    .bind::<Text, _>(&key)
                    .load(&mut **c)
                    .await
                    .map_err(|e| Error::Database(format!("Failed pg_advisory_unlock: {e}")))?;

            let released = rows.as_slice().first().map(|r| r.value).unwrap_or(false);
            Ok(released)
        } else {
            Ok(false)
        }
    }
}

// ==========================================
// 3. Redis Distributed Lock Provider
// ==========================================

pub struct RedisLockProvider {
    in_memory_sim: Arc<InMemoryLockProvider>,
    is_outage: Arc<AtomicBool>,
}

impl Default for RedisLockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RedisLockProvider {
    pub fn new() -> Self {
        Self {
            in_memory_sim: Arc::new(InMemoryLockProvider::new()),
            is_outage: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn simulate_outage(&self, outage: bool) {
        self.is_outage.store(outage, Ordering::SeqCst);
    }
}

#[async_trait]
impl LockProvider for RedisLockProvider {
    fn provider_id(&self) -> &'static str {
        "redis_lock"
    }

    async fn is_healthy(&self) -> bool {
        !self.is_outage.load(Ordering::SeqCst)
    }

    async fn try_acquire(&self, resource: &str, ttl_seconds: u64) -> Result<Option<LockToken>> {
        if !self.is_healthy().await {
            return Err(Error::Database("Redis connection refused".to_string()));
        }
        self.in_memory_sim.try_acquire(resource, ttl_seconds).await
    }

    async fn release(&self, resource: &str, token: &str) -> Result<bool> {
        if !self.is_healthy().await {
            return Err(Error::Database("Redis connection refused".to_string()));
        }
        self.in_memory_sim.release(resource, token).await
    }
}

// ==========================================
// 4. Lock Manager with Graceful Fallback
// ==========================================

pub struct LockManager {
    primary: Arc<dyn LockProvider>,
    fallback: Arc<dyn LockProvider>,
}

impl LockManager {
    pub fn new(primary: Arc<dyn LockProvider>, fallback: Arc<dyn LockProvider>) -> Self {
        Self { primary, fallback }
    }

    pub async fn acquire(
        &self,
        resource: &str,
        ttl_seconds: u64,
        max_wait: Duration,
    ) -> Result<LockToken> {
        let start = std::time::Instant::now();
        let retry_interval = Duration::from_millis(50);

        loop {
            // Attempt with primary if healthy
            if self.primary.is_healthy().await {
                match self.primary.try_acquire(resource, ttl_seconds).await {
                    Ok(Some(token)) => return Ok(token),
                    Ok(None) => {} // Locked, will retry
                    Err(e) => {
                        tracing::warn!("Primary lock provider failed ({}), trying fallback", e);
                        if let Ok(Some(token)) =
                            self.fallback.try_acquire(resource, ttl_seconds).await
                        {
                            return Ok(token);
                        }
                    }
                }
            } else if let Ok(Some(token)) = self.fallback.try_acquire(resource, ttl_seconds).await {
                return Ok(token);
            }

            if start.elapsed() >= max_wait {
                return Err(Error::Conflict {
                    code: "lock_acquisition_timeout",
                    message: format!(
                        "Timed out after {max_wait:?} waiting for lock on resource '{resource}'"
                    ),
                });
            }

            tokio::time::sleep(retry_interval).await;
        }
    }

    pub async fn release(&self, lock: &LockToken) -> Result<bool> {
        let mut released = false;
        if self.primary.is_healthy().await {
            if let Ok(res) = self.primary.release(&lock.resource, &lock.token).await {
                released = released || res;
            }
        }
        if let Ok(res) = self.fallback.release(&lock.resource, &lock.token).await {
            released = released || res;
        }
        Ok(released)
    }
}
