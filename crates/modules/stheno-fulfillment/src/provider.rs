//! `FulfillmentProvider` trait definition (DATA_MODEL §2.6, TASKS P5-11).

use async_trait::async_trait;
use rust_decimal::Decimal;
use stheno_types::Result;

use crate::dto::{
    CalculatePriceContext, CreateFulfillmentItemInput, CreateFulfillmentResponse,
    FulfillmentOptionDTO,
};
use crate::status::CourierStatus;

#[async_trait]
pub trait FulfillmentProvider: Send + Sync {
    /// Unique provider identifier (e.g., "manual", "shipstation", "easypost", "pathao").
    fn identifier(&self) -> &'static str;

    /// Returns list of supported fulfillment options.
    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>>;

    /// Validates option data.
    async fn validate_option(&self, option_data: Option<&serde_json::Value>) -> Result<bool>;

    /// Calculates real-time shipping rate/price through the provider (P5-12).
    async fn calculate_price(
        &self,
        option_data: Option<&serde_json::Value>,
        data: Option<&serde_json::Value>,
        context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal>;

    /// Creates fulfillment with tracking numbers and shipping labels (P5-13).
    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        items: &[CreateFulfillmentItemInput],
        order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse>;

    /// Cancels an existing fulfillment.
    async fn cancel_fulfillment(&self, data: Option<&serde_json::Value>) -> Result<()>;

    /// Creates return fulfillment label and details.
    async fn create_return_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse>;

    /// Map a courier webhook payload to a consignment reference and CourierStatus (P23-05).
    fn map_webhook_event(
        &self,
        _payload: &serde_json::Value,
    ) -> Result<Option<(String, CourierStatus)>> {
        Ok(None)
    }

    /// Query live tracking status by consignment/tracking reference (P23-10).
    async fn get_tracking_status(&self, _reference: &str) -> Result<CourierStatus> {
        Ok(CourierStatus::Created)
    }
}
