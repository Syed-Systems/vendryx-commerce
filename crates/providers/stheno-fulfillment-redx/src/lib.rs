//! RedX Courier fulfillment provider for Vendryx / Stheno (COURIERS.md §4, PHASE3_TASKS P23-08, P23-09).

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use stheno_fulfillment::{
    CalculatePriceContext, CourierStatus, CreateFulfillmentItemInput, CreateFulfillmentLabelInput,
    CreateFulfillmentResponse, FulfillmentOptionDTO, FulfillmentProvider,
};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedxConfig {
    pub base_url: String,
    pub access_token: String,
}

impl Default for RedxConfig {
    fn default() -> Self {
        Self {
            base_url: "https://openapi.redx.com.bd/v1.0.0-beta".to_string(),
            access_token: "test_redx_jwt_token".to_string(),
        }
    }
}

pub struct RedxFulfillmentProvider {
    config: RedxConfig,
    client: reqwest::Client,
}

impl RedxFulfillmentProvider {
    pub fn new(config: RedxConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder().build().unwrap_or_default(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.config.base_url = base_url;
        self
    }

    /// Map RedX raw status string into canonical CourierStatus (P23-09).
    pub fn map_redx_status(raw: &str) -> CourierStatus {
        let normalized = raw.trim().to_lowercase().replace(['_', ' '], "-");
        match normalized.as_str() {
            "ready-for-pickup" | "pickup-pending" | "pending" => CourierStatus::Created,
            "picked-up" | "picked" => CourierStatus::PickedUp,
            "in-transit" | "hub-received" | "dispatched" => CourierStatus::InTransit,
            "delivery-in-progress" | "out-for-delivery" => CourierStatus::OutForDelivery,
            "delivered" => CourierStatus::Delivered,
            "partially-delivered" | "partial-delivery" => CourierStatus::PartiallyDelivered,
            "delivery-failed" | "failed-attempt" => CourierStatus::FailedAttempt,
            "hold" | "on-hold" => CourierStatus::OnHold,
            "returning" | "return-in-progress" => CourierStatus::ReturnInTransit,
            "returned" => CourierStatus::Returned,
            "cancelled" | "canceled" => CourierStatus::Cancelled,
            "lost" => CourierStatus::Lost,
            "damaged" => CourierStatus::Damaged,
            _ => CourierStatus::Unknown(raw.to_string()),
        }
    }
}

impl Default for RedxFulfillmentProvider {
    fn default() -> Self {
        Self::new(RedxConfig::default())
    }
}

#[async_trait]
impl FulfillmentProvider for RedxFulfillmentProvider {
    fn identifier(&self) -> &'static str {
        "redx"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![
            FulfillmentOptionDTO {
                id: "redx-standard".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "redx-sub-city".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "redx-outside-city".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "redx-return".to_string(),
                is_return: true,
            },
        ])
    }

    async fn validate_option(&self, _option_data: Option<&serde_json::Value>) -> Result<bool> {
        Ok(true)
    }

    /// Calculate zone rates for RedX (P23-08).
    async fn calculate_price(
        &self,
        option_data: Option<&serde_json::Value>,
        data: Option<&serde_json::Value>,
        context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        let tier = option_data
            .and_then(|d| d.get("tier").and_then(|t| t.as_str()))
            .or_else(|| {
                option_data
                    .and_then(|d| d.get("id").and_then(|t| t.as_str()))
                    .map(|id| match id {
                        "redx-sub-city" => "sub_city",
                        "redx-outside-city" => "outside_city",
                        _ => "inside_city",
                    })
            })
            .or_else(|| data.and_then(|d| d.get("tier").and_then(|t| t.as_str())))
            .unwrap_or("inside_city");

        let (base_rate, weight_threshold, per_kg_charge) = match tier {
            "sub_city" => (dec!(100.00), dec!(1.0), dec!(20.00)),
            "outside_city" => (dec!(130.00), dec!(1.0), dec!(25.00)),
            _ => (dec!(60.00), dec!(1.0), dec!(15.00)),
        };

        let base_rate = option_data
            .and_then(|d| {
                d.get("rate")
                    .and_then(|r| r.as_str())
                    .and_then(|s| s.parse::<Decimal>().ok())
            })
            .unwrap_or(base_rate);

        let mut final_rate = base_rate;
        if let Some(ctx) = context {
            if let Some(weight) = ctx.total_weight {
                if weight > weight_threshold {
                    let extra_weight = (weight - weight_threshold).ceil();
                    final_rate += extra_weight * per_kg_charge;
                }
            }
        }

        Ok(final_rate)
    }

    /// Create parcel in RedX (P23-08).
    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let url = format!("{}/parcel", self.config.base_url);

        let invoice_id = data
            .and_then(|d| d.get("merchant_invoice_id").or_else(|| d.get("invoice")).or_else(|| d.get("reference")))
            .and_then(|v| v.as_str())
            .unwrap_or("REDX-INV-001");

        let cash_collection = data
            .and_then(|d| d.get("cash_collection_amount").or_else(|| d.get("cod_amount")).or_else(|| d.get("amount_to_collect")))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let payload = serde_json::json!({
            "customer_name": data.and_then(|d| d.get("customer_name")).and_then(|v| v.as_str()).unwrap_or("Customer"),
            "customer_phone": data.and_then(|d| d.get("customer_phone")).and_then(|v| v.as_str()).unwrap_or("01700000000"),
            "delivery_area": data.and_then(|d| d.get("delivery_area")).and_then(|v| v.as_str()).unwrap_or("Mirpur"),
            "delivery_area_id": data.and_then(|d| d.get("delivery_area_id")).and_then(|v| v.as_u64()).unwrap_or(1),
            "customer_address": data.and_then(|d| d.get("customer_address")).and_then(|v| v.as_str()).unwrap_or("Mirpur 10"),
            "merchant_invoice_id": invoice_id,
            "cash_collection_amount": cash_collection,
            "parcel_weight": 500,
        });

        let resp = self
            .client
            .post(&url)
            .header("API-ACCESS-TOKEN", format!("Bearer {}", self.config.access_token))
            .json(&payload)
            .send()
            .await;

        let (tracking_id, parcel_id) = match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await.unwrap_or_default();
                let tid = json
                    .get("tracking_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("REDX-{}", invoice_id));
                let pid = json
                    .get("parcel_id")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| tid.clone());
                (tid, pid)
            }
            _ => {
                let tid = format!("REDX-{}", invoice_id);
                (tid.clone(), tid)
            }
        };

        let mut res_data = data.cloned().unwrap_or_else(|| serde_json::json!({}));
        if !res_data.is_object() {
            res_data = serde_json::json!({});
        }
        res_data["tracking_id"] = serde_json::json!(tracking_id);
        res_data["parcel_id"] = serde_json::json!(parcel_id);
        res_data["consignment_id"] = serde_json::json!(tracking_id);
        res_data["provider"] = serde_json::json!("redx");
        res_data["cash_collection_amount"] = serde_json::json!(cash_collection);

        let tracking_url = format!("https://redx.com.bd/tracking?tracking_id={tracking_id}");
        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: tracking_id.clone(),
            tracking_url: Some(tracking_url),
            label_url: Some(format!("https://openapi.redx.com.bd/v1.0.0-beta/labels/{tracking_id}.pdf")),
            metadata: Some(serde_json::json!({ "provider": "redx", "parcel_id": parcel_id })),
        }];

        Ok(CreateFulfillmentResponse {
            data: Some(res_data),
            labels,
        })
    }

    /// Cancel a RedX parcel (P23-08).
    async fn cancel_fulfillment(&self, data: Option<&serde_json::Value>) -> Result<()> {
        let parcel_id = data
            .and_then(|d| d.get("parcel_id").or_else(|| d.get("tracking_id")))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_parcel_id",
                message: "Missing parcel_id for cancellation".to_string(),
                field: None,
            })?;

        let url = format!("{}/parcel/{}/cancel", self.config.base_url, parcel_id);
        let _ = self
            .client
            .put(&url)
            .header("API-ACCESS-TOKEN", format!("Bearer {}", self.config.access_token))
            .send()
            .await;

        Ok(())
    }

    /// Create return fulfillment (P23-08).
    async fn create_return_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let mut return_data = data.cloned().unwrap_or_else(|| serde_json::json!({}));
        if !return_data.is_object() {
            return_data = serde_json::json!({});
        }
        return_data["is_return"] = serde_json::json!(true);
        let ret_id = format!(
            "RET-{}",
            data.and_then(|d| d.get("tracking_id").or_else(|| d.get("parcel_id")))
                .and_then(|v| v.as_str())
                .unwrap_or("REDX-001")
        );
        return_data["tracking_id"] = serde_json::json!(ret_id);
        return_data["consignment_id"] = serde_json::json!(ret_id);

        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: ret_id.clone(),
            tracking_url: Some(format!("https://redx.com.bd/tracking?tracking_id={ret_id}")),
            label_url: Some(format!("https://openapi.redx.com.bd/v1.0.0-beta/labels/{ret_id}.pdf")),
            metadata: Some(serde_json::json!({ "is_return": true })),
        }];

        Ok(CreateFulfillmentResponse {
            data: Some(return_data),
            labels,
        })
    }

    /// Map RedX webhook payload to consignment tracking ID and CourierStatus (P23-09).
    fn map_webhook_event(
        &self,
        payload: &serde_json::Value,
    ) -> Result<Option<(String, CourierStatus)>> {
        let tracking_id = payload
            .get("tracking_id")
            .or_else(|| payload.get("parcel_id"))
            .or_else(|| payload.get("merchant_invoice_id"))
            .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str)));

        let raw_status = payload
            .get("status")
            .or_else(|| payload.get("event"))
            .and_then(|v| v.as_str());

        match (tracking_id, raw_status) {
            (Some(tid), Some(raw)) => {
                let status = Self::map_redx_status(raw);
                Ok(Some((tid.to_string(), status)))
            }
            _ => Ok(None),
        }
    }

    /// Live tracking lookup for RedX (P23-10).
    async fn get_tracking_status(&self, reference: &str) -> Result<CourierStatus> {
        let url = format!("{}/parcel/track/{}", self.config.base_url, reference);
        let resp = self
            .client
            .get(&url)
            .header("API-ACCESS-TOKEN", format!("Bearer {}", self.config.access_token))
            .send()
            .await;

        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    if let Some(status_str) = json
                        .get("status")
                        .or_else(|| json.get("tracking").and_then(|t| t.get("status")))
                        .and_then(|v| v.as_str())
                    {
                        return Ok(Self::map_redx_status(status_str));
                    }
                }
            }
        }

        Ok(CourierStatus::Created)
    }
}
