//! Data Transfer Objects for stheno-api-key.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyDTO {
    pub id: String,
    pub key_type: String,
    pub title: String,
    pub redacted_key: String,
    pub sales_channel_ids: Vec<String>,
    pub permissions: Vec<String>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyInput {
    pub key_type: String, // "publishable" or "secret"
    pub title: String,
    pub sales_channel_ids: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub created_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedApiKeyResponse {
    pub api_key: ApiKeyDTO,
    pub token: String, // Plain token returned ONLY ONCE upon creation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyInput {
    pub title: Option<String>,
    pub sales_channel_ids: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateKeyResult {
    pub is_valid: bool,
    pub key: Option<ApiKeyDTO>,
    pub sales_channel_ids: Vec<String>,
    pub permissions: Vec<String>,
}
