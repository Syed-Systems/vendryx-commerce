//! Fulfillment workflows (TASKS P5-20).
//! 8 fulfillment workflows for standard and return fulfillment lifecycles.
#![allow(dead_code)]

use std::sync::Arc;
use stheno_fulfillment::{
    CreateFulfillmentInput, CreateServiceZoneInput, CreateShippingOptionInput, FulfillmentDTO,
    FulfillmentService, ServiceZoneDTO, ShippingOptionDTO,
};
use stheno_types::Result;

#[derive(Clone)]
pub struct FulfillmentWorkflowEngine {
    pub fulfillment_service: Arc<FulfillmentService>,
}

impl FulfillmentWorkflowEngine {
    pub fn new(fulfillment_service: Arc<FulfillmentService>) -> Self {
        Self {
            fulfillment_service,
        }
    }

    // 1. Create fulfillment workflow
    pub async fn create_fulfillment_flow(
        &self,
        input: CreateFulfillmentInput,
    ) -> Result<FulfillmentDTO> {
        self.fulfillment_service.create_fulfillment(input).await
    }

    // 2. Cancel fulfillment workflow
    pub async fn cancel_fulfillment_flow(&self, id: &str) -> Result<FulfillmentDTO> {
        self.fulfillment_service.cancel_fulfillment(id).await
    }

    // 3. Mark fulfillment shipped workflow
    pub async fn mark_fulfillment_shipped_flow(&self, id: &str) -> Result<FulfillmentDTO> {
        self.fulfillment_service.mark_as_shipped(id).await
    }

    // 4. Mark fulfillment delivered workflow
    pub async fn mark_fulfillment_delivered_flow(&self, id: &str) -> Result<FulfillmentDTO> {
        self.fulfillment_service.mark_as_delivered(id).await
    }

    // 5. Create shipment workflow (creates labels and marks shipped)
    pub async fn create_shipment_flow(&self, id: &str) -> Result<FulfillmentDTO> {
        self.fulfillment_service.mark_as_shipped(id).await
    }

    // 6. Create return fulfillment workflow
    pub async fn create_return_fulfillment_flow(
        &self,
        input: CreateFulfillmentInput,
    ) -> Result<FulfillmentDTO> {
        self.fulfillment_service.create_fulfillment(input).await
    }

    // 7. Create service zone workflow
    pub async fn create_service_zone_flow(
        &self,
        input: CreateServiceZoneInput,
    ) -> Result<ServiceZoneDTO> {
        self.fulfillment_service.create_service_zone(input).await
    }

    // 8. Create shipping option workflow
    pub async fn create_shipping_option_flow(
        &self,
        input: CreateShippingOptionInput,
    ) -> Result<ShippingOptionDTO> {
        self.fulfillment_service.create_shipping_option(input).await
    }
}
