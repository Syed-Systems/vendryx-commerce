use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateLocaleInput, LocaleDTO, SetTranslationInput, SetTranslationOverrideInput, TranslationDTO,
    TranslationOverrideDTO,
};
use crate::models::{Locale, Translation, TranslationOverride};

pub struct TranslationService {
    locales: RwLock<HashMap<String, Locale>>,
    translations: RwLock<HashMap<String, Translation>>,
    overrides: RwLock<HashMap<String, TranslationOverride>>,
}

impl Default for TranslationService {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationService {
    pub fn new() -> Self {
        Self {
            locales: RwLock::new(HashMap::new()),
            translations: RwLock::new(HashMap::new()),
            overrides: RwLock::new(HashMap::new()),
        }
    }

    fn make_translation_key(
        locale_code: &str,
        entity_type: &str,
        entity_id: &str,
        field_name: &str,
    ) -> String {
        format!(
            "{}:{}:{}:{}",
            locale_code, entity_type, entity_id, field_name
        )
    }

    // ==========================================
    // 1. Locales
    // ==========================================

    pub async fn create_locale(&self, input: CreateLocaleInput) -> Result<LocaleDTO> {
        let id = Id::new();
        let now = Utc::now();

        let locale = Locale {
            id,
            code: input.code.clone(),
            name: input.name,
            is_default: input.is_default.unwrap_or(false),
            is_enabled: input.is_enabled.unwrap_or(true),
            created_at: now,
            updated_at: now,
        };

        let dto = LocaleDTO {
            id: locale.id.to_string(),
            code: locale.code.clone(),
            name: locale.name.clone(),
            is_default: locale.is_default,
            is_enabled: locale.is_enabled,
            created_at: locale.created_at,
            updated_at: locale.updated_at,
        };

        {
            let mut map = self.locales.write().unwrap();
            map.insert(locale.code.clone(), locale);
        }

        Ok(dto)
    }

    pub async fn get_locale(&self, code: &str) -> Result<LocaleDTO> {
        let map = self.locales.read().unwrap();
        let l = map.get(code).ok_or_else(|| Error::NotFound {
            entity: "locale",
            id: code.to_string(),
        })?;

        Ok(LocaleDTO {
            id: l.id.to_string(),
            code: l.code.clone(),
            name: l.name.clone(),
            is_default: l.is_default,
            is_enabled: l.is_enabled,
            created_at: l.created_at,
            updated_at: l.updated_at,
        })
    }

    pub async fn list_locales(&self) -> Result<Vec<LocaleDTO>> {
        let map = self.locales.read().unwrap();
        let list: Vec<LocaleDTO> = map
            .values()
            .map(|l| LocaleDTO {
                id: l.id.to_string(),
                code: l.code.clone(),
                name: l.name.clone(),
                is_default: l.is_default,
                is_enabled: l.is_enabled,
                created_at: l.created_at,
                updated_at: l.updated_at,
            })
            .collect();
        Ok(list)
    }

    // ==========================================
    // 2. Translations & Scoped Overrides
    // ==========================================

    pub async fn set_translation(&self, input: SetTranslationInput) -> Result<TranslationDTO> {
        let key = Self::make_translation_key(
            &input.locale_code,
            &input.entity_type,
            &input.entity_id,
            &input.field_name,
        );

        let id = Id::new();
        let now = Utc::now();

        let translation = Translation {
            id,
            locale_code: input.locale_code,
            entity_type: input.entity_type,
            entity_id: input.entity_id,
            field_name: input.field_name,
            value: input.value,
            created_at: now,
            updated_at: now,
        };

        let dto = TranslationDTO {
            id: translation.id.to_string(),
            locale_code: translation.locale_code.clone(),
            entity_type: translation.entity_type.clone(),
            entity_id: translation.entity_id.clone(),
            field_name: translation.field_name.clone(),
            value: translation.value.clone(),
            created_at: translation.created_at,
            updated_at: translation.updated_at,
        };

        {
            let mut map = self.translations.write().unwrap();
            map.insert(key, translation);
        }

        Ok(dto)
    }

    pub async fn set_translation_override(
        &self,
        input: SetTranslationOverrideInput,
    ) -> Result<TranslationOverrideDTO> {
        let id = Id::new();
        let now = Utc::now();

        let tr_override = TranslationOverride {
            id,
            translation_id: input.translation_id.clone(),
            sales_channel_id: input.sales_channel_id,
            region_id: input.region_id,
            override_value: input.override_value,
            created_at: now,
            updated_at: now,
        };

        let dto = TranslationOverrideDTO {
            id: tr_override.id.to_string(),
            translation_id: tr_override.translation_id.clone(),
            sales_channel_id: tr_override.sales_channel_id.clone(),
            region_id: tr_override.region_id.clone(),
            override_value: tr_override.override_value.clone(),
            created_at: tr_override.created_at,
            updated_at: tr_override.updated_at,
        };

        {
            let mut map = self.overrides.write().unwrap();
            map.insert(id.to_string(), tr_override);
        }

        Ok(dto)
    }

    pub async fn resolve_scoped_field(
        &self,
        locale_code: &str,
        entity_type: &str,
        entity_id: &str,
        field_name: &str,
        sales_channel_id: Option<&str>,
        region_id: Option<&str>,
        fallback: &str,
    ) -> String {
        let key = Self::make_translation_key(locale_code, entity_type, entity_id, field_name);
        let tr_map = self.translations.read().unwrap();
        if let Some(tr) = tr_map.get(&key) {
            let ov_map = self.overrides.read().unwrap();
            // Look for matching scope override
            for ov in ov_map.values() {
                if ov.translation_id == tr.id.to_string() {
                    if let (Some(sc), Some(ov_sc)) = (sales_channel_id, &ov.sales_channel_id) {
                        if sc == ov_sc {
                            return ov.override_value.clone();
                        }
                    }
                    if let (Some(reg), Some(ov_reg)) = (region_id, &ov.region_id) {
                        if reg == ov_reg {
                            return ov.override_value.clone();
                        }
                    }
                }
            }
            return tr.value.clone();
        }
        fallback.to_string()
    }
}
