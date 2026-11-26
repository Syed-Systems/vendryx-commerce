//! Admin Couriers & Logistics Gateway API (TRUTH-01, ADM-025).
//!
//! Exposes tenant-scoped configuration, encrypted credential storage, real gateway
//! connection probes, and rate zone management for domestic delivery couriers:
//! Pathao, Steadfast, and RedX.
//!
//! Replaces simulated browser storage and `setTimeout` probes with real server-side
//! requests against courier APIs.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use stheno_api_core::ErrorResponse;
use stheno_fulfillment::FulfillmentService;
use stheno_payment::{
    decrypt_credential_map, encrypt_credential_map, CredentialMap, EncryptedCredential,
};
use stheno_settings::dto::SetStoreSettingInput;
use stheno_settings::SettingsService;
use stheno_types::Error;

#[derive(Clone)]
pub struct CourierContext {
    pub fulfillment_service: Arc<FulfillmentService>,
    pub settings_service: Arc<SettingsService>,
}

fn failed(status: StatusCode, code: &'static str, message: String) -> Response {
    (status, Json(ErrorResponse::new(code, &message))).into_response()
}

const ENCRYPTED_CREDENTIALS_FIELD: &str = "_encrypted_credentials_v1";

fn credential_error(_error: Error) -> Response {
    failed(
        StatusCode::INTERNAL_SERVER_ERROR,
        "courier_credentials_unavailable",
        "Courier credentials could not be securely loaded. Check the encryption key and stored credential record.".to_string(),
    )
}

fn decode_encrypted_credentials(
    value: &serde_json::Value,
) -> Result<Option<EncryptedCredential>, Error> {
    let Some(envelope) = value.get(ENCRYPTED_CREDENTIALS_FIELD) else {
        return Ok(None);
    };
    let ciphertext = envelope
        .get("ciphertext")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::Invalid {
            code: "courier_credentials_envelope_invalid",
            message: "Encrypted courier credential envelope is invalid.".to_string(),
            field: None,
        })?;
    let nonce = envelope
        .get("nonce")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::Invalid {
            code: "courier_credentials_envelope_invalid",
            message: "Encrypted courier credential envelope is invalid.".to_string(),
            field: None,
        })?;
    let ciphertext = hex::decode(ciphertext).map_err(|_| Error::Invalid {
        code: "courier_credentials_envelope_invalid",
        message: "Encrypted courier credential envelope is invalid.".to_string(),
        field: None,
    })?;
    let nonce = hex::decode(nonce).map_err(|_| Error::Invalid {
        code: "courier_credentials_envelope_invalid",
        message: "Encrypted courier credential envelope is invalid.".to_string(),
        field: None,
    })?;
    Ok(Some(EncryptedCredential { ciphertext, nonce }))
}

fn encode_encrypted_credentials(encrypted: EncryptedCredential) -> serde_json::Value {
    json!({
        ENCRYPTED_CREDENTIALS_FIELD: {
            "ciphertext": hex::encode(encrypted.ciphertext),
            "nonce": hex::encode(encrypted.nonce),
        }
    })
}

pub(crate) fn destination_mappings_valid(provider_id: &str, credentials: &CredentialMap) -> bool {
    let Some(raw) = credentials
        .get("destination_mappings")
        .and_then(serde_json::Value::as_str)
    else {
        return false;
    };
    let Ok(mappings) = serde_json::from_str::<serde_json::Value>(raw) else {
        return false;
    };
    let Some(mappings) = mappings.as_object().filter(|mappings| !mappings.is_empty()) else {
        return false;
    };
    let mut seen_keys = std::collections::BTreeSet::new();
    mappings.iter().all(|(key, mapping)| {
        let parts = key.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 3 || !parts[0].eq_ignore_ascii_case("bd") || parts[2].is_empty() {
            return false;
        }
        let canonical_key = parts
            .iter()
            .map(|part| part.to_lowercase())
            .collect::<Vec<_>>()
            .join("|");
        if !seen_keys.insert(canonical_key) {
            return false;
        }
        match provider_id {
            "pathao" => ["city_id", "zone_id"].iter().all(|field| {
                mapping
                    .get(field)
                    .and_then(serde_json::Value::as_u64)
                    .is_some_and(|value| value > 0)
            }),
            "redx" => {
                mapping
                    .get("area_id")
                    .and_then(serde_json::Value::as_u64)
                    .is_some_and(|value| value > 0)
                    && mapping
                        .get("parcel_weight")
                        .and_then(serde_json::Value::as_u64)
                        .is_some_and(|value| value > 0)
                    && mapping
                        .get("area_name")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| !value.trim().is_empty())
            }
            _ => false,
        }
    })
}

