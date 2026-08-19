//! `stheno-platform`: the tenant registry (ADR-012).
//!
//! This module is what makes one deployment serve many merchants. It owns the `platform`
//! schema — tenants, their domains, and their subscriptions — and it is deliberately the
//! one part of the system that is **not** tenant-scoped: it is the thing that answers
//! "which tenant is this?", so it cannot itself be filtered by the answer.
//!
//! Everything here runs under [`stheno_db::tenant::PLATFORM_SCOPE`] rather than a
//! merchant's tenant. The `platform` tables carry no RLS policy, so that scope exists
//! only to satisfy the connection helper, not to filter anything.

mod models;
mod service;

pub use models::{Subscription, Tenant, TenantDomain};
pub use service::{NewTenant, PlatformService};
