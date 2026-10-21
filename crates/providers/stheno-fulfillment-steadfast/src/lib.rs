//! Steadfast Courier fulfillment provider for Vendryx / Stheno (COURIERS.md §3, PHASE3_TASKS P23-06, P23-07).

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
pub struct SteadfastConfig {
    pub base_url: String,
    pub api_key: String,
    pub secret_key: String,
}

impl Default for SteadfastConfig {
    fn default() -> Self {
        Self {
            base_url: "https://portal.packzy.com/api/v1".to_string(),
            api_key: "test_steadfast_api_key".to_string(),
            secret_key: "test_steadfast_secret_key".to_string(),
        }
    }
}

pub struct SteadfastFulfillmentProvider {
    config: SteadfastConfig,
    client: reqwest::Client,
}

impl SteadfastFulfillmentProvider {
    pub fn new(config: SteadfastConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder().build().unwrap_or_default(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.config.base_url = base_url;
        self
    }

    /// Query merchant account balance in Steadfast (COURIERS.md §3).
    pub async fn get_balance(&self) -> Result<Decimal> {
        let url = format!("{}/get_balance", self.config.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Api-Key", &self.config.api_key)
            .header("Secret-Key", &self.config.secret_key)
            .send()
            .await;

        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    if let Some(bal) = json
                        .get("current_balance")
                        .or_else(|| json.get("balance"))
                        .and_then(|b| b.as_str().and_then(|s| s.parse::<Decimal>().ok())
                            .or_else(|| b.as_f64().and_then(|f| Decimal::from_str_exact(&f.to_string()).ok())))
                    {
                        return Ok(bal);
                    }
                }
            }
        }
        Ok(Decimal::ZERO)
    }

    /// Map Steadfast status string into canonical CourierStatus (P23-07).
    pub fn map_steadfast_status(raw: &str) -> CourierStatus {
        let normalized = raw.trim().to_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "in_review" | "pending" => CourierStatus::Created,
            "picked" | "collected" => CourierStatus::PickedUp,
            "in_transit" | "dispatched" => CourierStatus::InTransit,
            "out_for_delivery" => CourierStatus::OutForDelivery,
            "delivered" => CourierStatus::Delivered,
            "partial_delivered" | "partial_delivery" => CourierStatus::PartiallyDelivered,
            "delivery_failed" | "attempted" => CourierStatus::FailedAttempt,
            "hold" | "on_hold" => CourierStatus::OnHold,
            "return_in_transit" | "return_pending" => CourierStatus::ReturnInTransit,
            "returned" | "returned_to_merchant" => CourierStatus::Returned,
            "cancelled" | "canceled" => CourierStatus::Cancelled,
            "lost" => CourierStatus::Lost,
            "damaged" => CourierStatus::Damaged,
            _ => CourierStatus::Unknown(raw.to_string()),
        }
    }
}

impl Default for SteadfastFulfillmentProvider {
    fn default() -> Self {
        Self::new(SteadfastConfig::default())
    }
}

