use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl CrudEntity for IndexData {
    const TABLE: &'static str = "index.index_data";
    const ENTITY: &'static str = "index_data";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IndexRelation {
    pub id: Id<IndexRelation>,
    pub parent_entity: String,
    pub parent_id: String,
    pub child_entity: String,
    pub child_id: String,
    pub relation_type: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for IndexRelation {
    const TABLE: &'static str = "index.index_relation";
    const ENTITY: &'static str = "index_relation";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IndexField {
    pub id: Id<IndexField>,
    pub entity_name: String,
    pub field_name: String,
    pub field_type: String,
    pub is_searchable: bool,
    pub is_filterable: bool,
    pub is_sortable: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for IndexField {
    const TABLE: &'static str = "index.index_field";
    const ENTITY: &'static str = "index_field";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IndexSyncState {
    pub id: Id<IndexSyncState>,
    pub entity_name: String,
    pub last_synced_version: i64,
    pub last_synced_at: DateTime<Utc>,
    pub lag_seconds: i64,
    pub status: String,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for IndexSyncState {
    const TABLE: &'static str = "index.index_sync_state";
    const ENTITY: &'static str = "index_sync_state";

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
    IndexData => "idxdata",
    IndexField => "idxfld",
    IndexRelation => "idxrel",
    IndexSyncState => "idxsync",
}
