use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateLocaleInput, LocaleDTO, SetTranslationInput, SetTranslationOverrideInput, TranslationDTO,
    TranslationOverrideDTO,
};
use crate::models::{Locale, Translation, TranslationOverride};

#[derive(Clone)]
pub struct TranslationService {
    locales: PgCrudRepository<Locale>,
    translations: PgCrudRepository<Translation>,
    overrides: PgCrudRepository<TranslationOverride>,
}

impl TranslationService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            locales: PgCrudRepository::new(pool.clone()),
            translations: PgCrudRepository::new(pool.clone()),
            overrides: PgCrudRepository::new(pool),
        }
    }

    // ==========================================
    // 1. Locales
    // ==========================================

    pub async fn create_locale(&self, input: CreateLocaleInput) -> Result<LocaleDTO> {
        let filter = FilterSpec::new()
            .with_predicate("code", FilterOp::Eq(input.code.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.locales.list(None, &filter).await?;
        if !existing.is_empty() {
            return Err(Error::Conflict {
                code: "locale_exists",
                message: format!("Locale with code '{}' already exists", input.code),
            });
        }

        let id = Id::new();
        let now = Utc::now();

        let locale = Locale {
            id,
            code: input.code,
            name: input.name,
            is_default: input.is_default.unwrap_or(false),
            is_enabled: input.is_enabled.unwrap_or(true),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.locales.create(None, locale).await?;

        Ok(LocaleDTO {
            id: created.id.to_string(),
            code: created.code,
            name: created.name,
            is_default: created.is_default,
            is_enabled: created.is_enabled,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn get_locale(&self, code: &str) -> Result<LocaleDTO> {
        let filter = FilterSpec::new()
            .with_predicate("code", FilterOp::Eq(code.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.locales.list(None, &filter).await?;
        let l = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "locale",
            id: code.to_string(),
        })?;

        Ok(LocaleDTO {
            id: l.id.to_string(),
            code: l.code,
            name: l.name,
            is_default: l.is_default,
            is_enabled: l.is_enabled,
            created_at: l.created_at,
            updated_at: l.updated_at,
        })
    }

    pub async fn list_locales(&self) -> Result<Vec<LocaleDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.locales.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|l| LocaleDTO {
                id: l.id.to_string(),
                code: l.code,
                name: l.name,
                is_default: l.is_default,
                is_enabled: l.is_enabled,
                created_at: l.created_at,
                updated_at: l.updated_at,
            })
            .collect())
    }

    // ==========================================
    // 2. Translations & Scoped Overrides
    // ==========================================

    /// Every translation, optionally narrowed by locale and entity type.
    ///
    /// `/admin/translations` was mounted `post`-only, so the translations screen could
    /// write a value and had no way to read the table back.
    pub async fn list_translations(
        &self,
        locale_code: Option<&str>,
        entity_type: Option<&str>,
    ) -> Result<Vec<TranslationDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("entity_type", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        if let Some(locale) = locale_code {
            filter = filter.with_predicate("locale_code", FilterOp::Eq(locale.to_string()));
        }
        if let Some(entity) = entity_type {
            filter = filter.with_predicate("entity_type", FilterOp::Eq(entity.to_string()));
        }

        Ok(self
            .translations
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|t| TranslationDTO {
                id: t.id.to_string(),
                locale_code: t.locale_code,
                entity_type: t.entity_type,
                entity_id: t.entity_id,
                field_name: t.field_name,
                value: t.value,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect())
    }

    pub async fn set_translation(&self, input: SetTranslationInput) -> Result<TranslationDTO> {
        let filter = FilterSpec::new()
            .with_predicate("locale_code", FilterOp::Eq(input.locale_code.clone()))
            .with_predicate("entity_type", FilterOp::Eq(input.entity_type.clone()))
            .with_predicate("entity_id", FilterOp::Eq(input.entity_id.clone()))
            .with_predicate("field_name", FilterOp::Eq(input.field_name.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.translations.list(None, &filter).await?;
        let now = Utc::now();

        let translation = if let Some(mut existing_tr) = existing.into_iter().next() {
            existing_tr.value = input.value;
            existing_tr.updated_at = now;
            self.translations
                .update(None, &existing_tr.id.to_string(), existing_tr)
                .await?
        } else {
            let id = Id::new();
            let tr = Translation {
                id,
                locale_code: input.locale_code,
                entity_type: input.entity_type,
                entity_id: input.entity_id,
                field_name: input.field_name,
                value: input.value,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            self.translations.create(None, tr).await?
        };

        Ok(TranslationDTO {
            id: translation.id.to_string(),
            locale_code: translation.locale_code,
            entity_type: translation.entity_type,
            entity_id: translation.entity_id,
            field_name: translation.field_name,
            value: translation.value,
            created_at: translation.created_at,
            updated_at: translation.updated_at,
        })
    }

    pub async fn set_translation_override(
        &self,
        input: SetTranslationOverrideInput,
    ) -> Result<TranslationOverrideDTO> {
        let id = Id::new();
        let now = Utc::now();

        let tr_override = TranslationOverride {
            id,
            translation_id: input.translation_id,
            sales_channel_id: input.sales_channel_id,
            region_id: input.region_id,
            override_value: input.override_value,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.overrides.create(None, tr_override).await?;

        Ok(TranslationOverrideDTO {
            id: created.id.to_string(),
            translation_id: created.translation_id,
            sales_channel_id: created.sales_channel_id,
            region_id: created.region_id,
            override_value: created.override_value,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    #[allow(clippy::too_many_arguments)]
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
        let filter = FilterSpec::new()
            .with_predicate("locale_code", FilterOp::Eq(locale_code.to_string()))
            .with_predicate("entity_type", FilterOp::Eq(entity_type.to_string()))
            .with_predicate("entity_id", FilterOp::Eq(entity_id.to_string()))
            .with_predicate("field_name", FilterOp::Eq(field_name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        if let Ok(list) = self.translations.list(None, &filter).await {
            if let Some(tr) = list.into_iter().next() {
                let ov_filter = FilterSpec::new()
                    .with_predicate("translation_id", FilterOp::Eq(tr.id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100);

                if let Ok(ov_list) = self.overrides.list(None, &ov_filter).await {
                    for ov in ov_list {
                        if let (Some(sc), Some(ov_sc)) = (sales_channel_id, &ov.sales_channel_id) {
                            if sc == ov_sc {
                                return ov.override_value;
                            }
                        }
                        if let (Some(reg), Some(ov_reg)) = (region_id, &ov.region_id) {
                            if reg == ov_reg {
                                return ov.override_value;
                            }
                        }
                    }
                }
                return tr.value;
            }
        }
        fallback.to_string()
    }
}
