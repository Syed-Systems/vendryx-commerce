//! `stheno-platform`: the tenant registry (ADR-012).
//!
//! This module is what makes one deployment serve many merchants. It owns the `platform`
//! schema — tenants, their domains, their subscriptions, and what each plan means on this
//! deployment (ADR-016) — and it is deliberately the one part of the system that is
//! **not** tenant-scoped: it is the thing that answers "which tenant is this?", so it
//! cannot itself be filtered by the answer.
//!
//! Everything here runs under [`stheno_db::tenant::PLATFORM_SCOPE`] rather than a
//! merchant's tenant. The `platform` tables carry no RLS policy, so that scope exists
//! only to satisfy the connection helper, not to filter anything.

pub mod calculator;
pub mod dto;
pub mod models;
pub mod service;

// Callers get DTOs. The entities in `models` stay a persistence detail (AGENTS.md §3.4):
// they carry `deleted_at`, they name their table, and nothing outside this crate should
// have to know either fact.
pub use dto::*;
pub use service::PlatformService;
