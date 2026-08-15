use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDataDTO {
    pub id: String,
    pub entity_name: String,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRelationDTO {
    pub id: String,
    pub parent_entity: String,
    pub parent_id: String,
    pub child_entity: String,
    pub child_id: String,
    pub relation_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexFieldDTO {
    pub id: String,
    pub entity_name: String,
    pub field_name: String,
    pub field_type: String,
    pub is_searchable: bool,
    pub is_filterable: bool,
    pub is_sortable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSyncStateDTO {
    pub id: String,
    pub entity_name: String,
    pub last_synced_version: i64,
    pub last_synced_at: DateTime<Utc>,
    pub lag_seconds: i64,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertIndexDataInput {
    pub entity_name: String,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub version: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryIndexFilterInput {
    pub entity_name: String,
    pub filters: std::collections::HashMap<String, serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc", "desc"
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
