use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndexData {
    pub id: Id<IndexData>,
    pub entity_name: String,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndexRelation {
    pub id: Id<IndexRelation>,
    pub parent_entity: String,
    pub parent_id: String,
    pub child_entity: String,
    pub child_id: String,
    pub relation_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndexField {
    pub id: Id<IndexField>,
    pub entity_name: String,
    pub field_name: String,
    pub field_type: String,
    pub is_searchable: bool,
    pub is_filterable: bool,
    pub is_sortable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndexSyncState {
    pub id: Id<IndexSyncState>,
    pub entity_name: String,
    pub last_synced_version: i64,
    pub last_synced_at: DateTime<Utc>,
    pub lag_seconds: i64,
    pub status: String,
}
