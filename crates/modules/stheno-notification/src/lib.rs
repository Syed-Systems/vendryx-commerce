//! Module `stheno-notification`: Notification templates, channels, delivery tracking, and provider routing.

pub mod dto;
pub(crate) mod models;
pub mod provider;
pub mod service;

pub use dto::*;
pub use provider::{NotificationProvider, NotificationProviderResult};
pub use service::{
    DailyNotificationChannelUsage, DailyNotificationUsage, NotificationQuotaPolicy,
    NotificationService, WhatsAppConsentChecker,
};
