//! Provider `stheno-notification-sendgrid` for `stheno-notification`

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stheno_notification::{NotificationProvider, NotificationProviderResult};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendGridConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendGridEmailRecord {
    pub to: String,
    pub from_email: String,
    pub from_name: Option<String>,
    pub subject: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
}

pub struct SendGridNotificationProvider {
    config: SendGridConfig,
    sent_records: Arc<RwLock<Vec<SendGridEmailRecord>>>,
}

impl SendGridNotificationProvider {
    pub fn new(config: SendGridConfig) -> Self {
        Self {
            config,
            sent_records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_sent_records(&self) -> Vec<SendGridEmailRecord> {
        self.sent_records.read().unwrap().clone()
    }

    pub fn config(&self) -> &SendGridConfig {
        &self.config
    }
}

#[async_trait]
impl NotificationProvider for SendGridNotificationProvider {
    fn provider_id(&self) -> &'static str {
        "sendgrid"
    }

    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["email"]
    }

    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult> {
        if channel != "email" {
            return Err(Error::Invalid {
                code: "unsupported_channel",
                message: format!("SendGrid provider does not support channel '{channel}'"),
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

        let subject_str = subject.unwrap_or("No Subject").to_string();
        let record = SendGridEmailRecord {
            to: to.to_string(),
            from_email: self.config.from_email.clone(),
            from_name: self.config.from_name.clone(),
            subject: subject_str,
            body: body.to_string(),
            data: data.cloned(),
        };

        {
            let mut list = self.sent_records.write().unwrap();
            list.push(record);
        }

        let message_id = format!("sg_{}", ulid::Ulid::new());
        let mut meta = serde_json::Map::new();
        meta.insert("provider".to_string(), serde_json::json!("sendgrid"));
        meta.insert("message_id".to_string(), serde_json::json!(message_id));

        Ok(NotificationProviderResult {
            provider_message_id: Some(message_id),
            metadata: Some(serde_json::Value::Object(meta)),
        })
    }
}
