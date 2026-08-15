use std::sync::Arc;

use async_trait::async_trait;
use stheno_events::Event;
use stheno_notification::{NotificationService, SendNotificationInput};
use stheno_types::Result;

use crate::EventSubscriber;

pub struct PushOrderPlacedSubscriber {
    notification_service: Arc<NotificationService>,
    owner_push_subscription: String,
}

impl PushOrderPlacedSubscriber {
    pub fn new(
        notification_service: Arc<NotificationService>,
        owner_push_subscription: impl Into<String>,
    ) -> Self {
        Self {
            notification_service,
            owner_push_subscription: owner_push_subscription.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for PushOrderPlacedSubscriber {
    fn event_name(&self) -> &'static str {
        "order.placed"
    }

    async fn handle(&self, event: &Event) -> Result<()> {
        let payload = &event.payload;

        // Check if metadata.source == "social" (PRODUCT §5.5 trap 3)
        let is_social = payload
            .get("metadata")
            .and_then(|m| m.get("source"))
            .and_then(|s| s.as_str())
            .map(|s| s == "social")
            .unwrap_or(false)
            || payload
                .get("source")
                .and_then(|s| s.as_str())
                .map(|s| s == "social")
                .unwrap_or(false);

        if is_social {
            tracing::info!(
                order_id = ?payload.get("order_id"),
                "Suppressed web push notification for social order per PRODUCT §5.5 trap 3"
            );
            return Ok(());
        }

        let order_id = payload
            .get("order_id")
            .or_else(|| payload.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let total = payload
            .get("total")
            .or_else(|| payload.get("total_amount"))
            .and_then(|v| v.as_str())
            .unwrap_or("$0.00");

        let _ = self
            .notification_service
            .send_notification(SendNotificationInput {
                to: self.owner_push_subscription.clone(),
                channel: "push".to_string(),
                template: None,
                subject: Some(format!("New Order Placed: #{}", order_id)),
                body: Some(format!("You have a new order #{} for {}", order_id, total)),
                data: Some(payload.clone()),
                trigger_type: Some("order.placed".to_string()),
                resource_id: Some(order_id.to_string()),
                resource_type: Some("order".to_string()),
                receiver_id: Some("owner".to_string()),
            })
            .await?;

        Ok(())
    }
}
