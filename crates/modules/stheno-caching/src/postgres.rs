//! Postgres fallback cache provider for stheno-caching (P14-15).

use async_trait::async_trait;
use chrono::{Duration, Utc};
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::Result;

use crate::models::CacheEntry;
use crate::provider::CacheProvider;

#[derive(Clone)]
pub struct PostgresCacheProvider {
    entries: PgCrudRepository<CacheEntry>,
}

impl PostgresCacheProvider {
    pub fn new(pool: DbPool) -> Self {
        Self {
            entries: PgCrudRepository::new(pool),
        }
    }
}

#[async_trait]
impl CacheProvider for PostgresCacheProvider {
    fn provider_id(&self) -> &'static str {
        "postgres_cache"
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let filter = FilterSpec::new()
            .with_predicate("key", FilterOp::Eq(key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.entries.list(None, &filter).await?;
        if let Some(entry) = list.into_iter().next() {
            if entry.expires_at > Utc::now() {
                let bytes = serde_json::to_vec(&entry.value).unwrap_or_default();
                return Ok(Some(bytes));
            }
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &[u8], ttl_seconds: Option<u64>) -> Result<()> {
        let val: serde_json::Value = serde_json::from_slice(value)
            .unwrap_or_else(|_| serde_json::Value::String(hex::encode(value)));
        let ttl = ttl_seconds.unwrap_or(86400) as i64;
        let now = Utc::now();
        let expires_at = now + Duration::seconds(ttl);

        let filter = FilterSpec::new()
            .with_predicate("key", FilterOp::Eq(key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.entries.list(None, &filter).await?;
        if let Some(mut entry) = existing.into_iter().next() {
            entry.value = val;
            entry.expires_at = expires_at;
            let key_str = entry.key.clone();
            let _ = self.entries.update(None, &key_str, entry).await;
        } else {
            let entry = CacheEntry {
                key: key.to_string(),
                value: val,
                expires_at,
                created_at: now,
                deleted_at: None,
            };
            let _ = self.entries.create(None, entry).await;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let _ = self.entries.soft_delete(None, key).await;
        Ok(())
    }

    async fn delete_by_prefix(&self, prefix: &str) -> Result<()> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.entries.list(None, &filter).await?;
        for entry in list {
            if entry.key.starts_with(prefix) {
                let _ = self.entries.soft_delete(None, &entry.key).await;
            }
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.entries.list(None, &filter).await?;
        for entry in list {
            let _ = self.entries.soft_delete(None, &entry.key).await;
        }
        Ok(())
    }
}
