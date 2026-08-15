//! Module `stheno-search`: Full-text search engine, provider routing, and event-driven index synchronization.

pub mod models;
pub mod provider;
pub mod service;

pub use models::*;
pub use provider::SearchProvider;
pub use service::SearchService;
