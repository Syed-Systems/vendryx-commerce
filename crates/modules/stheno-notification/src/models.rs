use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for Notification {
    const TABLE: &'static str = "notification.notification";
    const ENTITY: &'static str = "notification";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotificationTemplate {
    pub id: Id<NotificationTemplate>,
    pub name: String,
    pub channel: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for NotificationTemplate {
    const TABLE: &'static str = "notification.notification_template";
    const ENTITY: &'static str = "notification_template";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NotificationDelivery {
    pub id: Id<NotificationDelivery>,
    pub notification_id: String,
    pub provider_id: String,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CrudEntity for NotificationDelivery {
    const TABLE: &'static str = "notification.notification_delivery";
    const ENTITY: &'static str = "notification_delivery";

    fn id(&self) -> String {
        self.id.to_string()
    }
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    Notification => "noti",
    NotificationDelivery => "notidel",
    NotificationTemplate => "notitpl",
}
