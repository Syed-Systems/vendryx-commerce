use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StoreSetting {
    pub id: Id<StoreSetting>,
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ViewConfig {
    pub id: Id<ViewConfig>,
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PropertyLabel {
    pub id: Id<PropertyLabel>,
    pub entity_type: String,
    pub property_key: String,
    pub label: String,
    pub help_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LayoutPreference {
    pub id: Id<LayoutPreference>,
    pub user_id: String,
    pub layout_key: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
