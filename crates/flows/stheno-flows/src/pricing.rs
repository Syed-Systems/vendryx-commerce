//! Pricing Workflows (10 workflows):
//! 1. `create-price-sets`
//! 2. `update-price-sets`
//! 3. `delete-price-sets`
//! 4. `create-price-lists`
//! 5. `update-price-lists`
//! 6. `delete-price-lists`
//! 7. `batch-price-list-prices`
//! 8. `create-price-preferences`
//! 9. `update-price-preferences`
//! 10. `calculate-prices`

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use stheno_pricing::{
    BatchPriceListPricesInput, CreatePriceListInput, CreatePricePreferenceInput,
    CreatePriceSetInput, PriceListDTO, PriceSetDTO, PricingContext, PricingService,
    UpdatePriceListInput, UpdatePriceSetInput,
};
use stheno_types::{Error, Result};
use stheno_workflow::{SagaCoordinator, Step, StepConfig, StepContext};

// ── 1. Create Price Sets Step ──────────────────────────────────────────────

pub struct CreatePriceSetStep {
    service: Arc<PricingService>,
}

impl CreatePriceSetStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreatePriceSetStep {
    fn name(&self) -> &'static str {
        "CreatePriceSetStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreatePriceSetInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreatePriceSetInput: {}", e),
                field: None,
            })?;

        let res: PriceSetDTO = self.service.create_price_set(parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let Some(id) = input.get("id").and_then(|v| v.as_str()) {
            let _ = self.service.delete_price_set(id).await;
        }
        Ok(())
    }
}

// ── 2. Update Price Sets Step ──────────────────────────────────────────────

pub struct UpdatePriceSetStep {
    service: Arc<PricingService>,
}

impl UpdatePriceSetStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdatePriceSetStep {
    fn name(&self) -> &'static str {
        "UpdatePriceSetStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing price_set id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        let parsed: UpdatePriceSetInput =
            serde_json::from_value(input["data"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse UpdatePriceSetInput: {}", e),
                field: None,
            })?;