pub(crate) async fn load_courier_credentials(
    settings: &SettingsService,
    provider_id: &str,
) -> Result<CredentialMap, Error> {
    let key = format!("courier_{provider_id}");
    let setting = match settings.get_setting(&key).await {
        Ok(setting) => setting,
        Err(Error::NotFound { .. }) => return Ok(CredentialMap::new()),
        Err(error) => return Err(error),
    };
    let tenant_id = stheno_db::tenant::current()?;
    let value = setting.value;
    if let Some(encrypted) = decode_encrypted_credentials(&value)? {
        return decrypt_credential_map(&tenant_id, provider_id, &encrypted);
    }

    // Existing deployments stored courier secrets as plain JSON. Encrypt and persist
    // them on first access before returning them to the caller.
    let legacy = value.as_object().cloned().ok_or_else(|| Error::Invalid {
        code: "courier_credentials_record_invalid",
        message: "Stored courier credential record is invalid.".to_string(),
        field: None,
    })?;
    let credentials = legacy.into_iter().collect::<CredentialMap>();
    let encrypted = encrypt_credential_map(&tenant_id, provider_id, &credentials)?;
    settings
        .set_setting(SetStoreSettingInput {
            key,
            value: encode_encrypted_credentials(encrypted),
            category: "couriers".to_string(),
            is_secret: Some(true),
        })
        .await?;
    Ok(credentials)
}

async fn store_courier_credentials(
    settings: &SettingsService,
    provider_id: &str,
    credentials: &CredentialMap,
) -> Result<(), Error> {
    let tenant_id = stheno_db::tenant::current()?;
    let encrypted = encrypt_credential_map(&tenant_id, provider_id, credentials)?;
    settings
        .set_setting(SetStoreSettingInput {
            key: format!("courier_{provider_id}"),
            value: encode_encrypted_credentials(encrypted),
            category: "couriers".to_string(),
            is_secret: Some(true),
        })
        .await?;
    Ok(())
}

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierRates {
    pub inside_dhaka: Decimal,
    pub sub_dhaka: Decimal,
    pub outside_dhaka: Decimal,
}

