//! Provider `stheno-notification-webpush` for `stheno-notification` (VAPID Web Push)

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stheno_notification::{NotificationProvider, NotificationProviderResult};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapidConfig {
    pub subject: String,
    pub public_key: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushSubscription {
    pub endpoint: String,
    pub keys: WebPushKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushRecord {
    pub endpoint: String,
    pub title: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
    pub vapid_subject: String,
}

pub struct WebPushNotificationProvider {
    config: VapidConfig,
    dispatched_pushes: Arc<RwLock<Vec<WebPushRecord>>>,
}

impl WebPushNotificationProvider {
    pub fn new(config: VapidConfig) -> Self {
        Self {
            config,
            dispatched_pushes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_dispatched_pushes(&self) -> Vec<WebPushRecord> {
        self.dispatched_pushes.read().unwrap().clone()
    }

    pub fn config(&self) -> &VapidConfig {
        &self.config
    }
}

#[async_trait]
impl NotificationProvider for WebPushNotificationProvider {
    fn provider_id(&self) -> &'static str {
        "webpush"
    }

    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["push"]
    }

    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult> {
        if channel != "push" {
            return Err(Error::Invalid {
                code: "unsupported_channel",
                message: format!("WebPush provider does not support channel '{}'", channel),
                field: Some("channel".to_string()),
            });
        }

        // `to` can be a push subscription endpoint URL or JSON subscription object
        let endpoint = if to.starts_with('{') {
            let sub: WebPushSubscription =
                serde_json::from_str(to).map_err(|e| Error::Invalid {
                    code: "invalid_push_subscription",
                    message: format!("Invalid JSON push subscription: {}", e),
                    field: Some("to".to_string()),
                })?;
            sub.endpoint
        } else {
            to.to_string()
        };

        if !endpoint.starts_with("https://") && !endpoint.starts_with("http://") {
            return Err(Error::Invalid {
                code: "invalid_push_endpoint",
                message: format!("Invalid push endpoint URI: {}", endpoint),
                field: Some("to".to_string()),
            });
        }

        let title = subject.unwrap_or("New Notification").to_string();
        let record = WebPushRecord {
            endpoint,
            title,
            body: body.to_string(),
            data: data.cloned(),
            vapid_subject: self.config.subject.clone(),
        };

        {
            let mut list = self.dispatched_pushes.write().unwrap();
            list.push(record);
        }

        let message_id = format!("wp_{}", ulid::Ulid::new());
        let mut meta = serde_json::Map::new();
        meta.insert("provider".to_string(), serde_json::json!("webpush"));
        meta.insert(
            "vapid_key".to_string(),
            serde_json::json!(self.config.public_key),
        );

        Ok(NotificationProviderResult {
            provider_message_id: Some(message_id),
            metadata: Some(serde_json::Value::Object(meta)),
        })
    }
}
