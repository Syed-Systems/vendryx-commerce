use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDTO {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadFileInput {
    pub filename: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePresignedUploadInput {
    pub filename: String,
    pub content_type: String,
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUploadDTO {
    pub file_id: String,
    pub upload_url: String,
    pub file_url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub expires_at: DateTime<Utc>,
}
