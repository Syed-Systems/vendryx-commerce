//! Module `stheno-caching`: Cache abstraction, in-process memory cache, and outage degradation resilience.

pub mod in_process;
pub mod provider;
pub mod service;

pub use in_process::InMemoryCacheProvider;
pub use provider::CacheProvider;
pub use service::CacheService;
