use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDTO {
    pub id: String,
    pub to: String,
    pub channel: String,
    pub template: Option<String>,
    pub data: Option<serde_json::Value>,
    pub trigger_type: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub receiver_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplateDTO {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDeliveryDTO {
    pub id: String,
    pub notification_id: String,
    pub provider_id: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendNotificationInput {
    pub to: String,
    pub channel: String,
    pub template: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub data: Option<serde_json::Value>,
    pub trigger_type: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub receiver_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateInput {
    pub name: String,
    pub channel: String,
    pub subject_template: Option<String>,
    pub body_template: String,
}
