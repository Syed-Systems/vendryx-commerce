//! Region Workflows (3 workflows):
//! 1. `create-regions`
//! 2. `update-regions`
//! 3. `delete-regions`

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_region::{CreateRegionInput, RegionDTO, RegionService, UpdateRegionInput};
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepContext};

// ── 1. Create Region Step ──────────────────────────────────────────────────

pub struct CreateRegionStep {
    service: Arc<RegionService>,
}

impl CreateRegionStep {
    pub fn new(service: Arc<RegionService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreateRegionStep {
    fn name(&self) -> &'static str {
        "CreateRegionStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreateRegionInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreateRegionInput: {e}"),
                field: None,
            })?;

        let res: RegionDTO = self.service.create_region(parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let Some(id) = input.get("id").and_then(|v| v.as_str()) {
            let _ = self.service.delete_region(id).await;
        }
        Ok(())
    }
}

// ── 2. Update Region Step ──────────────────────────────────────────────────

pub struct UpdateRegionStep {
    service: Arc<RegionService>,
}

impl UpdateRegionStep {
    pub fn new(service: Arc<RegionService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdateRegionStep {
    fn name(&self) -> &'static str {
        "UpdateRegionStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing region id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        let parsed: UpdateRegionInput =
            serde_json::from_value(input["data"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse UpdateRegionInput: {e}"),
                field: None,
            })?;

        let res = self.service.update_region(id, parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 3. Delete Region Step ──────────────────────────────────────────────────

pub struct DeleteRegionStep {
    service: Arc<RegionService>,
}

impl DeleteRegionStep {
    pub fn new(service: Arc<RegionService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeleteRegionStep {
    fn name(&self) -> &'static str {
        "DeleteRegionStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing region id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        self.service.delete_region(id).await?;
        Ok(json!({ "id": id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── Builders ───────────────────────────────────────────────────────────────

pub fn build_create_region_saga(service: Arc<RegionService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreateRegionStep::new(service)));
    coordinator
}

pub fn build_update_region_saga(service: Arc<RegionService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdateRegionStep::new(service)));
    coordinator
}

pub fn build_delete_region_saga(service: Arc<RegionService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeleteRegionStep::new(service)));
    coordinator
}
