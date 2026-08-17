use std::sync::Arc;
use stheno_translation::{
    CreateLocaleInput, LocaleDTO, SetTranslationInput, SetTranslationOverrideInput, TranslationDTO,
    TranslationOverrideDTO, TranslationService,
};
use stheno_types::Result;

pub struct CreateLocaleWorkflow {
    translation_service: Arc<TranslationService>,
}

impl CreateLocaleWorkflow {
    pub fn new(translation_service: Arc<TranslationService>) -> Self {
        Self {
            translation_service,
        }
    }

    pub async fn run(&self, input: CreateLocaleInput) -> Result<LocaleDTO> {
        self.translation_service.create_locale(input).await
    }
}

pub struct SetTranslationWorkflow {
    translation_service: Arc<TranslationService>,
}

impl SetTranslationWorkflow {
    pub fn new(translation_service: Arc<TranslationService>) -> Self {
        Self {
            translation_service,
        }
    }

    pub async fn run(&self, input: SetTranslationInput) -> Result<TranslationDTO> {
        self.translation_service.set_translation(input).await
    }
}

pub struct SetTranslationOverrideWorkflow {
    translation_service: Arc<TranslationService>,
}

impl SetTranslationOverrideWorkflow {
    pub fn new(translation_service: Arc<TranslationService>) -> Self {
        Self {
            translation_service,
        }
    }

    pub async fn run(&self, input: SetTranslationOverrideInput) -> Result<TranslationOverrideDTO> {
        self.translation_service
            .set_translation_override(input)
            .await
    }
}

pub struct ResolveFieldTranslationWorkflow {
    translation_service: Arc<TranslationService>,
}

impl ResolveFieldTranslationWorkflow {
    pub fn new(translation_service: Arc<TranslationService>) -> Self {
        Self {
            translation_service,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        &self,
        locale_code: &str,
        entity_type: &str,
        entity_id: &str,
        field_name: &str,
        sales_channel_id: Option<&str>,
        region_id: Option<&str>,
        fallback: &str,
    ) -> String {
        self.translation_service
            .resolve_scoped_field(
                locale_code,
                entity_type,
                entity_id,
                field_name,
                sales_channel_id,
                region_id,
                fallback,
            )
            .await
    }
}

pub struct BatchTranslateWorkflow {
    translation_service: Arc<TranslationService>,
}

impl BatchTranslateWorkflow {
    pub fn new(translation_service: Arc<TranslationService>) -> Self {
        Self {
            translation_service,
        }
    }

    pub async fn run(&self, inputs: Vec<SetTranslationInput>) -> Result<Vec<TranslationDTO>> {
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            results.push(self.translation_service.set_translation(input).await?);
        }
        Ok(results)
    }
}
