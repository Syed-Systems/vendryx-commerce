//! Module `stheno-analytics`: Product analytics, event tracking, and PII-redacted provider synchronization (Local, PostHog).

pub mod dto;
pub(crate) mod models;
pub mod provider;
pub mod service;

pub use dto::*;
pub use provider::{
    AnalyticsProvider, LocalAnalyticsProvider, PostHogAnalyticsProvider, PostHogConfig,
    TrackedEvent,
};
pub use service::AnalyticsService;
