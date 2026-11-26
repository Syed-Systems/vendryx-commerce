//! Manual fulfillment provider implementation for Stheno (TASKS P5-11, P5-12).

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use stheno_fulfillment::{
    CalculatePriceContext, CreateFulfillmentItemInput, CreateFulfillmentLabelInput,
    CreateFulfillmentResponse, FulfillmentOptionDTO, FulfillmentProvider,
};
use stheno_types::Result;

pub struct ManualFulfillmentProvider;

impl ManualFulfillmentProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ManualFulfillmentProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FulfillmentProvider for ManualFulfillmentProvider {
    fn identifier(&self) -> &'static str {
        "manual"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![
            FulfillmentOptionDTO {
                id: "manual-standard".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "manual-express".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "manual-return".to_string(),
                is_return: true,
            },
        ])
    }

    async fn validate_option(&self, _option_data: Option<&serde_json::Value>) -> Result<bool> {
        Ok(true)
    }

    async fn calculate_price(
        &self,
        option_data: Option<&serde_json::Value>,
        _data: Option<&serde_json::Value>,
        context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        // Base rate from option_data or fallback default
        let base_rate = option_data
            .and_then(|d| {
                d.get("rate")
                    .and_then(|r| r.as_str())
                    .and_then(|s| s.parse::<Decimal>().ok())
            })
            .unwrap_or(dec!(10.00));

        // Weight modifier or item multiplier if context provided
        if let Some(ctx) = context {
            if let Some(weight) = ctx.total_weight {
                let weight_charge = weight * dec!(2.50);
                return Ok(base_rate + weight_charge);
            }
            if ctx.items_count > 1 {
                let extra_items = Decimal::from((ctx.items_count - 1) as i64) * dec!(1.50);
                return Ok(base_rate + extra_items);
            }
        }

        Ok(base_rate)
    }

    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let tracking_number = data
            .and_then(|d| d.get("tracking_number").and_then(|t| t.as_str()))
            .unwrap_or("MANUAL-TRK-99001");

        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: tracking_number.to_string(),
            tracking_url: Some(format!("https://tracking.example.com/{tracking_number}")),
            label_url: Some(format!("https://labels.example.com/{tracking_number}.pdf")),
            metadata: Some(serde_json::json!({ "item_count": items.len() })),
        }];

        Ok(CreateFulfillmentResponse {
            data: data.cloned(),
            labels,
        })
    }

    async fn cancel_fulfillment(&self, _data: Option<&serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn create_return_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let tracking_number = data
            .and_then(|d| d.get("tracking_number").and_then(|t| t.as_str()))
            .unwrap_or("MANUAL-RET-88001");

        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: tracking_number.to_string(),
            tracking_url: Some(format!("https://tracking.example.com/return/{tracking_number}")),
            label_url: Some(format!("https://labels.example.com/return/{tracking_number}.pdf")),
            metadata: None,
        }];

        Ok(CreateFulfillmentResponse {
            data: data.cloned(),
            labels,
        })
    }
}
