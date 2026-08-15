//! Provider `stheno-notification-local` for `stheno-notification`

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stheno_notification::{NotificationProvider, NotificationProviderResult};
use stheno_types::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentMessage {
    pub to: String,
    pub channel: String,
    pub subject: Option<String>,
    pub body: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Default)]
pub struct LocalNotificationProvider {
    sent_messages: Arc<RwLock<Vec<SentMessage>>>,
}

impl LocalNotificationProvider {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_sent_messages(&self) -> Vec<SentMessage> {
        self.sent_messages.read().unwrap().clone()
    }

    pub fn clear(&self) {
        self.sent_messages.write().unwrap().clear();
    }
}

#[async_trait]
impl NotificationProvider for LocalNotificationProvider {
    fn provider_id(&self) -> &'static str {
        "local"
    }

    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["email", "push", "sms"]
    }

    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult> {
        let msg = SentMessage {
            to: to.to_string(),
            channel: channel.to_string(),
            subject: subject.map(|s| s.to_string()),
            body: body.to_string(),
            data: data.cloned(),
        };

        {
            let mut list = self.sent_messages.write().unwrap();
            list.push(msg);
        }

        let mut meta = serde_json::Map::new();
        meta.insert("provider".to_string(), serde_json::json!("local"));
        meta.insert("channel".to_string(), serde_json::json!(channel));

        Ok(NotificationProviderResult {
            provider_message_id: Some(format!("local_{}", ulid::Ulid::new())),
            metadata: Some(serde_json::Value::Object(meta)),
        })
    }
}
