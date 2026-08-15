//! Module `stheno-index`: Denormalized cross-module projection engine, incremental index sync, and filter/sort resolution.

pub mod dto;
pub(crate) mod models;
pub mod service;

pub use dto::*;
pub use service::IndexService;
