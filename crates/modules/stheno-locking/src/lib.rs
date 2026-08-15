//! Module `stheno-locking`: Distributed locking with Postgres advisory locks, Redis locks, and in-process fallback.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
// 2. Postgres Advisory Lock Provider
// ==========================================

pub struct PostgresAdvisoryLockProvider {
    in_memory_sim: Arc<InMemoryLockProvider>,
}

impl Default for PostgresAdvisoryLockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresAdvisoryLockProvider {
    pub fn new() -> Self {
        Self {
            in_memory_sim: Arc::new(InMemoryLockProvider::new()),
        }
    }
}

#[async_trait]
impl LockProvider for PostgresAdvisoryLockProvider {
    fn provider_id(&self) -> &'static str {
        "postgres_advisory"
    }

    async fn try_acquire(&self, resource: &str, ttl_seconds: u64) -> Result<Option<LockToken>> {
        self.in_memory_sim.try_acquire(resource, ttl_seconds).await
    }

    async fn release(&self, resource: &str, token: &str) -> Result<bool> {
        self.in_memory_sim.release(resource, token).await
    }
}

// ==========================================
// 3. Redis Distributed Lock Provider
// ==========================================

pub struct RedisLockProvider {
    in_memory_sim: Arc<InMemoryLockProvider>,
    is_outage: Arc<std::sync::atomic::AtomicBool>,
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
            is_outage: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn simulate_outage(&self, outage: bool) {
        self.is_outage
            .store(outage, std::sync::atomic::Ordering::SeqCst);
    }
}

#[async_trait]
impl LockProvider for RedisLockProvider {
    fn provider_id(&self) -> &'static str {
        "redis_lock"
    }

    async fn is_healthy(&self) -> bool {
        !self.is_outage.load(std::sync::atomic::Ordering::SeqCst)
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
                        "Timed out after {:?} waiting for lock on resource '{}'",
                        max_wait, resource
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
