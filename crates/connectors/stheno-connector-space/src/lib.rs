//! `stheno-connector-space`: Client and webhook integration connector for Space Control Plane.
//!
//! Connects the Vendryx Commerce product (Store) to Space's `/space/integration/v1/*` boundary:
//! - Manifest registration and capability validation
//! - Live observed-state and drift reporting (`POST /instances/{ref}/observed-state`)
//! - ETag-backed conditional entitlement snapshot retrieval (`GET /instances/{ref}/entitlements`)
//! - Deduplicated usage event batch telemetry (`POST /usage-events`)
//! - Inbound webhook verification using HMAC-SHA256 signatures with replay protection.

use hmac::{Hmac, Mac};
use reqwest::header::{ETAG, IF_NONE_MATCH};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Configuration for connecting Store to Space Control Plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceConnectorConfig {
    pub space_api_url: String,
    pub instance_ref: String,
    pub webhook_secret: String,
    pub tolerance_seconds: u64,
}

impl Default for SpaceConnectorConfig {
    fn default() -> Self {
        Self {
            space_api_url: "http://127.0.0.1:9100".to_string(),
            instance_ref: "prod_inst_vendryx_default".to_string(),
            webhook_secret: "dev-space-connector-secret".to_string(),
            tolerance_seconds: 300,
        }
    }
}

impl SpaceConnectorConfig {
    pub fn from_env() -> Self {
        Self {
            space_api_url: std::env::var("SPACE_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9100".to_string()),
            instance_ref: std::env::var("SPACE_INSTANCE_REF")
                .unwrap_or_else(|_| "prod_inst_vendryx_default".to_string()),
            webhook_secret: std::env::var("SPACE_CONNECTOR_SECRET")
                .unwrap_or_else(|_| "dev-space-connector-secret".to_string()),
            tolerance_seconds: std::env::var("SPACE_TOLERANCE_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        }
    }
}

/// Observed state update payload sent to Space.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateObservedStateInput {
    pub observed_state: String,
    pub observed_revision: i64,
}

/// Observed state response from Space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedStateResponse {
    pub instance: serde_json::Value,
    pub drift_status: String,
}

/// Snapshot of granted entitlements received from Space.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitlementsSnapshot {
    pub instance_ref: String,
    pub etag: String,
    pub revision: i64,
    pub issued_at_epoch_sec: u64,
    pub grants: HashMap<String, serde_json::Value>,
}

/// Result of fetching entitlements conditionally with ETag.
#[derive(Debug, Clone, PartialEq)]
pub enum EntitlementsFetchResult {
    Fresh(EntitlementsSnapshot),
    NotModified { etag: String },
}

/// Usage event item for billing and telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageEventRecord {
    pub id: String,
    pub instance_id: String,
    pub metric: String,
    pub quantity: u64,
    pub idempotency_key: String,
    pub timestamp_epoch_sec: u64,
}

/// Batch usage ingestion input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEventBatchInput {
    pub events: Vec<UsageEventRecord>,
}

/// Response returned from Space usage ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBatchResponse {
    pub accepted: usize,
    pub deduplicated: usize,
}

/// Capability declaration inside manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestCapabilityInput {
    pub key: String,
    pub kind: String,
    pub unit: Option<String>,
    pub default_value: serde_json::Value,
    pub description: String,
}

/// Manifest validation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateManifestInput {
    pub product_key: String,
    pub connector_kind: String,
    pub manifest_version: String,
    pub capabilities: Vec<ManifestCapabilityInput>,
}

/// Manifest validation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateManifestResponse {
    pub valid: bool,
    pub product_key: String,
    pub connector_kind: String,
    pub capabilities_count: usize,
    pub validation_errors: Vec<String>,
}

/// Inbound webhook payload from Space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceWebhookPayload {
    pub event: String,
    pub timestamp: Option<u64>,
    pub store_id: Option<String>,
    pub instance_ref: Option<String>,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookVerifyError {
    TimestampExpired { skew_seconds: u64 },
    InvalidHeaderFormat,
    SignatureMismatch,
    MalformedPayload(String),
}

impl std::fmt::Display for WebhookVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimestampExpired { skew_seconds } => {
                write!(f, "Webhook timestamp expired (skew: {}s)", skew_seconds)
            }
            Self::InvalidHeaderFormat => write!(f, "Invalid signature header format"),
            Self::SignatureMismatch => write!(f, "Signature verification failed (digest mismatch)"),
            Self::MalformedPayload(msg) => write!(f, "Malformed webhook payload: {}", msg),
        }
    }
}

impl std::error::Error for WebhookVerifyError {}

