use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSettingDTO {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfigDTO {
    pub id: String,
    pub user_id: Option<String>,
    pub scope: String,
    pub columns: Vec<String>,
    pub filters: Option<serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page_size: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyLabelDTO {
    pub id: String,
    pub entity_type: String,
    pub property_key: String,
    pub label: String,
    pub help_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPreferenceDTO {
    pub id: String,
    pub user_id: String,
    pub layout_key: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetStoreSettingInput {
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub is_secret: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveViewConfigInput {
    pub user_id: Option<String>,
    pub scope: String,
    pub columns: Vec<String>,
    pub filters: Option<serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetPropertyLabelInput {
    pub entity_type: String,
    pub property_key: String,
    pub label: String,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveLayoutPreferenceInput {
    pub user_id: String,
    pub layout_key: String,
    pub config: serde_json::Value,
}
