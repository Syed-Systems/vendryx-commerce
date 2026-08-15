//! Internal entity models for stheno-api-key.
//!
//! Rule 2.9: Entities are strictly pub(crate) and never derive deserialization.

use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
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
