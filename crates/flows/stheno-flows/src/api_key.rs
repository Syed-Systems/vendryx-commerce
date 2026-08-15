//! API Key workflows (WORKFLOWS §2, TASKS P7-21).

use std::sync::Arc;
use stheno_api_key::*;
use stheno_types::Result;

pub struct ApiKeyWorkflows {
    api_key_service: Arc<ApiKeyService>,
}

impl ApiKeyWorkflows {
    pub fn new(api_key_service: Arc<ApiKeyService>) -> Self {
        Self { api_key_service }
    }

    // 1. create_api_key_workflow
    pub async fn create_api_key(&self, input: CreateApiKeyInput) -> Result<CreatedApiKeyResponse> {
        self.api_key_service.create_api_key(input).await
    }

    // 2. update_api_key_workflow
    pub async fn update_api_key(&self, id: &str, input: UpdateApiKeyInput) -> Result<ApiKeyDTO> {
        self.api_key_service.update_key(id, input).await
    }

    // 3. revoke_api_key_workflow
    pub async fn revoke_api_key(&self, id: &str, revoked_by: Option<&str>) -> Result<ApiKeyDTO> {
        self.api_key_service.revoke_key(id, revoked_by).await
    }

    // 4. delete_api_key_workflow
    pub async fn delete_api_key(&self, id: &str) -> Result<ApiKeyDTO> {
        self.api_key_service.delete_key(id).await
    }

    // 5. validate_api_key_workflow
    pub async fn validate_api_key(&self, plain_token: &str) -> Result<ValidateKeyResult> {
        self.api_key_service.validate_key(plain_token).await
    }
}