/// Generates standard HMAC-SHA256 signature header: `t={timestamp},v1={hex_digest}`.
pub fn sign_space_webhook(secret: &[u8], timestamp: u64, payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    let signed_content = format!("{}.", timestamp);
    mac.update(signed_content.as_bytes());
    mac.update(payload);
    let signature_hex = hex::encode(mac.finalize().into_bytes());
    format!("t={},v1={}", timestamp, signature_hex)
}

/// Verifies incoming Space webhook with replay-attack timestamp window protection.
pub fn verify_space_webhook(
    secret: &[u8],
    timestamp: u64,
    signature_header: &str,
    payload: &[u8],
    max_skew_sec: u64,
    current_time: u64,
) -> Result<(), WebhookVerifyError> {
    let skew = current_time.abs_diff(timestamp);
    if skew > max_skew_sec {
        return Err(WebhookVerifyError::TimestampExpired { skew_seconds: skew });
    }

    let mut expected_v1: Option<&str> = None;
    for part in signature_header.split(',') {
        let mut kv = part.splitn(2, '=');
        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
            if k.trim() == "v1" {
                expected_v1 = Some(v.trim());
            }
        }
    }

    let received_sig = expected_v1.ok_or(WebhookVerifyError::InvalidHeaderFormat)?;

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    let signed_content = format!("{}.", timestamp);
    mac.update(signed_content.as_bytes());
    mac.update(payload);
    let expected_bytes = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected_bytes);

    if received_sig.as_bytes().ct_eq(expected_hex.as_bytes()).into() {
        Ok(())
    } else {
        Err(WebhookVerifyError::SignatureMismatch)
    }
}

/// Errors occurring during connector HTTP requests.
#[derive(Debug)]
pub enum SpaceConnectorError {
    Http(reqwest::Error),
    ApiError {
        status: u16,
        title: String,
        detail: String,
    },
    Webhook(WebhookVerifyError),
    Serialization(serde_json::Error),
    InvalidConfig(String),
}

impl std::fmt::Display for SpaceConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "Space HTTP error: {}", e),
            Self::ApiError {
                status,
                title,
                detail,
            } => {
                write!(f, "Space API error ({}): {} - {}", status, title, detail)
            }
            Self::Webhook(e) => write!(f, "Space Webhook error: {}", e),
            Self::Serialization(e) => write!(f, "Space serialization error: {}", e),
            Self::InvalidConfig(msg) => write!(f, "Invalid Space config: {}", msg),
        }
    }
}

impl std::error::Error for SpaceConnectorError {}

impl From<reqwest::Error> for SpaceConnectorError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<serde_json::Error> for SpaceConnectorError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e)
    }
}

impl From<WebhookVerifyError> for SpaceConnectorError {
    fn from(e: WebhookVerifyError) -> Self {
        Self::Webhook(e)
    }
}

/// HTTP client communicating with Space Control Plane.
#[derive(Clone)]
pub struct SpaceClient {
    client: reqwest::Client,
    config: SpaceConnectorConfig,
}

