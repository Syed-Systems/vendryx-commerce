use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoreSetting {
    pub id: Id<StoreSetting>,
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for StoreSetting {
    const TABLE: &'static str = "settings.store_setting";
    const ENTITY: &'static str = "store_setting";

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
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for ViewConfig {
    const TABLE: &'static str = "settings.view_config";
    const ENTITY: &'static str = "view_config";

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
pub(crate) struct PropertyLabel {
    pub id: Id<PropertyLabel>,
    pub entity_type: String,
    pub property_key: String,
    pub label: String,
    pub help_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for PropertyLabel {
    const TABLE: &'static str = "settings.property_label";
    const ENTITY: &'static str = "property_label";

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
pub(crate) struct LayoutPreference {
    pub id: Id<LayoutPreference>,
    pub user_id: String,
    pub layout_key: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for LayoutPreference {
    const TABLE: &'static str = "settings.layout_preference";
    const ENTITY: &'static str = "layout_preference";

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
    LayoutPreference => "layoutpref",
    PropertyLabel => "proplbl",
    StoreSetting => "sset",
    ViewConfig => "viewcfg",
}
