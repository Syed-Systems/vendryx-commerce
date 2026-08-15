use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use stheno_types::Result;

use crate::provider::CacheProvider;

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Default)]
pub struct InMemoryCacheProvider {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl InMemoryCacheProvider {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn purge_expired(&self) {
        let now = Utc::now();
        let mut map = self.entries.write().unwrap();
        map.retain(|_, entry| {
            if let Some(exp) = entry.expires_at {
                exp > now
            } else {
                true
            }
        });
    }
}

#[async_trait]
impl CacheProvider for InMemoryCacheProvider {
    fn provider_id(&self) -> &'static str {
        "in_process"
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.purge_expired();
        let map = self.entries.read().unwrap();
        if let Some(entry) = map.get(key) {
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
        let expires_at = ttl_seconds.map(|s| Utc::now() + Duration::seconds(s as i64));
        let entry = CacheEntry {
            data: value.to_vec(),
            expires_at,
        };
        let mut map = self.entries.write().unwrap();
        map.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut map = self.entries.write().unwrap();
        map.remove(key);
        Ok(())
    }

    async fn delete_by_prefix(&self, prefix: &str) -> Result<()> {
        let mut map = self.entries.write().unwrap();
        map.retain(|k, _| !k.starts_with(prefix));
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut map = self.entries.write().unwrap();
        map.clear();
        Ok(())
    }
}