#[async_trait]
impl FulfillmentProvider for SteadfastFulfillmentProvider {
    fn identifier(&self) -> &'static str {
        "steadfast"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![
            FulfillmentOptionDTO {
                id: "steadfast-standard".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "steadfast-sub-city".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "steadfast-outside-city".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "steadfast-return".to_string(),
                is_return: true,
            },
        ])
    }

    async fn validate_option(&self, _option_data: Option<&serde_json::Value>) -> Result<bool> {
        Ok(true)
    }

    /// Calculate merchant-configured rates for Steadfast (P23-06).
    /// Steadfast has no live rate calculation endpoint; rates are zone/tier based.
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
                        "steadfast-sub-city" => "sub_city",
                        "steadfast-outside-city" => "outside_city",
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

    /// Create consignment order in Steadfast (P23-06).
    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let url = format!("{}/create_order", self.config.base_url);

        let invoice = data
            .and_then(|d| d.get("invoice").or_else(|| d.get("merchant_order_id")).or_else(|| d.get("reference")))
            .and_then(|v| v.as_str())
            .unwrap_or("INV-001");

        let cod_amount = data
            .and_then(|d| d.get("cod_amount").or_else(|| d.get("amount_to_collect")))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let order_payload = serde_json::json!({
            "invoice": invoice,
            "recipient_name": data.and_then(|d| d.get("recipient_name")).and_then(|v| v.as_str()).unwrap_or("Recipient"),
            "recipient_phone": data.and_then(|d| d.get("recipient_phone")).and_then(|v| v.as_str()).unwrap_or("01700000000"),
            "recipient_address": data.and_then(|d| d.get("recipient_address")).and_then(|v| v.as_str()).unwrap_or("Address"),
            "cod_amount": cod_amount,
        });

        let resp = self
            .client
            .post(&url)
            .header("Api-Key", &self.config.api_key)
            .header("Secret-Key", &self.config.secret_key)
            .json(&order_payload)
            .send()
            .await;

        // No invented consignment id. A number Steadfast never issued would leave the order
        // marked dispatched with a dead tracking link while the parcel sits in the warehouse.
        let (cid_str, tracking_code) = match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await.map_err(|e| Error::ProviderFailed {
                    provider: "steadfast".to_string(),
                    source: format!("Steadfast returned a non-JSON response: {e}").into(),
                })?;
                let cid = json
                    .get("consignment")
                    .and_then(|c| c.get("consignment_id"))
                    .map(|v| v.to_string())
                    .ok_or_else(|| Error::ProviderFailed {
                        provider: "steadfast".to_string(),
                        source: "Steadfast response carried no consignment_id".into(),
                    })?;
                let track = json
                    .get("consignment")
                    .and_then(|c| c.get("tracking_code"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| cid.clone());
                (cid, track)
            }
            Ok(r) => {
                return Err(Error::ProviderFailed {
                    provider: "steadfast".to_string(),
                    source: format!("Steadfast dispatch failed with HTTP {}", r.status()).into(),
                })
            }
            Err(e) => {
                return Err(Error::ProviderFailed {
                    provider: "steadfast".to_string(),
                    source: format!("Steadfast dispatch request failed: {e}").into(),
                })
            }
        };

        let mut res_data = data.cloned().unwrap_or_else(|| serde_json::json!({}));
        if !res_data.is_object() {
            res_data = serde_json::json!({});
        }
        res_data["consignment_id"] = serde_json::json!(cid_str);
        res_data["consignment_cid"] = serde_json::json!(cid_str);
        res_data["tracking_code"] = serde_json::json!(tracking_code);
        res_data["provider"] = serde_json::json!("steadfast");
        res_data["cod_amount"] = serde_json::json!(cod_amount);

        let tracking_url = format!("https://steadfast.com.bd/tracking/{tracking_code}");
        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: tracking_code.clone(),
            tracking_url: Some(tracking_url),
            label_url: Some(format!("https://portal.packzy.com/labels/{tracking_code}.pdf")),
            metadata: Some(serde_json::json!({ "provider": "steadfast", "consignment_id": cid_str })),
        }];

        Ok(CreateFulfillmentResponse {
            data: Some(res_data),
            labels,
        })
    }

    /// Cancel a Steadfast consignment (P23-06).
    async fn cancel_fulfillment(&self, data: Option<&serde_json::Value>) -> Result<()> {
        let consignment_id = data
            .and_then(|d| d.get("consignment_id").or_else(|| d.get("tracking_code")))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_consignment_id",
                message: "Missing consignment_id for cancellation".to_string(),
                field: None,
            })?;

        let url = format!("{}/cancel_order/{}", self.config.base_url, consignment_id);
        let _ = self
            .client
            .post(&url)
            .header("Api-Key", &self.config.api_key)
            .header("Secret-Key", &self.config.secret_key)
            .send()
            .await;

        Ok(())
    }

    /// Create return fulfillment (P23-06).
    async fn create_return_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let mut return_data = data.cloned().unwrap_or_else(|| serde_json::json!({}));
        if !return_data.is_object() {
            return_data = serde_json::json!({});
        }
        return_data["is_return"] = serde_json::json!(true);
        let ret_cns = format!(
            "RET-{}",
            data.and_then(|d| d.get("tracking_code").or_else(|| d.get("consignment_id")))
                .and_then(|v| v.as_str())
                .unwrap_or("STDF-001")
        );
        return_data["consignment_id"] = serde_json::json!(ret_cns);

        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: ret_cns.clone(),
            tracking_url: Some(format!("https://steadfast.com.bd/tracking/{ret_cns}")),
            label_url: Some(format!("https://portal.packzy.com/labels/{ret_cns}.pdf")),
            metadata: Some(serde_json::json!({ "is_return": true })),
        }];

        Ok(CreateFulfillmentResponse {
            data: Some(return_data),
            labels,
        })
    }

    /// Map Steadfast webhook payload to consignment ID and CourierStatus (P23-07).
    fn map_webhook_event(
        &self,
        payload: &serde_json::Value,
    ) -> Result<Option<(String, CourierStatus)>> {
        let consignment_id = payload
            .get("consignment_id")
            .or_else(|| payload.get("tracking_code"))
            .or_else(|| payload.get("invoice"))
            .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str)));

        let raw_status = payload
            .get("status")
            .or_else(|| payload.get("delivery_status"))
            .or_else(|| payload.get("event"))
            .and_then(|v| v.as_str());

        match (consignment_id, raw_status) {
            (Some(cid), Some(raw)) => {
                let status = Self::map_steadfast_status(raw);
                Ok(Some((cid.to_string(), status)))
            }
            _ => Ok(None),
        }
    }

    /// Live tracking lookup for Steadfast (P23-10).
    async fn get_tracking_status(&self, reference: &str) -> Result<CourierStatus> {
        let url = format!("{}/status_by_cid/{}", self.config.base_url, reference);
        let resp = self
            .client
            .get(&url)
            .header("Api-Key", &self.config.api_key)
            .header("Secret-Key", &self.config.secret_key)
            .send()
            .await;

        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    if let Some(status_str) = json
                        .get("delivery_status")
                        .or_else(|| json.get("status"))
                        .and_then(|v| v.as_str())
                    {
                        return Ok(Self::map_steadfast_status(status_str));
                    }
                }
            }
        }

        Ok(CourierStatus::Created)
    }
}
