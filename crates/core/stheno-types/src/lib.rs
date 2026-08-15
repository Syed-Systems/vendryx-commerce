//! `stheno-types`: Core types for Stheno Commerce Engine.
//! Contains Money, Id<T>, CurrencyCode, Entitlements, and universal error definitions.
//! This crate depends on no other Stheno crates.

pub mod currency;
pub mod entitlements;
pub mod error;
pub mod id;
pub mod money;

pub use currency::CurrencyCode;
pub use entitlements::{Entitlements, Feature, PlanTier};
pub use error::Error;
pub use id::Id;
pub use money::Money;

pub type Result<T> = std::result::Result<T, Error>;
