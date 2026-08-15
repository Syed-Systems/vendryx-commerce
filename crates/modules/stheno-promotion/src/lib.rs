//! `stheno-promotion`: Promotions, campaigns, rules, budgets, and discount engine (P4-08 through P4-15).

pub mod dto;
pub mod models;
pub mod service;

pub use dto::*;
pub use service::{constant_time_eq_str, PromotionService};
