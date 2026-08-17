//! Module `stheno-search`: Full-text search engine, provider routing, and event-driven index synchronization.

pub mod dto;
pub(crate) mod models;
pub mod provider;
pub mod service;

pub use dto::*;
pub use provider::SearchProvider;
pub use service::SearchService;
