use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    LayoutPreferenceDTO, PropertyLabelDTO, SaveLayoutPreferenceInput, SaveViewConfigInput,
    SetPropertyLabelInput, SetStoreSettingInput, StoreSettingDTO, ViewConfigDTO,
};
use crate::models::{LayoutPreference, PropertyLabel, StoreSetting, ViewConfig};

#[derive(Clone)]
pub struct SettingsService {
    settings: PgCrudRepository<StoreSetting>,
    view_configs: PgCrudRepository<ViewConfig>,
    property_labels: PgCrudRepository<PropertyLabel>,
    layout_preferences: PgCrudRepository<LayoutPreference>,
}

impl SettingsService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            settings: PgCrudRepository::new(pool.clone()),
            view_configs: PgCrudRepository::new(pool.clone()),
            property_labels: PgCrudRepository::new(pool.clone()),
            layout_preferences: PgCrudRepository::new(pool),
        }
    }

    // ==========================================
    // 1. Store Settings
    // ==========================================

    pub async fn set_setting(&self, input: SetStoreSettingInput) -> Result<StoreSettingDTO> {
        let filter = FilterSpec::new()
            .with_predicate("key", FilterOp::Eq(input.key.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.settings.list(None, &filter).await?;
        let now = Utc::now();

        let setting = if let Some(mut existing_setting) = existing.into_iter().next() {
            existing_setting.value = input.value;
            existing_setting.category = input.category;
            existing_setting.is_secret = input.is_secret.unwrap_or(existing_setting.is_secret);
            existing_setting.updated_at = now;
            self.settings
                .update(None, &existing_setting.id.to_string(), existing_setting)
                .await?
        } else {
            let id = Id::new();
            let setting = StoreSetting {
                id,
                key: input.key,
                value: input.value,
                category: input.category,
                is_secret: input.is_secret.unwrap_or(false),
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.settings.create(None, setting).await?
        };

        Ok(StoreSettingDTO {
            id: setting.id.to_string(),
            key: setting.key,
            value: setting.value,
            category: setting.category,
            is_secret: setting.is_secret,
            created_at: setting.created_at,
            updated_at: setting.updated_at,
        })
    }

    pub async fn get_setting(&self, key: &str) -> Result<StoreSettingDTO> {
        let filter = FilterSpec::new()
            .with_predicate("key", FilterOp::Eq(key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.settings.list(None, &filter).await?;
        let s = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "store_setting",
            id: key.to_string(),
        })?;

        Ok(StoreSettingDTO {
            id: s.id.to_string(),
            key: s.key,
            value: s.value,
            category: s.category,
            is_secret: s.is_secret,
            created_at: s.created_at,
            updated_at: s.updated_at,
        })
    }

    pub async fn list_settings_by_category(&self, category: &str) -> Result<Vec<StoreSettingDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("category", FilterOp::Eq(category.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.settings.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|s| StoreSettingDTO {
                id: s.id.to_string(),
                key: s.key,
                value: s.value,
                category: s.category,
                is_secret: s.is_secret,
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect())
    }

    // ==========================================
    // 2. View Configurations & Custom Layouts
    // ==========================================

    pub async fn save_view_config(&self, input: SaveViewConfigInput) -> Result<ViewConfigDTO> {
        let mut filter = FilterSpec::new()
            .with_predicate("scope", FilterOp::Eq(input.scope.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        if let Some(uid) = &input.user_id {
            filter = filter.with_predicate("user_id", FilterOp::Eq(uid.clone()));
        } else {
            filter = filter.with_predicate("user_id", FilterOp::IsNull);
        }

        let existing = self.view_configs.list(None, &filter).await?;
        let now = Utc::now();

        let view = if let Some(mut existing_cfg) = existing.into_iter().next() {
            existing_cfg.columns = input.columns;
            existing_cfg.filters = input.filters;
            existing_cfg.sort_by = input.sort_by;
            existing_cfg.sort_order = input.sort_order;
            existing_cfg.page_size = input.page_size.unwrap_or(existing_cfg.page_size);
            existing_cfg.updated_at = now;
            self.view_configs
                .update(None, &existing_cfg.id.to_string(), existing_cfg)
                .await?
        } else {
            let id = Id::new();
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
                deleted_at: None,
            };
            self.view_configs.create(None, view).await?
        };

        Ok(ViewConfigDTO {
            id: view.id.to_string(),
            user_id: view.user_id,
            scope: view.scope,
            columns: view.columns,
            filters: view.filters,
            sort_by: view.sort_by,
            sort_order: view.sort_order,
            page_size: view.page_size,
            created_at: view.created_at,
            updated_at: view.updated_at,
        })
    }

    pub async fn get_view_config(
        &self,
        user_id: Option<&str>,
        scope: &str,
    ) -> Result<ViewConfigDTO> {
        if let Some(uid) = user_id {
            let user_filter = FilterSpec::new()
                .with_predicate("scope", FilterOp::Eq(scope.to_string()))
                .with_predicate("user_id", FilterOp::Eq(uid.to_string()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 1);

            let user_list = self.view_configs.list(None, &user_filter).await?;
            if let Some(view) = user_list.into_iter().next() {
                return Ok(ViewConfigDTO {
                    id: view.id.to_string(),
                    user_id: view.user_id,
                    scope: view.scope,
                    columns: view.columns,
                    filters: view.filters,
                    sort_by: view.sort_by,
                    sort_order: view.sort_order,
                    page_size: view.page_size,
                    created_at: view.created_at,
                    updated_at: view.updated_at,
                });
            }
        }

        let global_filter = FilterSpec::new()
            .with_predicate("scope", FilterOp::Eq(scope.to_string()))
            .with_predicate("user_id", FilterOp::IsNull)
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let global_list = self.view_configs.list(None, &global_filter).await?;
        let view = global_list
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound {
                entity: "view_config",
                id: scope.to_string(),
            })?;

        Ok(ViewConfigDTO {
            id: view.id.to_string(),
            user_id: view.user_id,
            scope: view.scope,
            columns: view.columns,
            filters: view.filters,
            sort_by: view.sort_by,
            sort_order: view.sort_order,
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
        let filter = FilterSpec::new()
            .with_predicate("entity_type", FilterOp::Eq(input.entity_type.clone()))
            .with_predicate("property_key", FilterOp::Eq(input.property_key.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.property_labels.list(None, &filter).await?;
        let now = Utc::now();

        let pl = if let Some(mut existing_pl) = existing.into_iter().next() {
            existing_pl.label = input.label;
            existing_pl.help_text = input.help_text;
            existing_pl.updated_at = now;
            self.property_labels
                .update(None, &existing_pl.id.to_string(), existing_pl)
                .await?
        } else {
            let id = Id::new();
            let pl = PropertyLabel {
                id,
                entity_type: input.entity_type,
                property_key: input.property_key,
                label: input.label,
                help_text: input.help_text,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.property_labels.create(None, pl).await?
        };

        Ok(PropertyLabelDTO {
            id: pl.id.to_string(),
            entity_type: pl.entity_type,
            property_key: pl.property_key,
            label: pl.label,
            help_text: pl.help_text,
            created_at: pl.created_at,
            updated_at: pl.updated_at,
        })
    }

    pub async fn get_property_label(
        &self,
        entity_type: &str,
        property_key: &str,
    ) -> Result<PropertyLabelDTO> {
        let filter = FilterSpec::new()
            .with_predicate("entity_type", FilterOp::Eq(entity_type.to_string()))
            .with_predicate("property_key", FilterOp::Eq(property_key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.property_labels.list(None, &filter).await?;
        let pl = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "property_label",
            id: format!("{entity_type}:{property_key}"),
        })?;

        Ok(PropertyLabelDTO {
            id: pl.id.to_string(),
            entity_type: pl.entity_type,
            property_key: pl.property_key,
            label: pl.label,
            help_text: pl.help_text,
            created_at: pl.created_at,
            updated_at: pl.updated_at,
        })
    }

    pub async fn save_layout_preference(
        &self,
        input: SaveLayoutPreferenceInput,
    ) -> Result<LayoutPreferenceDTO> {
        let filter = FilterSpec::new()
            .with_predicate("user_id", FilterOp::Eq(input.user_id.clone()))
            .with_predicate("layout_key", FilterOp::Eq(input.layout_key.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.layout_preferences.list(None, &filter).await?;
        let now = Utc::now();

        let pref = if let Some(mut existing_pref) = existing.into_iter().next() {
            existing_pref.config = input.config;
            existing_pref.updated_at = now;
            self.layout_preferences
                .update(None, &existing_pref.id.to_string(), existing_pref)
                .await?
        } else {
            let id = Id::new();
            let pref = LayoutPreference {
                id,
                user_id: input.user_id,
                layout_key: input.layout_key,
                config: input.config,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.layout_preferences.create(None, pref).await?
        };

        Ok(LayoutPreferenceDTO {
            id: pref.id.to_string(),
            user_id: pref.user_id,
            layout_key: pref.layout_key,
            config: pref.config,
            created_at: pref.created_at,
            updated_at: pref.updated_at,
        })
    }

    pub async fn get_layout_preference(
        &self,
        user_id: &str,
        layout_key: &str,
    ) -> Result<LayoutPreferenceDTO> {
        let filter = FilterSpec::new()
            .with_predicate("user_id", FilterOp::Eq(user_id.to_string()))
            .with_predicate("layout_key", FilterOp::Eq(layout_key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.layout_preferences.list(None, &filter).await?;
        let pref = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "layout_preference",
            id: format!("{user_id}:{layout_key}"),
        })?;

        Ok(LayoutPreferenceDTO {
            id: pref.id.to_string(),
            user_id: pref.user_id,
            layout_key: pref.layout_key,
            config: pref.config,
            created_at: pref.created_at,
            updated_at: pref.updated_at,
        })
    }
}