impl SpaceClient {
    pub fn new(config: SpaceConnectorConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            config,
        }
    }

    pub fn config(&self) -> &SpaceConnectorConfig {
        &self.config
    }

    /// Report Store's observed operational state and revision to Space Control Plane.
    pub async fn report_observed_state(
        &self,
        observed_state: &str,
        observed_revision: i64,
    ) -> Result<ObservedStateResponse, SpaceConnectorError> {
        let url = format!(
            "{}/space/integration/v1/instances/{}/observed-state",
            self.config.space_api_url.trim_end_matches('/'),
            self.config.instance_ref
        );

        let input = UpdateObservedStateInput {
            observed_state: observed_state.to_string(),
            observed_revision,
        };
        let body_bytes = serde_json::to_vec(&input)?;
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let sig = sign_space_webhook(self.config.webhook_secret.as_bytes(), timestamp, &body_bytes);

        let response = self.client.post(&url)
            .header("x-space-signature", sig)
            .header("content-type", "application/json")
            .body(body_bytes)
            .send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(SpaceConnectorError::ApiError {
                status,
                title: "Failed to report observed state".to_string(),
                detail: text,
            });
        }

        let body: ObservedStateResponse = response.json().await?;
        Ok(body)
    }

    /// Fetch latest entitlement snapshot for this store instance with conditional ETag check.
    pub async fn fetch_entitlements(
        &self,
        cached_etag: Option<&str>,
    ) -> Result<EntitlementsFetchResult, SpaceConnectorError> {
        let url = format!(
            "{}/space/integration/v1/instances/{}/entitlements",
            self.config.space_api_url.trim_end_matches('/'),
            self.config.instance_ref
        );

        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let sig = sign_space_webhook(self.config.webhook_secret.as_bytes(), timestamp, b"");
        let mut req = self.client.get(&url).header("x-space-signature", sig);
        if let Some(etag) = cached_etag {
            req = req.header(IF_NONE_MATCH, etag);
        }

        let response = req.send().await?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            let etag = response
                .headers()
                .get(ETAG)
                .and_then(|h| h.to_str().ok())
                .unwrap_or(cached_etag.unwrap_or_default())
                .to_string();
            return Ok(EntitlementsFetchResult::NotModified { etag });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(SpaceConnectorError::ApiError {
                status,
                title: "Failed to fetch entitlements".to_string(),
                detail: text,
            });
        }

        let snapshot: EntitlementsSnapshot = response.json().await?;
        Ok(EntitlementsFetchResult::Fresh(snapshot))
    }

    /// Report a batch of usage events to Space for metering and billing.
    pub async fn report_usage(
        &self,
        events: Vec<UsageEventRecord>,
    ) -> Result<UsageBatchResponse, SpaceConnectorError> {
        let url = format!(
            "{}/space/integration/v1/usage-events",
            self.config.space_api_url.trim_end_matches('/')
        );

        let input = UsageEventBatchInput { events };
        let body_bytes = serde_json::to_vec(&input)?;
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let sig = sign_space_webhook(self.config.webhook_secret.as_bytes(), timestamp, &body_bytes);

        let response = self.client.post(&url)
            .header("x-space-signature", sig)
            .header("content-type", "application/json")
            .body(body_bytes)
            .send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(SpaceConnectorError::ApiError {
                status,
                title: "Failed to ingest usage events".to_string(),
                detail: text,
            });
        }

        let res: UsageBatchResponse = response.json().await?;
        Ok(res)
    }

    /// Validate the connector manifest against Space.
    pub async fn validate_manifest(
        &self,
        capabilities: Vec<ManifestCapabilityInput>,
    ) -> Result<ValidateManifestResponse, SpaceConnectorError> {
        let url = format!(
            "{}/space/integration/v1/manifest/validate",
            self.config.space_api_url.trim_end_matches('/')
        );

        let input = ValidateManifestInput {
            product_key: "vendryx".to_string(),
            connector_kind: "stheno_commerce".to_string(),
            manifest_version: "1.0.0".to_string(),
            capabilities,
        };
        let body_bytes = serde_json::to_vec(&input)?;
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let sig = sign_space_webhook(self.config.webhook_secret.as_bytes(), timestamp, &body_bytes);

        let response = self.client.post(&url)
            .header("x-space-signature", sig)
            .header("content-type", "application/json")
            .body(body_bytes)
            .send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(SpaceConnectorError::ApiError {
                status,
                title: "Failed to validate manifest".to_string(),
                detail: text,
            });
        }

        let res: ValidateManifestResponse = response.json().await?;
        Ok(res)
    }

    /// Verify an inbound webhook from Space.
    pub fn verify_webhook(
        &self,
        timestamp: u64,
        signature_header: &str,
        payload: &[u8],
        current_time: u64,
    ) -> Result<SpaceWebhookPayload, WebhookVerifyError> {
        verify_space_webhook(
            self.config.webhook_secret.as_bytes(),
            timestamp,
            signature_header,
            payload,
            self.config.tolerance_seconds,
            current_time,
        )?;

        serde_json::from_slice(payload)
            .map_err(|e| WebhookVerifyError::MalformedPayload(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_sign_and_verify_webhook() {
        let secret = b"my_super_secret_webhook_key";
        let timestamp = 1725800000u64;
        let payload = br#"{"event":"instance.suspend","store_id":"ten_demo"}"#;

        let sig_header = sign_space_webhook(secret, timestamp, payload);
        assert!(sig_header.starts_with("t=1725800000,v1="));

        // Valid verification at exact time
        let verify_result = verify_space_webhook(secret, timestamp, &sig_header, payload, 300, timestamp + 5);
        assert!(verify_result.is_ok());

        // Replay attack / timestamp expired
        let expired_result = verify_space_webhook(secret, timestamp, &sig_header, payload, 300, timestamp + 301);
        assert!(matches!(expired_result, Err(WebhookVerifyError::TimestampExpired { .. })));

        // Tampered payload
        let tampered_payload = br#"{"event":"instance.suspend","store_id":"ten_hacked"}"#;
        let tampered_result = verify_space_webhook(secret, timestamp, &sig_header, tampered_payload, 300, timestamp + 5);
        assert_eq!(tampered_result, Err(WebhookVerifyError::SignatureMismatch));
    }

    #[tokio::test]
    async fn test_report_observed_state_mock() {
        let mock_server = MockServer::start().await;

        let expected_response = serde_json::json!({
            "instance": {
                "id": "inst_01",
                "external_reference": "prod_inst_vendryx_default",
                "observed_state": "ready",
                "observed_revision": 42
            },
            "drift_status": "in_sync"
        });

        Mock::given(method("POST"))
            .and(path("/space/integration/v1/instances/prod_inst_vendryx_default/observed-state"))
            .respond_with(ResponseTemplate::new(200).set_body_json(expected_response))
            .mount(&mock_server)
            .await;

        let config = SpaceConnectorConfig {
            space_api_url: mock_server.uri(),
            instance_ref: "prod_inst_vendryx_default".to_string(),
            webhook_secret: "secret".to_string(),
            tolerance_seconds: 300,
        };

        let client = SpaceClient::new(config);
        let resp = client.report_observed_state("ready", 42).await.expect("successful report");
        assert_eq!(resp.drift_status, "in_sync");
        assert_eq!(resp.instance["observed_state"], "ready");
    }

    #[tokio::test]
    async fn test_fetch_entitlements_conditional_etag() {
        let mock_server = MockServer::start().await;

        // 1. Fresh entitlements
        let snapshot = serde_json::json!({
            "instance_ref": "prod_inst_vendryx_default",
            "etag": "\"rev-42-1725800000\"",
            "revision": 42,
            "issued_at_epoch_sec": 1725800000,
            "grants": {
                "custom_domains": 5,
                "api_concurrency": 100
            }
        });

        Mock::given(method("GET"))
            .and(path("/space/integration/v1/instances/prod_inst_vendryx_default/entitlements"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("ETag", "\"rev-42-1725800000\"")
                    .set_body_json(snapshot),
            )
            .mount(&mock_server)
            .await;

        let config = SpaceConnectorConfig {
            space_api_url: mock_server.uri(),
            instance_ref: "prod_inst_vendryx_default".to_string(),
            webhook_secret: "secret".to_string(),
            tolerance_seconds: 300,
        };

        let client = SpaceClient::new(config.clone());
        let res = client.fetch_entitlements(None).await.expect("fresh snapshot");
        match res {
            EntitlementsFetchResult::Fresh(s) => {
                assert_eq!(s.revision, 42);
                assert_eq!(s.etag, "\"rev-42-1725800000\"");
            }
            EntitlementsFetchResult::NotModified { .. } => panic!("expected fresh snapshot"),
        }

        // 2. 304 Not Modified
        let mock_server2 = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/space/integration/v1/instances/prod_inst_vendryx_default/entitlements"))
            .and(header("If-None-Match", "\"rev-42-1725800000\""))
            .respond_with(
                ResponseTemplate::new(304).insert_header("ETag", "\"rev-42-1725800000\""),
            )
            .mount(&mock_server2)
            .await;

        let client2 = SpaceClient::new(SpaceConnectorConfig {
            space_api_url: mock_server2.uri(),
            ..config
        });

        let res304 = client2
            .fetch_entitlements(Some("\"rev-42-1725800000\""))
            .await
            .expect("304 response");
        assert_eq!(
            res304,
            EntitlementsFetchResult::NotModified {
                etag: "\"rev-42-1725800000\"".to_string()
            }
        );
    }

    #[tokio::test]
    async fn test_report_usage_batch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/space/integration/v1/usage-events"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "accepted": 2,
                    "deduplicated": 1
                })),
            )
            .mount(&mock_server)
            .await;

        let client = SpaceClient::new(SpaceConnectorConfig {
            space_api_url: mock_server.uri(),
            ..SpaceConnectorConfig::default()
        });

        let events = vec![
            UsageEventRecord {
                id: "ue_1".to_string(),
                instance_id: "inst_1".to_string(),
                metric: "api_requests".to_string(),
                quantity: 10,
                idempotency_key: "idem_1".to_string(),
                timestamp_epoch_sec: 1725800000,
            },
            UsageEventRecord {
                id: "ue_2".to_string(),
                instance_id: "inst_1".to_string(),
                metric: "api_requests".to_string(),
                quantity: 5,
                idempotency_key: "idem_2".to_string(),
                timestamp_epoch_sec: 1725800005,
            },
        ];

        let res = client.report_usage(events).await.expect("usage reported");
        assert_eq!(res.accepted, 2);
        assert_eq!(res.deduplicated, 1);
    }
}
