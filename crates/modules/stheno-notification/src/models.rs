use chrono::{DateTime, Utc};
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Notification {
    pub id: Id<Notification>,
    pub to: String,
    pub channel: String, // "email", "push", "sms"
    pub template: Option<String>,
    pub data: Option<serde_json::Value>,
    pub trigger_type: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub receiver_id: Option<String>,
    pub status: String, // "pending", "sent", "failed"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NotificationTemplate {
    pub id: Id<NotificationTemplate>,
    pub name: String,
    pub channel: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NotificationDelivery {
    pub id: Id<NotificationDelivery>,
    pub notification_id: String,
    pub provider_id: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
