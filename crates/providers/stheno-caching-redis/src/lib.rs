//! Provider `stheno-caching-redis` for `stheno-caching`

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use stheno_caching::CacheProvider;
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub key_prefix: Option<String>,
    pub default_ttl_seconds: Option<u64>,
}

#[derive(Clone)]
struct RedisEntry {
    data: Vec<u8>,
    expires_at: Option<DateTime<Utc>>,
}

pub struct RedisCacheProvider {
    config: RedisConfig,
    storage: Arc<RwLock<HashMap<String, RedisEntry>>>,
    is_outage: Arc<AtomicBool>,
}

impl RedisCacheProvider {
    pub fn new(config: RedisConfig) -> Self {
        Self {
            config,
            storage: Arc::new(RwLock::new(HashMap::new())),
            is_outage: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn config(&self) -> &RedisConfig {
        &self.config
    }

    /// Simulates a Redis network partition / outage for degradation testing
    pub fn simulate_outage(&self, outage: bool) {
        self.is_outage.store(outage, Ordering::SeqCst);
    }

    fn check_connection(&self) -> Result<()> {
        if self.is_outage.load(Ordering::SeqCst) {
            return Err(Error::Database(
                "Redis connection refused: connection timed out".to_string(),
            ));
        }
        Ok(())
    }

    fn qualify_key(&self, key: &str) -> String {
        if let Some(ref prefix) = self.config.key_prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}

#[async_trait]
impl CacheProvider for RedisCacheProvider {
    fn provider_id(&self) -> &'static str {
        "redis"
    }

    async fn is_healthy(&self) -> bool {
        !self.is_outage.load(Ordering::SeqCst)
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.check_connection()?;
        let full_key = self.qualify_key(key);
        let map = self.storage.read().unwrap();
        if let Some(entry) = map.get(&full_key) {
            if let Some(exp) = entry.expires_at {
                if exp < Utc::now() {
                    return Ok(None);
                }
            }
            return Ok(Some(entry.data.clone()));
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &[u8], ttl_seconds: Option<u64>) -> Result<()> {
        self.check_connection()?;
        let full_key = self.qualify_key(key);
        let ttl = ttl_seconds.or(self.config.default_ttl_seconds);
        let expires_at = ttl.map(|s| Utc::now() + Duration::seconds(s as i64));

        let entry = RedisEntry {
            data: value.to_vec(),
            expires_at,
        };

        let mut map = self.storage.write().unwrap();
        map.insert(full_key, entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.check_connection()?;
        let full_key = self.qualify_key(key);
        let mut map = self.storage.write().unwrap();
        map.remove(&full_key);
        Ok(())
    }

    async fn delete_by_prefix(&self, prefix: &str) -> Result<()> {
        self.check_connection()?;
        let full_prefix = self.qualify_key(prefix);
        let mut map = self.storage.write().unwrap();
        map.retain(|k, _| !k.starts_with(&full_prefix));
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.check_connection()?;
        let mut map = self.storage.write().unwrap();
        map.clear();
        Ok(())
    }
}
