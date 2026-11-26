use async_trait::async_trait;
use std::sync::Arc;
use stheno_notification::service::NotificationQuotaPolicy;
use stheno_platform::PlatformService;
use stheno_types::Result;

/// Free plans receive a small platform-funded daily email and WhatsApp allowance.
pub const FREE_DAILY_WHATSAPP_LIMIT: i32 = 300;
pub const FREE_DAILY_EMAIL_LIMIT: i32 = 200;

pub struct PlatformNotificationQuotaPolicy {
    platform: Arc<PlatformService>,
}

impl PlatformNotificationQuotaPolicy {
    pub fn new(platform: Arc<PlatformService>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl NotificationQuotaPolicy for PlatformNotificationQuotaPolicy {
    async fn daily_limit(&self, tenant_id: &str, channel: &str) -> Result<Option<i32>> {
        let subscription = self.platform.get_subscription(tenant_id).await?;
        if !subscription.tier.eq_ignore_ascii_case("free") {
            return Ok(None);
        }

        Ok(match channel {
            "whatsapp" => Some(FREE_DAILY_WHATSAPP_LIMIT),
            "email" => Some(FREE_DAILY_EMAIL_LIMIT),
            _ => None,
        })
    }
}
