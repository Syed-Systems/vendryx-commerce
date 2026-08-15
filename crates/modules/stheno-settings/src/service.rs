use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use stheno_types::{Error, Id, Result};

use crate::dto::{
    LayoutPreferenceDTO, PropertyLabelDTO, SaveLayoutPreferenceInput, SaveViewConfigInput,
    SetPropertyLabelInput, SetStoreSettingInput, StoreSettingDTO, ViewConfigDTO,
};
use crate::models::{LayoutPreference, PropertyLabel, StoreSetting, ViewConfig};

pub struct SettingsService {
    settings: RwLock<HashMap<String, StoreSetting>>,
    view_configs: RwLock<HashMap<String, ViewConfig>>,
    property_labels: RwLock<HashMap<String, PropertyLabel>>,
    layout_preferences: RwLock<HashMap<String, LayoutPreference>>,
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsService {
    pub fn new() -> Self {
        Self {
            settings: RwLock::new(HashMap::new()),
            view_configs: RwLock::new(HashMap::new()),
            property_labels: RwLock::new(HashMap::new()),
            layout_preferences: RwLock::new(HashMap::new()),
        }
    }

    // ==========================================
    // 1. Store Settings
    // ==========================================

    pub async fn set_setting(&self, input: SetStoreSettingInput) -> Result<StoreSettingDTO> {
        let now = Utc::now();
        let id = Id::new();

        let setting = StoreSetting {
            id,
            key: input.key.clone(),
            value: input.value,
            category: input.category,
            is_secret: input.is_secret.unwrap_or(false),
            created_at: now,
            updated_at: now,
        };

        let dto = StoreSettingDTO {
            id: setting.id.to_string(),
            key: setting.key.clone(),
            value: setting.value.clone(),
            category: setting.category.clone(),
            is_secret: setting.is_secret,
            created_at: setting.created_at,
            updated_at: setting.updated_at,
        };

        {
            let mut map = self.settings.write().unwrap();
            map.insert(setting.key.clone(), setting);
        }

        Ok(dto)
    }

    pub async fn get_setting(&self, key: &str) -> Result<StoreSettingDTO> {
        let map = self.settings.read().unwrap();
        let s = map.get(key).ok_or_else(|| Error::NotFound {
            entity: "store_setting",
            id: key.to_string(),
        })?;

        Ok(StoreSettingDTO {
            id: s.id.to_string(),
            key: s.key.clone(),
            value: s.value.clone(),
            category: s.category.clone(),
            is_secret: s.is_secret,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
    }

    pub async fn list_settings_by_category(&self, category: &str) -> Result<Vec<StoreSettingDTO>> {
        let map = self.settings.read().unwrap();
        let list: Vec<StoreSettingDTO> = map
            .values()
            .filter(|s| s.category == category)
            .map(|s| StoreSettingDTO {
                id: s.id.to_string(),
                key: s.key.clone(),
                value: s.value.clone(),
                category: s.category.clone(),
                is_secret: s.is_secret,
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect();
        Ok(list)
    }

    // ==========================================
    // 2. View Configurations & Custom Layouts
    // ==========================================

    pub async fn save_view_config(&self, input: SaveViewConfigInput) -> Result<ViewConfigDTO> {
        let id = Id::new();
        let now = Utc::now();
        let key = format!(
            "{}:{}",
            input.user_id.as_deref().unwrap_or("global"),
            input.scope
        );

        let view = ViewConfig {
            id,
            user_id: input.user_id,
            scope: input.scope,
            columns: input.columns,
            filters: input.filters,
            sort_by: input.sort_by,
            sort_order: input.sort_order,
            page_size: input.page_size.unwrap_or(20),
            created_at: now,
            updated_at: now,
        };

        let dto = ViewConfigDTO {
            id: view.id.to_string(),
            user_id: view.user_id.clone(),
            scope: view.scope.clone(),
            columns: view.columns.clone(),
            filters: view.filters.clone(),
            sort_by: view.sort_by.clone(),
            sort_order: view.sort_order.clone(),
            page_size: view.page_size,
            created_at: view.created_at,
            updated_at: view.updated_at,
        };

        {
            let mut map = self.view_configs.write().unwrap();
            map.insert(key, view);
        }

        Ok(dto)
    }

    pub async fn get_view_config(
        &self,
        user_id: Option<&str>,
        scope: &str,
    ) -> Result<ViewConfigDTO> {
        let user_key = format!("{}:{}", user_id.unwrap_or("global"), scope);
        let global_key = format!("global:{}", scope);

        let map = self.view_configs.read().unwrap();
        let view = map
            .get(&user_key)
            .or_else(|| map.get(&global_key))
            .ok_or_else(|| Error::NotFound {
                entity: "view_config",
                id: scope.to_string(),
            })?;

        Ok(ViewConfigDTO {
            id: view.id.to_string(),
            user_id: view.user_id.clone(),
            scope: view.scope.clone(),
            columns: view.columns.clone(),
            filters: view.filters.clone(),
            sort_by: view.sort_by.clone(),
            sort_order: view.sort_order.clone(),
            page_size: view.page_size,
            created_at: view.created_at,
            updated_at: view.updated_at,
        })
    }

    // ==========================================
    // 3. Property Labels & Preferences
    // ==========================================

    pub async fn set_property_label(
        &self,
        input: SetPropertyLabelInput,
    ) -> Result<PropertyLabelDTO> {
        let id = Id::new();
        let now = Utc::now();
        let key = format!("{}:{}", input.entity_type, input.property_key);

        let pl = PropertyLabel {
            id,
            entity_type: input.entity_type,
            property_key: input.property_key,
            label: input.label,
            help_text: input.help_text,
            created_at: now,
            updated_at: now,
        };

        let dto = PropertyLabelDTO {
            id: pl.id.to_string(),
            entity_type: pl.entity_type.clone(),
            property_key: pl.property_key.clone(),
            label: pl.label.clone(),
            help_text: pl.help_text.clone(),
            created_at: pl.created_at,
            updated_at: pl.updated_at,
        };

        {
            let mut map = self.property_labels.write().unwrap();
            map.insert(key, pl);
        }

        Ok(dto)
    }

    pub async fn get_property_label(
        &self,
        entity_type: &str,
        property_key: &str,
    ) -> Result<PropertyLabelDTO> {
        let key = format!("{}:{}", entity_type, property_key);
        let map = self.property_labels.read().unwrap();
        let pl = map.get(&key).ok_or_else(|| Error::NotFound {
            entity: "property_label",
            id: key,
        })?;

        Ok(PropertyLabelDTO {
            id: pl.id.to_string(),
            entity_type: pl.entity_type.clone(),
            property_key: pl.property_key.clone(),
            label: pl.label.clone(),
            help_text: pl.help_text.clone(),
            created_at: pl.created_at,
            updated_at: pl.updated_at,
        })
    }

    pub async fn save_layout_preference(
        &self,
        input: SaveLayoutPreferenceInput,
    ) -> Result<LayoutPreferenceDTO> {
        let id = Id::new();
        let now = Utc::now();
        let key = format!("{}:{}", input.user_id, input.layout_key);

        let pref = LayoutPreference {
            id,
            user_id: input.user_id,
            layout_key: input.layout_key,
            config: input.config,
            created_at: now,
            updated_at: now,
        };

        let dto = LayoutPreferenceDTO {
            id: pref.id.to_string(),
            user_id: pref.user_id.clone(),
            layout_key: pref.layout_key.clone(),
            config: pref.config.clone(),
            created_at: pref.created_at,
            updated_at: pref.updated_at,
        };

        {
            let mut map = self.layout_preferences.write().unwrap();
            map.insert(key, pref);
        }

        Ok(dto)
    }

    pub async fn get_layout_preference(
        &self,
        user_id: &str,
        layout_key: &str,
    ) -> Result<LayoutPreferenceDTO> {
        let key = format!("{}:{}", user_id, layout_key);
        let map = self.layout_preferences.read().unwrap();
        let pref = map.get(&key).ok_or_else(|| Error::NotFound {
            entity: "layout_preference",
            id: key,
        })?;

        Ok(LayoutPreferenceDTO {
            id: pref.id.to_string(),
            user_id: pref.user_id.clone(),
            layout_key: pref.layout_key.clone(),
            config: pref.config.clone(),
            created_at: pref.created_at,
            updated_at: pref.updated_at,
        })
    }
}
