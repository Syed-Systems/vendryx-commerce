//! Provider `stheno-notification-resend` for `stheno-notification`.
//!
//! Delivers transactional emails (order confirmations, customer welcomes,
//! password resets, staff invitations) via the Resend API (https://api.resend.com/emails).

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use stheno_notification::{NotificationProvider, NotificationProviderResult};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendEmailRecord {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
}

pub struct ResendNotificationProvider {
    config: ResendConfig,
    client: reqwest::Client,
    sent_records: Arc<RwLock<Vec<ResendEmailRecord>>>,
}

impl ResendNotificationProvider {
    pub fn new(config: ResendConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            sent_records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_client(config: ResendConfig, client: reqwest::Client) -> Self {
        Self {
            config,
            client,
            sent_records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_sent_records(&self) -> Vec<ResendEmailRecord> {
        self.sent_records.read().unwrap().clone()
    }

    pub fn config(&self) -> &ResendConfig {
        &self.config
    }

    fn formatted_from(&self) -> String {
        match &self.config.from_name {
            Some(name) => format!("{} <{}>", name, self.config.from_email),
            None => self.config.from_email.clone(),
        }
    }
}

#[async_trait]
impl NotificationProvider for ResendNotificationProvider {
    fn provider_id(&self) -> &'static str {
        "resend"
    }

    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["email"]
    }

    fn supports_idempotency(&self) -> bool {
        true
    }

    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult> {
        self.send_idempotent(to, channel, subject, body, data, None)
            .await
    }

    async fn send_idempotent(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
        idempotency_key: Option<&str>,
    ) -> Result<NotificationProviderResult> {
        if channel != "email" {
            return Err(Error::Invalid {
                code: "unsupported_channel",
                message: format!("Resend provider does not support channel '{channel}'"),
                field: Some("channel".to_string()),
            });
        }

        if !to.contains('@') {
            return Err(Error::Invalid {
                code: "invalid_email_recipient",
                message: format!("Invalid email recipient '{to}'"),
                field: Some("to".to_string()),
            });
        }

        let subject_str = subject.unwrap_or("Notification").to_string();
        let from_formatted = self.formatted_from();

        let record = ResendEmailRecord {
            to: to.to_string(),
            from: from_formatted.clone(),
            subject: subject_str.clone(),
            body: body.to_string(),
            data: data.cloned(),
        };

        {
            let mut list = self.sent_records.write().unwrap();
            list.push(record);
        }

        // In development or testing without an active API key or base URL override,
        // record the send and return a synthetic Resend identifier.
        if self.config.api_key.is_empty() && self.config.base_url.is_none() {
            let message_id = format!("re_mock_{}", ulid::Ulid::new());
            let mut meta = serde_json::Map::new();
            meta.insert("provider".to_string(), json!("resend"));
            meta.insert("message_id".to_string(), json!(&message_id));
            meta.insert("dry_run".to_string(), json!(true));

            return Ok(NotificationProviderResult {
                provider_message_id: Some(message_id),
                metadata: Some(serde_json::Value::Object(meta)),
            });
        }

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.resend.com");
        let endpoint = format!("{}/emails", base_url.trim_end_matches('/'));

        let payload = json!({
            "from": from_formatted,
            "to": [to],
            "subject": subject_str,
            "html": body,
            "text": body,
        });

        let mut request = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&payload);
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        let resp = request.send().await.map_err(|e| Error::ProviderFailed {
            provider: "resend".to_string(),
            source: format!("Network error connecting to Resend: {e}").into(),
        })?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(Error::ProviderFailed {
                provider: "resend".to_string(),
                source: format!("Resend API error (status {status}): {err_text}").into(),
            });
        }

        let resp_json: serde_json::Value =
            resp.json().await.map_err(|e| Error::ProviderFailed {
                provider: "resend".to_string(),
                source: format!("Failed to parse Resend response JSON: {e}").into(),
            })?;

        let message_id = resp_json
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("re_{}", ulid::Ulid::new()));

        let mut meta = serde_json::Map::new();
        meta.insert("provider".to_string(), json!("resend"));
        meta.insert("message_id".to_string(), json!(&message_id));
        if let Some(created_at) = resp_json.get("created_at") {
            meta.insert("created_at".to_string(), created_at.clone());
        }

        Ok(NotificationProviderResult {
            provider_message_id: Some(message_id),
            metadata: Some(serde_json::Value::Object(meta)),
        })
    }
}
