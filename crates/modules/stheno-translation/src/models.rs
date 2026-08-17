use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Locale {
    pub id: Id<Locale>,
    pub code: String,
    pub name: String,
    pub is_default: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Locale {
    const TABLE: &'static str = "translation.locale";
    const ENTITY: &'static str = "locale";

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
pub(crate) struct Translation {
    pub id: Id<Translation>,
    pub locale_code: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Translation {
    const TABLE: &'static str = "translation.translation";
    const ENTITY: &'static str = "translation";

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
pub(crate) struct TranslationOverride {
    pub id: Id<TranslationOverride>,
    pub translation_id: String,
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub override_value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for TranslationOverride {
    const TABLE: &'static str = "translation.translation_override";
    const ENTITY: &'static str = "translation_override";

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
    Locale => "loc",
    Translation => "trans",
    TranslationOverride => "transov",
}
