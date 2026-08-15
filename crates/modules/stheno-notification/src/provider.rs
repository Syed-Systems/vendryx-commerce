use async_trait::async_trait;
use stheno_types::Result;

#[derive(Debug, Clone)]
pub struct NotificationProviderResult {
    pub provider_message_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn supported_channels(&self) -> Vec<&'static str>;
    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult>;
}
