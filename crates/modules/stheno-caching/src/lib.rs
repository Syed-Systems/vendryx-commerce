//! Module `stheno-caching`: Cache abstraction, Redis cache, Postgres fallback, and outage degradation resilience (P14-15).

pub mod in_process;
pub mod models;
pub mod postgres;
pub mod provider;
pub mod service;

pub use in_process::InMemoryCacheProvider;
pub use postgres::PostgresCacheProvider;
pub use provider::CacheProvider;
pub use service::CacheService;
