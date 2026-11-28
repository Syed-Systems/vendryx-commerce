//! Store Workflows (3 workflows):
//! 1. `create-stores`
//! 2. `update-stores`
//! 3. `delete-stores`

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_store::{CreateStoreInput, StoreDTO, StoreService, UpdateStoreInput};
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepContext};

// ── 1. Create Store Step ───────────────────────────────────────────────────

pub struct CreateStoreStep {
    service: Arc<StoreService>,
}

impl CreateStoreStep {
    pub fn new(service: Arc<StoreService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreateStoreStep {
    fn name(&self) -> &'static str {
        "CreateStoreStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreateStoreInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreateStoreInput: {e}"),
                field: None,
            })?;

        let res: StoreDTO = self.service.create_store(parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let Some(id) = input.get("id").and_then(|v| v.as_str()) {
            let _ = self.service.delete_store(id).await;
        }
        Ok(())
    }
}

// ── 2. Update Store Step ───────────────────────────────────────────────────

pub struct UpdateStoreStep {
    service: Arc<StoreService>,
}

impl UpdateStoreStep {
    pub fn new(service: Arc<StoreService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdateStoreStep {
    fn name(&self) -> &'static str {
        "UpdateStoreStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing store id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        let parsed: UpdateStoreInput =
            serde_json::from_value(input["data"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse UpdateStoreInput: {e}"),
                field: None,
            })?;

        let res = self.service.update_store(id, parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 3. Delete Store Step ───────────────────────────────────────────────────

pub struct DeleteStoreStep {
    service: Arc<StoreService>,
}

impl DeleteStoreStep {
    pub fn new(service: Arc<StoreService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeleteStoreStep {
    fn name(&self) -> &'static str {
        "DeleteStoreStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing store id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        self.service.delete_store(id).await?;
        Ok(json!({ "id": id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── Builders ───────────────────────────────────────────────────────────────

pub fn build_create_store_saga(service: Arc<StoreService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreateStoreStep::new(service)));
    coordinator
}

pub fn build_update_store_saga(service: Arc<StoreService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdateStoreStep::new(service)));
    coordinator
}

pub fn build_delete_store_saga(service: Arc<StoreService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeleteStoreStep::new(service)));
    coordinator
}
