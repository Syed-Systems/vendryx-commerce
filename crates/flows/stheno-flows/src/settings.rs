use std::sync::Arc;
use stheno_settings::{
    LayoutPreferenceDTO, PropertyLabelDTO, SaveLayoutPreferenceInput, SaveViewConfigInput,
    SetPropertyLabelInput, SetStoreSettingInput, SettingsService, StoreSettingDTO, ViewConfigDTO,
};
use stheno_types::Result;

pub struct SetSettingWorkflow {
    settings_service: Arc<SettingsService>,
}

impl SetSettingWorkflow {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn run(&self, input: SetStoreSettingInput) -> Result<StoreSettingDTO> {
        self.settings_service.set_setting(input).await
    }
}

pub struct GetSettingWorkflow {
    settings_service: Arc<SettingsService>,
}

impl GetSettingWorkflow {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn run(&self, key: &str) -> Result<StoreSettingDTO> {
        self.settings_service.get_setting(key).await
    }
}

pub struct SaveViewConfigWorkflow {
    settings_service: Arc<SettingsService>,
}

impl SaveViewConfigWorkflow {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn run(&self, input: SaveViewConfigInput) -> Result<ViewConfigDTO> {
        self.settings_service.save_view_config(input).await
    }
}

pub struct SetPropertyLabelWorkflow {
    settings_service: Arc<SettingsService>,
}

impl SetPropertyLabelWorkflow {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn run(&self, input: SetPropertyLabelInput) -> Result<PropertyLabelDTO> {
        self.settings_service.set_property_label(input).await
    }
}

pub struct SaveLayoutPreferenceWorkflow {
    settings_service: Arc<SettingsService>,
}

impl SaveLayoutPreferenceWorkflow {
    pub fn new(settings_service: Arc<SettingsService>) -> Self {
        Self { settings_service }
    }

    pub async fn run(&self, input: SaveLayoutPreferenceInput) -> Result<LayoutPreferenceDTO> {
        self.settings_service.save_layout_preference(input).await
    }
}
