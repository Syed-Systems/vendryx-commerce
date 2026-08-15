use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleDTO {
    pub id: String,
    pub code: String,
    pub name: String,
    pub is_default: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDTO {
    pub id: String,
    pub locale_code: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationOverrideDTO {
    pub id: String,
    pub translation_id: String,
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub override_value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLocaleInput {
    pub code: String,
    pub name: String,
    pub is_default: Option<bool>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetTranslationInput {
    pub locale_code: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetTranslationOverrideInput {
    pub translation_id: String,
    pub sales_channel_id: Option<String>,
    pub region_id: Option<String>,
    pub override_value: String,
}
