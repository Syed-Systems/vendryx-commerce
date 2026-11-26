//! Pathao Courier fulfillment provider for Vendryx / Stheno (COURIERS.md §2, PHASE3_TASKS P23-02..P23-05).

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use stheno_fulfillment::{
    CalculatePriceContext, CourierStatus, CreateFulfillmentItemInput, CreateFulfillmentLabelInput,
    CreateFulfillmentResponse, FulfillmentOptionDTO, FulfillmentProvider,
};
use stheno_types::{Error, Result};
use tokio::sync::RwLock;

/// Configuration for Pathao Courier provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathaoConfig {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub store_id: Option<u32>,
    pub sandbox: bool,
}

impl Default for PathaoConfig {
    fn default() -> Self {
        Self {
            base_url: "https://courier-api-sandbox.pathao.com".to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            username: String::new(),
            password: String::new(),
            store_id: None,
            sandbox: true,
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
            code: "pathao_shipment_field_missing",
            message: format!("{label} is required to create a Pathao shipment."),
            field: Some(field.to_string()),
        })
}

fn required_positive_u64(
    data: Option<&serde_json::Value>,
    field: &str,
    label: &str,
) -> Result<u64> {
    data.and_then(|value| value.get(field))
        .and_then(serde_json::Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| Error::Invalid {
            code: "pathao_destination_mapping_missing",
            message: format!("{label} is required to create a Pathao shipment."),
            field: Some(field.to_string()),
        })
}

fn optional_nonnegative_u64(data: Option<&serde_json::Value>, field: &str) -> Result<u64> {
    match data.and_then(|value| value.get(field)) {
        None | Some(serde_json::Value::Null) => Ok(0),
        Some(value) => value.as_u64().ok_or_else(|| Error::Invalid {
            code: "pathao_collection_amount_invalid",
            message:
                "Pathao amount to collect must be a non-negative integer in minor currency units."
                    .to_string(),
            field: Some(field.to_string()),
        }),
    }
}

fn location_rows(body: &serde_json::Value) -> Result<&Vec<serde_json::Value>> {
    body.get("data")
        .and_then(|data| data.get("data").or(Some(data)))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::Invalid {
            code: "pathao_location_response_invalid",
            message: "Pathao returned an invalid destination list.".to_string(),
            field: None,
        })
}

fn positive_u32(value: Option<&serde_json::Value>, field: &str) -> Result<u32> {
    value
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| Error::Invalid {
            code: "pathao_location_id_invalid",
            message: format!("Pathao returned an invalid {field}."),
            field: Some(field.to_string()),
        })
}

