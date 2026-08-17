use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CacheEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for CacheEntry {
    const TABLE: &'static str = "caching.cache_entry";
    const ENTITY: &'static str = "cache_entry";
    const ID_COLUMN: &'static str = "key";

    fn id(&self) -> String {
        self.key.clone()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}
