//! `stheno-tax`: Tax regions, rates, rules, and providers (P3-12 to P3-15).

pub mod dto;
pub mod models;
pub mod service;

pub use dto::*;
pub use service::{SystemTaxProvider, TaxProviderTrait, TaxService};