fn nonempty_text<'a>(value: Option<&'a serde_json::Value>, field: &str) -> Result<&'a str> {
    value
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Error::Invalid {
            code: "pathao_location_name_invalid",
            message: format!("Pathao returned an invalid {field}."),
            field: Some(field.to_string()),
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathaoCity {
    pub city_id: u32,
    pub city_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathaoZone {
    pub zone_id: u32,
    pub zone_name: String,
    pub city_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathaoArea {
    pub area_id: u32,
    pub area_name: String,
    pub zone_id: u32,
}

pub struct PathaoFulfillmentProvider {
    config: PathaoConfig,
    client: reqwest::Client,
    access_token: Arc<RwLock<Option<String>>>,
}

impl PathaoFulfillmentProvider {
    pub fn new(config: PathaoConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder().build().unwrap_or_default(),
            access_token: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.config.base_url = base_url;
        self
    }

    /// Obtain or return cached bearer token.
    pub async fn get_token(&self) -> Result<String> {
        if self.config.client_id.trim().is_empty()
            || self.config.client_secret.trim().is_empty()
            || self.config.username.trim().is_empty()
            || self.config.password.trim().is_empty()
        {
            return Err(Error::Invalid {
                code: "pathao_credentials_missing",
                message: "Pathao merchant credentials are not configured.".to_string(),
                field: None,
            });
        }
        {
            let guard = self.access_token.read().await;
            if let Some(token) = &*guard {
                return Ok(token.clone());
            }
        }

        let url = format!("{}/aladdin/api/v1/issue-token", self.config.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "client_id": self.config.client_id,
                "client_secret": self.config.client_secret,
                "grant_type": "password",
                "username": self.config.username,
                "password": self.config.password,
            }))
            .send()
            .await;

        let response = resp.map_err(|_| Error::Invalid {
            code: "pathao_auth_request_failed",
            message: "Pathao authentication request failed.".to_string(),
            field: None,
        })?;
        if !response.status().is_success() {
            return Err(Error::Invalid {
                code: "pathao_auth_rejected",
                message: "Pathao rejected the configured merchant credentials.".to_string(),
                field: None,
            });
        }
        let body: serde_json::Value = response.json().await.map_err(|_| Error::Invalid {
            code: "pathao_auth_response_invalid",
            message: "Pathao returned an invalid authentication response.".to_string(),
            field: None,
        })?;
        let token = body
            .get("access_token")
            .and_then(|v| v.as_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::Invalid {
                code: "pathao_auth_token_missing",
                message: "Pathao authentication response did not contain an access token."
                    .to_string(),
                field: None,
            })?
            .to_string();
        let mut guard = self.access_token.write().await;
        *guard = Some(token.clone());
        Ok(token)
    }

    /// List serviceable cities from Pathao (P23-02).
    pub async fn list_cities(&self) -> Result<Vec<PathaoCity>> {
        let token = self.get_token().await?;
        let url = format!(
            "{}/aladdin/api/v1/countries/1/city-list",
            self.config.base_url
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| Error::Invalid {
                code: "pathao_city_list_unavailable",
                message: "Pathao destination cities could not be loaded.".to_string(),
                field: None,
            })?;
        if !resp.status().is_success() {
            return Err(Error::Invalid {
                code: "pathao_city_list_rejected",
                message: format!(
                    "Pathao destination city lookup failed with HTTP {}.",
                    resp.status()
                ),
                field: None,
            });
        }
        let body: serde_json::Value = resp.json().await.map_err(|_| Error::Invalid {
            code: "pathao_city_list_invalid",
            message: "Pathao returned an invalid destination city response.".to_string(),
            field: None,
        })?;
        location_rows(&body)?
            .iter()
            .map(|city| {
                Ok(PathaoCity {
                    city_id: positive_u32(city.get("city_id"), "city_id")?,
                    city_name: nonempty_text(city.get("city_name"), "city_name")?.to_string(),
                })
            })
            .collect()
    }

    /// List zones for a city from Pathao (P23-02).
    pub async fn list_zones(&self, city_id: u32) -> Result<Vec<PathaoZone>> {
        let token = self.get_token().await?;
        let url = format!(
            "{}/aladdin/api/v1/cities/{city_id}/zone-list",
            self.config.base_url
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| Error::Invalid {
                code: "pathao_zone_list_unavailable",
                message: "Pathao destination zones could not be loaded.".to_string(),
                field: None,
            })?;
        if !resp.status().is_success() {
            return Err(Error::Invalid {
                code: "pathao_zone_list_rejected",
                message: format!(
                    "Pathao destination zone lookup failed with HTTP {}.",
                    resp.status()
                ),
                field: None,
            });
        }
        let body: serde_json::Value = resp.json().await.map_err(|_| Error::Invalid {
            code: "pathao_zone_list_invalid",
            message: "Pathao returned an invalid destination zone response.".to_string(),
            field: None,
        })?;
        location_rows(&body)?
            .iter()
            .map(|zone| {
                let response_city_id = positive_u32(zone.get("city_id"), "city_id")?;
                if response_city_id != city_id {
                    return Err(Error::Invalid {
                        code: "pathao_zone_city_mismatch",
                        message: "Pathao returned a zone for a different city.".to_string(),
                        field: Some("city_id".to_string()),
                    });
                }
                Ok(PathaoZone {
                    zone_id: positive_u32(zone.get("zone_id"), "zone_id")?,
                    zone_name: nonempty_text(zone.get("zone_name"), "zone_name")?.to_string(),
                    city_id,
                })
            })
            .collect()
    }

    /// List areas for a zone from Pathao (P23-02).
    pub async fn list_areas(&self, zone_id: u32) -> Result<Vec<PathaoArea>> {
        let token = self.get_token().await?;
        let url = format!(
            "{}/aladdin/api/v1/zones/{zone_id}/area-list",
            self.config.base_url
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|_| Error::Invalid {
                code: "pathao_area_list_unavailable",
                message: "Pathao destination areas could not be loaded.".to_string(),
                field: None,
            })?;
        if !resp.status().is_success() {
            return Err(Error::Invalid {
                code: "pathao_area_list_rejected",
                message: format!(
                    "Pathao destination area lookup failed with HTTP {}.",
                    resp.status()
                ),
                field: None,
            });
        }
        let body: serde_json::Value = resp.json().await.map_err(|_| Error::Invalid {
            code: "pathao_area_list_invalid",
            message: "Pathao returned an invalid destination area response.".to_string(),
            field: None,
        })?;
        location_rows(&body)?
            .iter()
            .map(|area| {
                let response_zone_id = positive_u32(area.get("zone_id"), "zone_id")?;
                if response_zone_id != zone_id {
                    return Err(Error::Invalid {
                        code: "pathao_area_zone_mismatch",
                        message: "Pathao returned an area for a different zone.".to_string(),
                        field: Some("zone_id".to_string()),
                    });
                }
                Ok(PathaoArea {
                    area_id: positive_u32(area.get("area_id"), "area_id")?,
                    area_name: nonempty_text(area.get("area_name"), "area_name")?.to_string(),
                    zone_id,
                })
            })
            .collect()
    }

    /// Validate if an area is serviceable (P23-02).
    pub async fn is_serviceable_area(
        &self,
        city_id: u32,
        zone_id: u32,
        area_id: Option<u32>,
    ) -> Result<bool> {
        let cities = self.list_cities().await?;
        if !cities.iter().any(|c| c.city_id == city_id) {
            return Ok(false);
        }
        let zones = self.list_zones(city_id).await?;
        if !zones.iter().any(|zone| zone.zone_id == zone_id) {
            return Ok(false);
        }
        match area_id {
            Some(area_id) => Ok(self
                .list_areas(zone_id)
                .await?
                .iter()
                .any(|area| area.area_id == area_id)),
            None => Ok(true),
        }
    }

    /// Map raw status from Pathao into canonical CourierStatus (P23-05).
    pub fn map_pathao_status(raw: &str) -> CourierStatus {
        let normalized = raw.trim().to_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "order_created" | "pending" | "pickup_requested" | "pickup_pending" => {
                CourierStatus::Created
            }
            "assigned_for_pickup" | "pickup_in_progress" | "picked" | "picked_up" => {
                CourierStatus::PickedUp
            }
            "in_transit" | "received_at_hub" | "dispatched" => CourierStatus::InTransit,
            "out_for_delivery" => CourierStatus::OutForDelivery,
            "delivered" | "delivery_completed" => CourierStatus::Delivered,
            "partial_delivery" | "partially_delivered" => CourierStatus::PartiallyDelivered,
            "delivery_failed" | "customer_unavailable" => CourierStatus::FailedAttempt,
            "hold" | "on_hold" | "delayed" => CourierStatus::OnHold,
            "return_in_progress" | "return_requested" => CourierStatus::ReturnInTransit,
            "returned" | "returned_to_merchant" => CourierStatus::Returned,
            "order_cancelled" | "cancelled" | "canceled" => CourierStatus::Cancelled,
            "lost" => CourierStatus::Lost,
            "damaged" => CourierStatus::Damaged,
            _ => CourierStatus::Unknown(raw.to_string()),
        }
    }
}

