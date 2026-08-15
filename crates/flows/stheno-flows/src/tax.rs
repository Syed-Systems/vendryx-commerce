//! Tax Workflows (9 workflows):
//! 1. `create-tax-regions`
//! 2. `update-tax-regions`
//! 3. `delete-tax-regions`
//! 4. `create-tax-rates`
//! 5. `update-tax-rates`
//! 6. `delete-tax-rates`
//! 7. `create-tax-rate-rules`
//! 8. `delete-tax-rate-rules`
//! 9. `resolve-tax-lines`

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_tax::{
    CreateTaxRateInput, CreateTaxRateRuleInput, CreateTaxRegionInput, TaxContext, TaxRateDTO,
    TaxRateRuleDTO, TaxRegionDTO, TaxService, UpdateTaxRateInput, UpdateTaxRegionInput,
};
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepConfig, StepContext};

// ── 1. Create Tax Region Step ──────────────────────────────────────────────

pub struct CreateTaxRegionStep {
    service: Arc<TaxService>,
}

impl CreateTaxRegionStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreateTaxRegionStep {
    fn name(&self) -> &'static str {
        "CreateTaxRegionStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreateTaxRegionInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreateTaxRegionInput: {}", e),
                field: None,
            })?;

        let res: TaxRegionDTO = self.service.create_tax_region(parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let Some(id) = input.get("id").and_then(|v| v.as_str()) {
            let _ = self.service.delete_tax_region(id).await;
        }
        Ok(())
    }
}

// ── 2. Update Tax Region Step ──────────────────────────────────────────────

pub struct UpdateTaxRegionStep {
    service: Arc<TaxService>,
}

impl UpdateTaxRegionStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdateTaxRegionStep {
    fn name(&self) -> &'static str {
        "UpdateTaxRegionStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing tax_region id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        let parsed: UpdateTaxRegionInput =
            serde_json::from_value(input["data"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse UpdateTaxRegionInput: {}", e),
                field: None,
            })?;

        let res = self.service.update_tax_region(id, parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 3. Delete Tax Region Step ──────────────────────────────────────────────

pub struct DeleteTaxRegionStep {
    service: Arc<TaxService>,
}

impl DeleteTaxRegionStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeleteTaxRegionStep {
    fn name(&self) -> &'static str {
        "DeleteTaxRegionStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing tax_region id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        self.service.delete_tax_region(id).await?;
        Ok(json!({ "id": id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 4. Create Tax Rate Step ────────────────────────────────────────────────

pub struct CreateTaxRateStep {
    service: Arc<TaxService>,
}

impl CreateTaxRateStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreateTaxRateStep {
    fn name(&self) -> &'static str {
        "CreateTaxRateStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreateTaxRateInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreateTaxRateInput: {}", e),
                field: None,
            })?;

        let res: TaxRateDTO = self.service.create_tax_rate(parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let Some(id) = input.get("id").and_then(|v| v.as_str()) {
            let _ = self.service.delete_tax_rate(id).await;
        }
        Ok(())
    }
}

// ── 5. Update Tax Rate Step ────────────────────────────────────────────────

pub struct UpdateTaxRateStep {
    service: Arc<TaxService>,
}

impl UpdateTaxRateStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdateTaxRateStep {
    fn name(&self) -> &'static str {
        "UpdateTaxRateStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing tax_rate id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        let parsed: UpdateTaxRateInput =
            serde_json::from_value(input["data"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse UpdateTaxRateInput: {}", e),
                field: None,
            })?;

        let res = self.service.update_tax_rate(id, parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 6. Delete Tax Rate Step ────────────────────────────────────────────────

pub struct DeleteTaxRateStep {
    service: Arc<TaxService>,
}

impl DeleteTaxRateStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeleteTaxRateStep {
    fn name(&self) -> &'static str {
        "DeleteTaxRateStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing tax_rate id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        self.service.delete_tax_rate(id).await?;
        Ok(json!({ "id": id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 7. Create Tax Rate Rule Step ───────────────────────────────────────────

pub struct CreateTaxRateRuleStep {
    service: Arc<TaxService>,
}

impl CreateTaxRateRuleStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreateTaxRateRuleStep {
    fn name(&self) -> &'static str {
        "CreateTaxRateRuleStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let tax_rate_id = input["tax_rate_id"]
            .as_str()
            .ok_or_else(|| Error::Invalid {
                code: "missing_tax_rate_id",
                message: "Missing tax_rate_id in input".to_string(),
                field: Some("tax_rate_id".to_string()),
            })?;

        let parsed: CreateTaxRateRuleInput = serde_json::from_value(input["rule"].clone())
            .map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreateTaxRateRuleInput: {}", e),
                field: None,
            })?;

        let res: TaxRateRuleDTO = self
            .service
            .create_tax_rate_rule(tax_rate_id, parsed)
            .await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 8. Delete Tax Rate Rule Step ───────────────────────────────────────────

pub struct DeleteTaxRateRuleStep {
    service: Arc<TaxService>,
}

impl DeleteTaxRateRuleStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeleteTaxRateRuleStep {
    fn name(&self) -> &'static str {
        "DeleteTaxRateRuleStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let tax_rate_id = input["tax_rate_id"]
            .as_str()
            .ok_or_else(|| Error::Invalid {
                code: "missing_tax_rate_id",
                message: "Missing tax_rate_id in input".to_string(),
                field: Some("tax_rate_id".to_string()),
            })?;
        let rule_id = input["rule_id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_rule_id",
            message: "Missing rule_id in input".to_string(),
            field: Some("rule_id".to_string()),
        })?;

        self.service
            .delete_tax_rate_rule(tax_rate_id, rule_id)
            .await?;
        Ok(json!({ "tax_rate_id": tax_rate_id, "rule_id": rule_id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 9. Resolve Tax Lines Step ──────────────────────────────────────────────

pub struct ResolveTaxLinesStep {
    service: Arc<TaxService>,
}

impl ResolveTaxLinesStep {
    pub fn new(service: Arc<TaxService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for ResolveTaxLinesStep {
    fn name(&self) -> &'static str {
        "ResolveTaxLinesStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            store: false, // Pure-read workflow
            ..Default::default()
        }
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let ctx: TaxContext = serde_json::from_value(input).map_err(|e| Error::Invalid {
            code: "invalid_tax_context",
            message: format!("Failed to parse TaxContext: {}", e),
            field: None,
        })?;

        let lines = self.service.resolve_tax_lines(&ctx).await?;
        Ok(serde_json::to_value(lines).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── Builders ───────────────────────────────────────────────────────────────

pub fn build_create_tax_region_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreateTaxRegionStep::new(service)));
    coordinator
}

pub fn build_update_tax_region_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdateTaxRegionStep::new(service)));
    coordinator
}

pub fn build_delete_tax_region_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeleteTaxRegionStep::new(service)));
    coordinator
}

pub fn build_create_tax_rate_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreateTaxRateStep::new(service)));
    coordinator
}

pub fn build_update_tax_rate_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdateTaxRateStep::new(service)));
    coordinator
}

pub fn build_delete_tax_rate_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeleteTaxRateStep::new(service)));
    coordinator
}

pub fn build_create_tax_rate_rule_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreateTaxRateRuleStep::new(service)));
    coordinator
}

pub fn build_delete_tax_rate_rule_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeleteTaxRateRuleStep::new(service)));
    coordinator
}

pub fn build_resolve_tax_lines_saga(service: Arc<TaxService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(ResolveTaxLinesStep::new(service)));
    coordinator
}