        let res = self.service.update_price_set(id, parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 3. Delete Price Sets Step ──────────────────────────────────────────────

pub struct DeletePriceSetStep {
    service: Arc<PricingService>,
}

impl DeletePriceSetStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeletePriceSetStep {
    fn name(&self) -> &'static str {
        "DeletePriceSetStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing price_set id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        self.service.delete_price_set(id).await?;
        Ok(json!({ "id": id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 4. Create Price Lists Step ─────────────────────────────────────────────

pub struct CreatePriceListStep {
    service: Arc<PricingService>,
}

impl CreatePriceListStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreatePriceListStep {
    fn name(&self) -> &'static str {
        "CreatePriceListStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreatePriceListInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreatePriceListInput: {}", e),
                field: None,
            })?;

        let res: PriceListDTO = self.service.create_price_list(parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let Some(id) = input.get("id").and_then(|v| v.as_str()) {
            let _ = self.service.delete_price_list(id).await;
        }
        Ok(())
    }
}

// ── 5. Update Price Lists Step ─────────────────────────────────────────────

pub struct UpdatePriceListStep {
    service: Arc<PricingService>,
}

impl UpdatePriceListStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdatePriceListStep {
    fn name(&self) -> &'static str {
        "UpdatePriceListStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing price_list id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        let parsed: UpdatePriceListInput =
            serde_json::from_value(input["data"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse UpdatePriceListInput: {}", e),
                field: None,
            })?;

        let res = self.service.update_price_list(id, parsed).await?;
        Ok(serde_json::to_value(res).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 6. Delete Price Lists Step ─────────────────────────────────────────────

pub struct DeletePriceListStep {
    service: Arc<PricingService>,
}

impl DeletePriceListStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for DeletePriceListStep {
    fn name(&self) -> &'static str {
        "DeletePriceListStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let id = input["id"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_id",
            message: "Missing price_list id in input".to_string(),
            field: Some("id".to_string()),
        })?;

        self.service.delete_price_list(id).await?;
        Ok(json!({ "id": id, "deleted": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 7. Batch Price List Prices Step ────────────────────────────────────────

pub struct BatchPriceListPricesStep {
    service: Arc<PricingService>,
}

impl BatchPriceListPricesStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for BatchPriceListPricesStep {
    fn name(&self) -> &'static str {
        "BatchPriceListPricesStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let price_list_id = input["price_list_id"]
            .as_str()
            .ok_or_else(|| Error::Invalid {
                code: "missing_price_list_id",
                message: "Missing price_list_id in input".to_string(),
                field: Some("price_list_id".to_string()),
            })?;

        let parsed: BatchPriceListPricesInput = serde_json::from_value(input["data"].clone())
            .map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse BatchPriceListPricesInput: {}", e),
                field: None,
            })?;

        self.service
            .batch_price_list_prices(price_list_id, parsed)
            .await?;
        Ok(json!({ "price_list_id": price_list_id, "success": true }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 8. Create Price Preference Step ────────────────────────────────────────

pub struct CreatePricePreferenceStep {
    service: Arc<PricingService>,
}

impl CreatePricePreferenceStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CreatePricePreferenceStep {
    fn name(&self) -> &'static str {
        "CreatePricePreferenceStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let parsed: CreatePricePreferenceInput =
            serde_json::from_value(input).map_err(|e| Error::Invalid {
                code: "invalid_input",
                message: format!("Failed to parse CreatePricePreferenceInput: {}", e),
                field: None,
            })?;

        self.service
            .set_price_preference(&parsed.attribute, &parsed.value, parsed.is_tax_inclusive)
            .await?;

        Ok(json!({
            "attribute": parsed.attribute,
            "value": parsed.value,
            "is_tax_inclusive": parsed.is_tax_inclusive
        }))
    }

    async fn compensate(&self, _ctx: &StepContext, input: Value) -> Result<()> {
        if let (Some(attr), Some(val)) = (
            input.get("attribute").and_then(|v| v.as_str()),
            input.get("value").and_then(|v| v.as_str()),
        ) {
            let _ = self.service.delete_price_preference(attr, val).await;
        }
        Ok(())
    }
}

// ── 9. Update Price Preference Step ────────────────────────────────────────

pub struct UpdatePricePreferenceStep {
    service: Arc<PricingService>,
}

impl UpdatePricePreferenceStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for UpdatePricePreferenceStep {
    fn name(&self) -> &'static str {
        "UpdatePricePreferenceStep"
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let attribute = input["attribute"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_attribute",
            message: "Missing attribute in input".to_string(),
            field: Some("attribute".to_string()),
        })?;
        let value = input["value"].as_str().ok_or_else(|| Error::Invalid {
            code: "missing_value",
            message: "Missing value in input".to_string(),
            field: Some("value".to_string()),
        })?;
        let is_tax_inclusive = input["is_tax_inclusive"].as_bool().unwrap_or(false);

        self.service
            .set_price_preference(attribute, value, is_tax_inclusive)
            .await?;

        Ok(json!({
            "attribute": attribute,
            "value": value,
            "is_tax_inclusive": is_tax_inclusive
        }))
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── 10. Calculate Prices Step ──────────────────────────────────────────────

pub struct CalculatePricesStep {
    service: Arc<PricingService>,
}

impl CalculatePricesStep {
    pub fn new(service: Arc<PricingService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Step for CalculatePricesStep {
    fn name(&self) -> &'static str {
        "CalculatePricesStep"
    }

    fn config(&self) -> StepConfig {
        StepConfig {
            store: false, // Pure-read workflow
            ..Default::default()
        }
    }

    async fn invoke(&self, _ctx: &StepContext, input: Value) -> Result<Value> {
        let price_set_ids: Vec<String> = serde_json::from_value(input["price_set_ids"].clone())
            .map_err(|e| Error::Invalid {
                code: "invalid_price_set_ids",
                message: format!("Failed to parse price_set_ids: {}", e),
                field: Some("price_set_ids".to_string()),
            })?;

        let context: PricingContext =
            serde_json::from_value(input["context"].clone()).map_err(|e| Error::Invalid {
                code: "invalid_pricing_context",
                message: format!("Failed to parse PricingContext: {}", e),
                field: Some("context".to_string()),
            })?;

        let results = self
            .service
            .calculate_prices_batch(&price_set_ids, &context)
            .await?;

        Ok(serde_json::to_value(results).unwrap())
    }

    async fn compensate(&self, _ctx: &StepContext, _input: Value) -> Result<()> {
        Ok(())
    }
}

// ── Builders ───────────────────────────────────────────────────────────────

pub fn build_create_price_set_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreatePriceSetStep::new(service)));
    coordinator
}

pub fn build_update_price_set_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdatePriceSetStep::new(service)));
    coordinator
}

pub fn build_delete_price_set_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeletePriceSetStep::new(service)));
    coordinator
}

pub fn build_create_price_list_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreatePriceListStep::new(service)));
    coordinator
}

pub fn build_update_price_list_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdatePriceListStep::new(service)));
    coordinator
}

pub fn build_delete_price_list_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(DeletePriceListStep::new(service)));
    coordinator
}

pub fn build_batch_price_list_prices_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(BatchPriceListPricesStep::new(service)));
    coordinator
}

pub fn build_create_price_preference_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CreatePricePreferenceStep::new(service)));
    coordinator
}

pub fn build_update_price_preference_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(UpdatePricePreferenceStep::new(service)));
    coordinator
}

pub fn build_calculate_prices_saga(service: Arc<PricingService>) -> SagaCoordinator {
    let mut coordinator = SagaCoordinator::new();
    coordinator.add_step(Arc::new(CalculatePricesStep::new(service)));
    coordinator
}