impl Default for PathaoFulfillmentProvider {
    fn default() -> Self {
        Self::new(PathaoConfig::default())
    }
}

#[async_trait]
impl FulfillmentProvider for PathaoFulfillmentProvider {
    fn identifier(&self) -> &'static str {
        "pathao"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![
            FulfillmentOptionDTO {
                id: "pathao-standard".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "pathao-sub-city".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "pathao-outside-city".to_string(),
                is_return: false,
            },
            FulfillmentOptionDTO {
                id: "pathao-return".to_string(),
                is_return: true,
            },
        ])
    }

    async fn validate_option(&self, option_data: Option<&serde_json::Value>) -> Result<bool> {
        if let Some(data) = option_data {
            if let Some(city_id) = data.get("city_id").and_then(|v| v.as_u64()) {
                if let Some(zone_id) = data.get("zone_id").and_then(|v| v.as_u64()) {
                    return self
                        .is_serviceable_area(city_id as u32, zone_id as u32, None)
                        .await;
                }
            }
        }
        Ok(true)
    }

    /// Calculate shipping rates across inside-city, sub-city, and outside-city tiers (P23-03).
    ///
    /// Inside city: 60 BDT base rate (+15 BDT per kg over 0.5kg)
    /// Sub-city: 100 BDT base rate (+20 BDT per kg over 1.0kg)
    /// Outside city: 130 BDT base rate (+25 BDT per kg over 1.0kg)
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
                        "pathao-sub-city" => "sub_city",
                        "pathao-outside-city" => "outside_city",
                        _ => "inside_city",
                    })
            })
            .or_else(|| data.and_then(|d| d.get("tier").and_then(|t| t.as_str())))
            .unwrap_or("inside_city");

        let (base_rate, weight_threshold, per_kg_charge) = match tier {
            "sub_city" => (dec!(100.00), dec!(1.0), dec!(20.00)),
            "outside_city" => (dec!(130.00), dec!(1.0), dec!(25.00)),
            _ => (dec!(60.00), dec!(0.5), dec!(15.00)),
        };

        // Override base rate if option_data provides explicit rate
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
                if weight > weight_threshold_compat(weight_threshold) {
                    let extra_weight = (weight - weight_threshold_compat(weight_threshold)).ceil();
                    final_rate += extra_weight * per_kg_charge;
                }
            }
        }

        Ok(final_rate)
    }

    /// Create fulfillment order in Pathao and obtain consignment tracking (P23-04).
    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        let merchant_ref = data
            .and_then(|d| d.get("merchant_order_id").or_else(|| d.get("reference")))
            .and_then(|v| v.as_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::Invalid {
                code: "pathao_order_reference_missing",
                message: "Pathao shipment requires a stable merchant order reference.".to_string(),
                field: Some("merchant_order_id".to_string()),
            })?;
        let store_id = data
            .and_then(|value| value.get("store_id"))
            .and_then(|value| {
                value
                    .as_str()
                    .and_then(|raw| raw.parse::<u32>().ok())
                    .or_else(|| value.as_u64().and_then(|raw| u32::try_from(raw).ok()))
            })
            .filter(|value| *value > 0)
            .or(self.config.store_id)
            .ok_or_else(|| Error::Invalid {
                code: "pathao_store_mapping_missing",
                message: "Pathao shipment requires the configured merchant store ID.".to_string(),
                field: Some("store_id".to_string()),
            })?;
        let recipient_name = required_text(data, "recipient_name", "Pathao recipient name")?;
        let recipient_phone = required_text(data, "recipient_phone", "Pathao recipient phone")?;
        let recipient_address =
            required_text(data, "recipient_address", "Pathao recipient address")?;
        let recipient_city = required_positive_u64(data, "recipient_city", "Pathao city mapping")?;
        let recipient_zone = required_positive_u64(data, "recipient_zone", "Pathao zone mapping")?;
        let amount_to_collect = optional_nonnegative_u64(data, "amount_to_collect")?;
        let base_url = match data
            .and_then(|value| value.get("sandbox_mode"))
            .and_then(serde_json::Value::as_bool)
        {
            Some(true) => "https://courier-api-sandbox.pathao.com",
            Some(false) => "https://api.pathao.com",
            None => self.config.base_url.as_str(),
        };
        let request_credentials = ["client_id", "client_secret", "username", "password"]
            .iter()
            .all(|key| {
                data.and_then(|value| value.get(key))
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty())
            });
        let token = if request_credentials {
            let credentials = data.ok_or_else(|| Error::Invalid {
                code: "pathao_credentials_missing",
                message: "Pathao merchant credentials are not configured.".to_string(),
                field: None,
            })?;
            let url = format!("{base_url}/aladdin/api/v1/issue-token");
            let response = self
                .client
                .post(url)
                .json(&serde_json::json!({
                    "client_id": credentials["client_id"],
                    "client_secret": credentials["client_secret"],
                    "grant_type": "password",
                    "username": credentials["username"],
                    "password": credentials["password"],
                }))
                .send()
                .await
                .map_err(|_| Error::Invalid {
                    code: "pathao_auth_request_failed",
                    message: "Pathao authentication request failed.".to_string(),
                    field: None,
                })?;
            if !response.status().is_success() {
                return Err(Error::Invalid {
                    code: "pathao_auth_rejected",
                    message: "Pathao rejected the configured merchant credentials.".to_string(),
                    field: None,
                });
            }
            let body: serde_json::Value = response.json().await.map_err(|_| Error::Invalid {
                code: "pathao_auth_response_invalid",
                message: "Pathao returned an invalid authentication response.".to_string(),
                field: None,
            })?;
            body.get("access_token")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| Error::Invalid {
                    code: "pathao_auth_response_invalid",
                    message: "Pathao returned an invalid authentication response.".to_string(),
                    field: None,
                })?
                .to_string()
        } else {
            self.get_token().await?
        };
        let url = format!("{base_url}/aladdin/api/v1/orders");

        let order_payload = serde_json::json!({
            "store_id": store_id,
            "merchant_order_id": merchant_ref,
            "recipient_name": recipient_name,
            "recipient_phone": recipient_phone,
            "recipient_address": recipient_address,
            "recipient_city": recipient_city,
            "recipient_zone": recipient_zone,
            "delivery_type": 48,
            "item_type": 1,
            "amount_to_collect": amount_to_collect,
        });

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .json(&order_payload)
            .send()
            .await;

        let response = resp.map_err(|_| Error::Invalid {
            code: "pathao_create_request_failed",
            message: "Pathao shipment request failed; shipment status is unknown.".to_string(),
            field: None,
        })?;
        if !response.status().is_success() {
            return Err(Error::Invalid {
                code: "pathao_create_rejected",
                message: format!(
                    "Pathao rejected shipment creation with HTTP {}.",
                    response.status()
                ),
                field: None,
            });
        }
        let body: serde_json::Value = response.json().await.map_err(|_| Error::Invalid {
            code: "pathao_create_response_invalid",
            message: "Pathao shipment response was invalid; shipment status is unknown."
                .to_string(),
            field: None,
        })?;
        let consignment_id = body
            .get("data")
            .and_then(|d| d.get("consignment_id"))
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::Invalid {
                code: "pathao_consignment_id_missing",
                message: "Pathao accepted the request but returned no consignment ID; shipment status is unknown.".to_string(),
                field: None,
            })?
            .to_string();
        let tracking_url =
            format!("https://courier.pathao.com/tracking?consignment_id={consignment_id}");

        let mut res_data = data.cloned().unwrap_or_else(|| serde_json::json!({}));
        if !res_data.is_object() {
            res_data = serde_json::json!({});
        }
        res_data["consignment_id"] = serde_json::json!(consignment_id);
        res_data["tracking_code"] = serde_json::json!(consignment_id);
        res_data["provider"] = serde_json::json!("pathao");

        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: consignment_id.clone(),
            tracking_url: Some(tracking_url),
            label_url: Some(format!(
                "https://courier.pathao.com/labels/{consignment_id}.pdf"
            )),
            metadata: Some(serde_json::json!({ "provider": "pathao" })),
        }];

        Ok(CreateFulfillmentResponse {
            data: Some(res_data),
            labels,
        })
    }

    /// Cancel a Pathao consignment (P23-04).
    async fn cancel_fulfillment(&self, data: Option<&serde_json::Value>) -> Result<()> {
        let consignment_id = data
            .and_then(|d| d.get("consignment_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Invalid {
                code: "missing_consignment_id",
                message: "Missing consignment_id for cancellation".to_string(),
                field: None,
            })?;

        let token = self.get_token().await?;
        let url = format!(
            "{}/aladdin/api/v1/orders/{consignment_id}/cancel",
            self.config.base_url
        );
        let _ = self.client.post(&url).bearer_auth(token).send().await;

        Ok(())
    }

    /// Create return fulfillment (P23-04).
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
            data.and_then(|d| d.get("consignment_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("PATHAO-001")
        );
        return_data["consignment_id"] = serde_json::json!(ret_cns);

        let labels = vec![CreateFulfillmentLabelInput {
            tracking_number: ret_cns.clone(),
            tracking_url: Some(format!(
                "https://courier.pathao.com/tracking?consignment_id={ret_cns}"
            )),
            label_url: Some(format!("https://courier.pathao.com/labels/{ret_cns}.pdf")),
            metadata: Some(serde_json::json!({ "is_return": true })),
        }];

        Ok(CreateFulfillmentResponse {
            data: Some(return_data),
            labels,
        })
    }

    /// Map webhook payload to consignment ID and CourierStatus (P23-05).
    fn map_webhook_event(
        &self,
        payload: &serde_json::Value,
    ) -> Result<Option<(String, CourierStatus)>> {
        let consignment_id = payload
            .get("consignment_id")
            .or_else(|| payload.get("id"))
            .or_else(|| payload.get("order_id"))
            .and_then(|v| v.as_str());

        let raw_status = payload
            .get("order_status")
            .or_else(|| payload.get("order_status_slug"))
            .or_else(|| payload.get("event"))
            .or_else(|| payload.get("status"))
            .and_then(|v| v.as_str());

        match (consignment_id, raw_status) {
            (Some(cid), Some(raw)) => {
                let status = Self::map_pathao_status(raw);
                Ok(Some((cid.to_string(), status)))
            }
            _ => Ok(None),
        }
    }

    /// Live tracking lookup for Pathao (P23-10).
    async fn get_tracking_status(&self, reference: &str) -> Result<CourierStatus> {
        let token = self.get_token().await?;
        let url = format!("{}/aladdin/api/v1/orders/{reference}", self.config.base_url);
        let resp = self.client.get(&url).bearer_auth(token).send().await;

        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    if let Some(status_str) = json
                        .get("data")
                        .and_then(|d| d.get("order_status").or_else(|| d.get("order_status_slug")))
                        .and_then(|v| v.as_str())
                    {
                        return Ok(Self::map_pathao_status(status_str));
                    }
                }
            }
        }

        Ok(CourierStatus::Created)
    }
}

fn weight_threshold_compat(t: Decimal) -> Decimal {
    t
}
