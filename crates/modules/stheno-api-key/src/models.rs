//! Internal entity models for stheno-api-key.
//!
//! Rule 2.9: Entities are strictly pub(crate).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ApiKey {
    pub id: Id<ApiKey>,
    pub key_type: String, // "publishable", "secret"
    pub title: String,
    pub redacted_key: String,           // e.g. "pk_...3a8f"
    pub token_hash: String,             // SHA-256 hash of plain token
    pub sales_channel_ids: Vec<String>, // Scopes (F20)
    pub permissions: Vec<String>,
    pub created_by: Option<String>,
    pub revoked_by: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for ApiKey {
    const TABLE: &'static str = "api_key.api_key";
    const ENTITY: &'static str = "api_key";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    ApiKey => "apk",
}