impl Default for CourierRates {
    fn default() -> Self {
        Self {
            inside_dhaka: Decimal::new(60, 0),
            sub_dhaka: Decimal::new(100, 0),
            outside_dhaka: Decimal::new(130, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierProbeRecord {
    pub verified: bool,
    pub status: String,
    pub latency_ms: u64,
    pub message: String,
    pub probed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierFieldMeta {
    pub key: String,
    pub label: String,
    pub kind: String, // "text" | "secret" | "bool"
    pub required: bool,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourierProviderInfo {
    pub id: String,
    pub name: String,
    pub status: String, // "not_configured" | "connected" | "needs_verification" | "failed"
    pub is_enabled: bool,
    pub webhook_url: String,
    pub fields: Vec<CourierFieldMeta>,
    pub credentials: HashMap<String, serde_json::Value>,
    pub rates: CourierRates,
    pub last_probe: Option<CourierProbeRecord>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertCourierCredentialsRequest {
    pub credentials: HashMap<String, serde_json::Value>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCourierRatesRequest {
    pub inside_dhaka: Decimal,
    pub sub_dhaka: Decimal,
    pub outside_dhaka: Decimal,
}

// ── Provider Definitions ──────────────────────────────────────────────────────

fn provider_metadata(id: &str) -> Option<(&'static str, Vec<CourierFieldMeta>)> {
    match id {
        "pathao" => Some((
            "Pathao Courier",
            vec![
                CourierFieldMeta {
                    key: "client_id".to_string(),
                    label: "Client ID".to_string(),
                    kind: "text".to_string(),
                    required: true,
                    placeholder: Some("From Pathao Merchant Portal".to_string()),
                },
                CourierFieldMeta {
                    key: "client_secret".to_string(),
                    label: "Client Secret".to_string(),
                    kind: "secret".to_string(),
                    required: true,
                    placeholder: None,
                },
                CourierFieldMeta {
                    key: "username".to_string(),
                    label: "Merchant Email / Username".to_string(),
                    kind: "text".to_string(),
                    required: true,
                    placeholder: None,
                },
                CourierFieldMeta {
                    key: "password".to_string(),
                    label: "Password".to_string(),
                    kind: "secret".to_string(),
                    required: true,
                    placeholder: None,
                },
                CourierFieldMeta {
                    key: "store_id".to_string(),
                    label: "Store ID".to_string(),
                    kind: "text".to_string(),
                    required: true,
                    placeholder: Some("e.g. 10243".to_string()),
                },
                CourierFieldMeta {
                    key: "destination_mappings".to_string(),
                    label: "Destination mappings (JSON)".to_string(),
                    kind: "text".to_string(),
                    required: true,
                    placeholder: Some("{\"BD|Dhaka|Dhaka\":{\"city_id\":1,\"zone_id\":101}}".to_string()),
                },
                CourierFieldMeta {
                    key: "sandbox_mode".to_string(),
                    label: "Sandbox Mode".to_string(),
                    kind: "bool".to_string(),
                    required: false,
                    placeholder: None,
                },
            ],
        )),
        "steadfast" => Some((
            "Steadfast Courier",
            vec![
                CourierFieldMeta {
                    key: "api_key".to_string(),
                    label: "API Key".to_string(),
                    kind: "secret".to_string(),
                    required: true,
                    placeholder: Some("From Steadfast Merchant Portal".to_string()),
                },
                CourierFieldMeta {
                    key: "secret_key".to_string(),
                    label: "Secret Key".to_string(),
                    kind: "secret".to_string(),
                    required: true,
                    placeholder: None,
                },
            ],
        )),
        "redx" => Some((
            "RedX Logistics",
            vec![
                CourierFieldMeta {
                    key: "api_key".to_string(),
                    label: "Bearer Token / API Key".to_string(),
                    kind: "secret".to_string(),
                    required: true,
                    placeholder: Some("RedX Authorization Token".to_string()),
                },
                CourierFieldMeta {
                    key: "sandbox_mode".to_string(),
                    label: "Sandbox Mode".to_string(),
                    kind: "bool".to_string(),
                    required: false,
                    placeholder: None,
                },
                CourierFieldMeta {
                    key: "destination_mappings".to_string(),
                    label: "Destination mappings (JSON)".to_string(),
                    kind: "text".to_string(),
                    required: true,
                    placeholder: Some("{\"BD|Dhaka|Dhaka\":{\"area_id\":1,\"area_name\":\"Dhanmondi\",\"parcel_weight\":500}}".to_string()),
                },
            ],
        )),
        _ => None,
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /admin/couriers`
pub async fn list_couriers(
    State(ctx): State<CourierContext>,
) -> Result<impl IntoResponse, Response> {
    let provider_ids = ["pathao", "steadfast", "redx"];
    let mut list = Vec::new();

    for id in provider_ids {
        let (name, fields) = match provider_metadata(id) {
            Some(meta) => meta,
            None => continue,
        };

        let rate_key = format!("courier_rates_{id}");
        let probe_key = format!("courier_probe_{id}");

        let stored_creds = load_courier_credentials(&ctx.settings_service, id)
            .await
            .map_err(credential_error)?;

        let is_enabled = stored_creds
            .get("is_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Mask secret fields
        let mut masked_creds = HashMap::new();
        let mut has_configured_keys = false;

        for field in &fields {
            if let Some(val) = stored_creds.get(&field.key) {
                if !val.is_null() && val.as_str().map_or(true, |s| !s.is_empty()) {
                    has_configured_keys = true;
                }
                if field.kind == "secret" {
                    let str_val = val.as_str().unwrap_or("");
                    if !str_val.is_empty() {
                        masked_creds.insert(field.key.clone(), json!("••••••••"));
                    } else {
                        masked_creds.insert(field.key.clone(), json!(""));
                    }
                } else {
                    masked_creds.insert(field.key.clone(), val.clone());
                }
            }
        }

        // Load rates
        let rates: CourierRates = ctx
            .settings_service
            .get_setting(&rate_key)
            .await
            .ok()
            .and_then(|s| serde_json::from_value(s.value).ok())
            .unwrap_or_default();

        // Load last probe result
        let last_probe: Option<CourierProbeRecord> = ctx
            .settings_service
            .get_setting(&probe_key)
            .await
            .ok()
            .and_then(|s| serde_json::from_value(s.value).ok());

        // Derive status
        let status = if !has_configured_keys {
            "not_configured".to_string()
        } else if let Some(probe) = &last_probe {
            probe.status.clone()
        } else {
            "needs_verification".to_string()
        };

        list.push(CourierProviderInfo {
            id: id.to_string(),
            name: name.to_string(),
            status,
            is_enabled,
            webhook_url: format!("https://api.vendryx.shop/hooks/courier/{id}"),
            fields,
            credentials: masked_creds,
            rates,
            last_probe,
        });
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "couriers": list, "data": list })),
    ))
}

/// `POST /admin/couriers/{id}/credentials`
pub async fn upsert_courier_credentials(
    State(ctx): State<CourierContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpsertCourierCredentialsRequest>,
) -> Result<impl IntoResponse, Response> {
    let (_, fields) = provider_metadata(&id).ok_or_else(|| {
        failed(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("Courier '{id}' is not supported"),
        )
    })?;

    // Load existing credentials so masked fields aren't overwritten with literal bullet points
    let existing_creds = load_courier_credentials(&ctx.settings_service, &id)
        .await
        .map_err(credential_error)?;

    let mut merged = existing_creds;
    for (k, v) in payload.credentials {
        let Some(field) = fields.iter().find(|field| field.key == k) else {
            return Err(failed(
                StatusCode::BAD_REQUEST,
                "credential_field_unknown",
                format!("Unknown courier credential field '{k}'."),
            ));
        };
        if let Some(s) = v.as_str() {
            // If the client submitted masked dots, preserve existing value
            if s.contains('•') {
                continue;
            }
        }
        let valid = match field.kind.as_str() {
            "text" | "secret" => v.is_string(),
            "bool" => v.is_boolean(),
            _ => false,
        };
        if !valid {
            return Err(failed(
                StatusCode::BAD_REQUEST,
                "credential_field_invalid",
                format!("Courier credential field '{k}' has an invalid value."),
            ));
        }
        merged.insert(k, v);
    }

    if let Some(enabled) = payload.is_enabled {
        merged.insert("is_enabled".to_string(), json!(enabled));
    }

    let enabled = merged
        .get("is_enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let required_missing = fields.iter().filter(|field| field.required).any(|field| {
        merged
            .get(&field.key)
            .and_then(serde_json::Value::as_str)
            .map_or(true, |value| value.trim().is_empty())
    });
    if enabled && required_missing {
        return Err(failed(
            StatusCode::BAD_REQUEST,
            "courier_credentials_incomplete",
            "Required credentials must be configured before enabling this courier.".to_string(),
        ));
    }
    if enabled
        && matches!(id.as_str(), "pathao" | "redx")
        && !destination_mappings_valid(&id, &merged)
    {
        return Err(failed(
            StatusCode::BAD_REQUEST,
            "courier_destination_mappings_invalid",
            "Destination mappings must be a non-empty JSON object keyed by country|province|city, with valid provider IDs and shipment details.".to_string(),
        ));
    }

    store_courier_credentials(&ctx.settings_service, &id, &merged)
        .await
        .map_err(|_| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Courier credentials could not be securely stored. Check the encryption key and database availability.".to_string(),
            )
        })?;

    // Reset probe status to needs_verification
    let probe_key = format!("courier_probe_{id}");
    let _ = ctx
        .settings_service
        .set_setting(SetStoreSettingInput {
            key: probe_key,
            value: json!(CourierProbeRecord {
                verified: false,
                status: "needs_verification".to_string(),
                latency_ms: 0,
                message: "Credentials updated; verification probe required.".to_string(),
                probed_at: Utc::now(),
            }),
            category: "couriers".to_string(),
            is_secret: Some(false),
        })
        .await;

    // Return updated courier info
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "status": "needs_verification",
            "message": "Courier credentials saved successfully.",
        })),
    ))
}

/// `POST /admin/couriers/{id}/probe` — executes a real server-side health handshake.
pub async fn probe_courier(
    State(ctx): State<CourierContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let (name, _) = provider_metadata(&id).ok_or_else(|| {
        failed(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("Courier '{id}' is not supported"),
        )
    })?;

    let probe_key = format!("courier_probe_{id}");

    let creds = load_courier_credentials(&ctx.settings_service, &id)
        .await
        .map_err(credential_error)?;

    if creds.is_empty() {
        return Err(failed(
            StatusCode::BAD_REQUEST,
            "not_configured",
            format!("{name} credentials are not configured yet."),
        ));
    }

    let start = Instant::now();
    let mut verified = false;
    let mut message = String::new();

    match id.as_str() {
        "pathao" => {
            let client_id = creds
                .get("client_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let client_secret = creds
                .get("client_secret")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let username = creds.get("username").and_then(|v| v.as_str()).unwrap_or("");
            let password = creds.get("password").and_then(|v| v.as_str()).unwrap_or("");
            let sandbox = creds
                .get("sandbox_mode")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            if client_id.is_empty()
                || client_secret.is_empty()
                || username.is_empty()
                || password.is_empty()
            {
                message =
                    "Pathao requires Client ID, Client Secret, Username, and Password.".to_string();
            } else {
                let base_url = if sandbox {
                    "https://courier-api-sandbox.pathao.com"
                } else {
                    "https://api.pathao.com"
                };

                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()
                    .unwrap_or_default();

                let resp = client
                    .post(format!("{base_url}/aladdin/api/v1/issue-token"))
                    .json(&json!({
                        "client_id": client_id,
                        "client_secret": client_secret,
                        "grant_type": "password",
                        "username": username,
                        "password": password,
                    }))
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        if r.status().is_success() {
                            verified = true;
                            message =
                                "Gateway handshake successful: Pathao issued authorization token."
                                    .to_string();
                        } else {
                            message = format!(
                                "Pathao gateway rejected credentials with HTTP status {}",
                                r.status()
                            );
                        }
                    }
                    Err(e) => {
                        message = format!("Pathao gateway network connection error: {e}");
                    }
                }
            }
        }
        "steadfast" => {
            let api_key = creds.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
            let secret_key = creds
                .get("secret_key")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if api_key.is_empty() || secret_key.is_empty() {
                message = "Steadfast requires API Key and Secret Key.".to_string();
            } else {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()
                    .unwrap_or_default();

                let resp = client
                    .get("https://portal.packzy.com/api/v1/get_balance")
                    .header("Api-Key", api_key)
                    .header("Secret-Key", secret_key)
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        if r.status().is_success() {
                            verified = true;
                            message = "Gateway handshake successful: Steadfast merchant balance retrieved.".to_string();
                        } else {
                            message = format!(
                                "Steadfast gateway rejected credentials with HTTP status {}",
                                r.status()
                            );
                        }
                    }
                    Err(e) => {
                        message = format!("Steadfast gateway network connection error: {e}");
                    }
                }
            }
        }
        "redx" => {
            let api_key = creds.get("api_key").and_then(|v| v.as_str()).unwrap_or("");
            let sandbox = creds
                .get("sandbox_mode")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if api_key.is_empty() {
                message = "RedX requires Bearer Token / API Key.".to_string();
            } else {
                let base_url = if sandbox {
                    "https://openapi.redx.com.bd/v1.0.0-beta"
                } else {
                    "https://openapi.redx.com.bd/v1.0.0"
                };

                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()
                    .unwrap_or_default();

                let resp = client
                    .get(format!("{base_url}/areas"))
                    .header("Authorization", format!("Bearer {api_key}"))
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        if r.status().is_success() {
                            verified = true;
                            message = "Gateway handshake successful: RedX delivery area coverage verified.".to_string();
                        } else {
                            message = format!("RedX gateway returned HTTP status {}", r.status());
                        }
                    }
                    Err(e) => {
                        message = format!("RedX gateway network connection error: {e}");
                    }
                }
            }
        }
        _ => {}
    }

    let latency_ms = start.elapsed().as_millis() as u64;
    let status = if verified { "connected" } else { "failed" };

    let record = CourierProbeRecord {
        verified,
        status: status.to_string(),
        latency_ms,
        message: message.clone(),
        probed_at: Utc::now(),
    };

    // Store probe record
    let _ = ctx
        .settings_service
        .set_setting(SetStoreSettingInput {
            key: probe_key,
            value: json!(record),
            category: "couriers".to_string(),
            is_secret: Some(false),
        })
        .await;

    Ok((
        StatusCode::OK,
        Json(json!({
            "verified": verified,
            "status": status,
            "latency_ms": latency_ms,
            "message": message,
            "courier_id": id,
        })),
    ))
}

/// `POST /admin/couriers/{id}/rates`
pub async fn update_courier_rates(
    State(ctx): State<CourierContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCourierRatesRequest>,
) -> Result<impl IntoResponse, Response> {
    if provider_metadata(&id).is_none() {
        return Err(failed(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("Courier '{id}' is not supported"),
        ));
    }

    if payload.inside_dhaka < Decimal::ZERO
        || payload.sub_dhaka < Decimal::ZERO
        || payload.outside_dhaka < Decimal::ZERO
    {
        return Err(failed(
            StatusCode::BAD_REQUEST,
            "invalid_rates",
            "Courier delivery rates cannot be negative".to_string(),
        ));
    }

    let rates = CourierRates {
        inside_dhaka: payload.inside_dhaka,
        sub_dhaka: payload.sub_dhaka,
        outside_dhaka: payload.outside_dhaka,
    };

    let rate_key = format!("courier_rates_{id}");
    ctx.settings_service
        .set_setting(SetStoreSettingInput {
            key: rate_key,
            value: json!(rates),
            category: "couriers".to_string(),
            is_secret: Some(false),
        })
        .await
        .map_err(|e| {
            failed(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(json!({ "id": id, "rates": rates }))))
}

#[cfg(test)]
mod credential_envelope_tests {
    use super::{
        decode_encrypted_credentials, destination_mappings_valid, encode_encrypted_credentials,
        CredentialMap,
    };
    use stheno_payment::EncryptedCredential;

    #[test]
    fn encrypted_envelope_round_trips_as_ciphertext_and_nonce() {
        let encrypted = EncryptedCredential {
            ciphertext: vec![0x10, 0x20, 0x30],
            nonce: vec![0x40, 0x50, 0x60],
        };
        let encoded = encode_encrypted_credentials(encrypted.clone());
        assert!(encoded.get("api_key").is_none());
        assert_eq!(
            decode_encrypted_credentials(&encoded).expect("decode envelope"),
            Some(encrypted)
        );
    }

    #[test]
    fn malformed_encrypted_envelope_fails_closed() {
        let malformed =
            serde_json::json!({ "_encrypted_credentials_v1": { "ciphertext": "not-hex" } });

        assert!(decode_encrypted_credentials(&malformed).is_err());
    }

    #[test]
    fn destination_mappings_require_exact_owner_configured_locations() {
        let mut credentials = CredentialMap::new();
        credentials.insert(
            "destination_mappings".to_string(),
            serde_json::json!(r#"{"dhaka":{"city_id":1,"zone_id":2}}"#),
        );
        assert!(!destination_mappings_valid("pathao", &credentials));

        credentials.insert(
            "destination_mappings".to_string(),
            serde_json::json!(r#"{"bd|dhaka|dhaka":{"city_id":1,"zone_id":2}}"#),
        );
        assert!(destination_mappings_valid("pathao", &credentials));
        credentials.insert(
            "destination_mappings".to_string(),
            serde_json::json!(r#"{"bd|dhaka|dhaka":{"city_id":1,"zone_id":2},"BD|Dhaka|Dhaka":{"city_id":3,"zone_id":4}}"#),
        );
        assert!(!destination_mappings_valid("pathao", &credentials));

        credentials.insert(
            "destination_mappings".to_string(),
            serde_json::json!(
                r#"{"bd|dhaka|dhaka":{"area_id":3,"area_name":"Dhanmondi","parcel_weight":500}}"#
            ),
        );
        assert!(destination_mappings_valid("redx", &credentials));
        credentials.insert(
            "destination_mappings".to_string(),
            serde_json::json!(
                r#"{"bd|dhaka|dhaka":{"area_id":3,"area_name":"Dhanmondi","parcel_weight":0}}"#
            ),
        );
        assert!(!destination_mappings_valid("redx", &credentials));
    }
}

/// `GET /admin/couriers/{id}/rates`
pub async fn get_courier_rates(
    State(ctx): State<CourierContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Response> {
    if provider_metadata(&id).is_none() {
        return Err(failed(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("Courier '{id}' is not supported"),
        ));
    }

    let rate_key = format!("courier_rates_{id}");
    let rates = if let Ok(setting) = ctx.settings_service.get_setting(&rate_key).await {
        serde_json::from_value::<CourierRates>(setting.value).unwrap_or_default()
    } else {
        CourierRates::default()
    };

    Ok((StatusCode::OK, Json(json!({ "id": id, "rates": rates }))))
}

/// Mounts admin courier logistics gateway routes.
pub fn admin_courier_routes(
    fulfillment_service: Arc<FulfillmentService>,
    settings_service: Arc<SettingsService>,
) -> Router {
    let ctx = CourierContext {
        fulfillment_service,
        settings_service,
    };
    Router::new()
        .route("/admin/couriers", get(list_couriers))
        .route("/admin/couriers/{id}", post(upsert_courier_credentials))
        .route("/admin/couriers/{id}/probe", post(probe_courier))
        .route(
            "/admin/couriers/{id}/rates",
            get(get_courier_rates).post(update_courier_rates),
        )
        .with_state(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn courier_metadata_returns_all_three_bangladesh_providers() {
        assert!(provider_metadata("pathao").is_some());
        assert!(provider_metadata("steadfast").is_some());
        assert!(provider_metadata("redx").is_some());
        assert!(provider_metadata("unknown_courier").is_none());
    }

    #[test]
    fn courier_rates_default_is_positive() {
        let rates = CourierRates::default();
        assert_eq!(rates.inside_dhaka, Decimal::new(60, 0));
        assert_eq!(rates.sub_dhaka, Decimal::new(100, 0));
        assert_eq!(rates.outside_dhaka, Decimal::new(130, 0));
    }
}
