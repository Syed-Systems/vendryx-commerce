use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Locale {
    pub id: Id<Locale>,
    pub code: String,
    pub name: String,
    pub is_default: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Translation {
    pub id: Id<Translation>,
    pub locale_code: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TranslationOverride {
    pub id: Id<TranslationOverride>,
    pub translation_id: String,
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub override_value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
