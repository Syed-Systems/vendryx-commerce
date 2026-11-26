use async_trait::async_trait;
use stheno_types::Result;

#[derive(Debug, Clone)]
pub struct NotificationProviderResult {
    pub provider_message_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// Keep this compatibility trait's established send shape explicit for provider implementers.
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn supported_channels(&self) -> Vec<&'static str>;
    /// Returns true only when retries with the same key suppress duplicate provider effects.
    fn supports_idempotency(&self) -> bool {
        false
    }
    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult>;

    /// Sends with a stable key when the provider supports deduplicating retries.
    /// Providers without native support retain their existing behavior.
    async fn send_idempotent(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
        _idempotency_key: Option<&str>,
    ) -> Result<NotificationProviderResult> {
        self.send(to, channel, subject, body, data).await
    }

    /// Sends for the tenant recorded on a durable intent. Providers with tenant-owned sender
    /// credentials override this method and resolve credentials from the trusted tenant ID.
    async fn send_for_tenant(
        &self,
        tenant_id: &str,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        data: Option<&serde_json::Value>,
        idempotency_key: Option<&str>,
    ) -> Result<NotificationProviderResult> {
        let _ = tenant_id;
        self.send_idempotent(to, channel, subject, body, data, idempotency_key)
            .await
    }
}
