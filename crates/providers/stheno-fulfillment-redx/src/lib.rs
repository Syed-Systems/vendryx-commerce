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
            access_token: String::new(),
        }
    }
}

fn required_text<'a>(
    data: Option<&'a serde_json::Value>,
    field: &str,
    label: &str,
) -> Result<&'a str> {
    data.and_then(|value| value.get(field))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::Invalid {
            code: "redx_shipment_field_missing",
            message: format!("RedX {label} is required to create a shipment."),
            field: Some(field.to_string()),
        })
}

fn positive_u64(data: Option<&serde_json::Value>, field: &str) -> Result<u64> {
    data.and_then(|value| value.get(field))
        .and_then(serde_json::Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| Error::Invalid {
            code: "redx_shipment_value_missing",
            message: format!("RedX shipment requires a valid {field}."),
            field: Some(field.to_string()),
        })
}

fn value_to_id(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) if !value.trim().is_empty() => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
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
        let access_token = data
            .and_then(|value| value.get("api_key"))
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&self.config.access_token);
        if access_token.trim().is_empty() {
            return Err(Error::Invalid {
                code: "redx_credentials_missing",
                message: "RedX merchant credentials are not configured.".to_string(),
                field: None,
            });
        }
        let base_url = match data
            .and_then(|value| value.get("sandbox_mode"))
            .and_then(serde_json::Value::as_bool)
        {
            Some(true) => "https://openapi.redx.com.bd/v1.0.0-beta",
            Some(false) => "https://openapi.redx.com.bd/v1.0.0",
            None => self.config.base_url.as_str(),
        };
        let url = format!("{base_url}/parcel");

        let invoice_id = data
            .and_then(|d| {
                d.get("merchant_invoice_id")
                    .or_else(|| d.get("invoice"))
                    .or_else(|| d.get("reference"))
            })
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| Error::Invalid {
                code: "redx_invoice_missing",
                message: "RedX shipment requires a stable merchant invoice reference.".to_string(),
                field: Some("merchant_invoice_id".to_string()),
            })?;
        let customer_name = required_text(data, "customer_name", "customer name")?;
        let customer_phone = required_text(data, "customer_phone", "customer phone")?;
        let delivery_area = required_text(data, "delivery_area", "delivery area")?;
        let delivery_area_id = positive_u64(data, "delivery_area_id")?;
        let customer_address = required_text(data, "customer_address", "customer address")?;
        let parcel_weight = positive_u64(data, "parcel_weight")?;

        let cash_collection = data
            .and_then(|d| d.get("cash_collection_amount").or_else(|| d.get("cod_amount")).or_else(|| d.get("amount_to_collect")))
            .map_or(Ok(0), |value| value.as_u64().ok_or_else(|| Error::Invalid {
                code: "redx_cash_collection_invalid",
                message: "RedX cash collection amount must be a non-negative integer in minor currency units.".to_string(),
                field: Some("cash_collection_amount".to_string()),
            }))?;

        let payload = serde_json::json!({
            "customer_name": customer_name,
            "customer_phone": customer_phone,
            "delivery_area": delivery_area,
            "delivery_area_id": delivery_area_id,
            "customer_address": customer_address,
            "merchant_invoice_id": invoice_id,
            "cash_collection_amount": cash_collection,
            "parcel_weight": parcel_weight,
        });

        let resp = self
            .client
            .post(&url)
            .header("API-ACCESS-TOKEN", format!("Bearer {access_token}"))
            .json(&payload)
            .send()
            .await;

        let response = resp.map_err(|_| Error::Invalid {
            code: "redx_create_request_failed",
            message: "RedX shipment request failed; shipment status is unknown.".to_string(),
            field: None,
        })?;
        if !response.status().is_success() {
            return Err(Error::Invalid {
                code: "redx_create_rejected",
                message: format!(
                    "RedX rejected shipment creation with HTTP {}.",
                    response.status()
                ),
                field: None,
            });
        }
        let body: serde_json::Value = response.json().await.map_err(|_| Error::Invalid {
            code: "redx_create_response_invalid",
            message: "RedX shipment response was invalid; shipment status is unknown.".to_string(),
            field: None,
        })?;
        let tracking_id = body
            .get("tracking_id")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| Error::Invalid {
                code: "redx_tracking_id_missing",
                message: "RedX accepted the request but returned no tracking ID; shipment status is unknown.".to_string(),
                field: None,
            })?;
        let parcel_id = body
            .get("parcel_id")
            .and_then(value_to_id)
            .ok_or_else(|| Error::Invalid {
                code: "redx_parcel_id_missing",
                message: "RedX accepted the request but returned no parcel ID; shipment status is unknown.".to_string(),
                field: None,
            })?;

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
            label_url: Some(format!("{base_url}/labels/{tracking_id}.pdf")),
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
            .header(
                "API-ACCESS-TOKEN",
                format!("Bearer {}", self.config.access_token),
            )
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
            label_url: Some(format!(
                "https://openapi.redx.com.bd/v1.0.0-beta/labels/{ret_id}.pdf"
            )),
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
            .and_then(|v| {
                v.as_str().or_else(|| {
                    v.as_i64()
                        .map(|n| Box::leak(n.to_string().into_boxed_str()) as &str)
                })
            });

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
            .header(
                "API-ACCESS-TOKEN",
                format!("Bearer {}", self.config.access_token),
            )
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
