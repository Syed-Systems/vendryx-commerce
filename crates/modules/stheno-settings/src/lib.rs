//! Module `stheno-settings`: Store settings, custom view configurations, property labels, and operator layout preferences.

pub mod dto;
pub(crate) mod models;
pub mod service;

pub use dto::*;
pub use service::SettingsService;
