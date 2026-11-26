//! Database wiring for the server boot path (P11-03).
//!
//! Phase 1 defined `create_pool` in `stheno-db` and never called it from
//! anywhere. Boot now reads `DATABASE_URL`, builds the pool, and refuses to
//! start without one -- a server that silently boots on in-memory storage is
//! the exact failure mode Phase 2 exists to remove.

use std::time::Duration;
use stheno_db::{DbPool, create_pool};
use stheno_types::Error;

/// Default pool size, overridable per deployment via `STHENO_DB_POOL_SIZE`.
///
/// The right number depends on the `max_connections` the deployment's Postgres
/// was given, which nothing in this process can know.
pub const DEFAULT_POOL_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
    pub pool_size: usize,
}

impl DbConfig {
    /// Reads database configuration from the environment.
    pub fn from_env() -> Result<Self, Error> {
        Self::from_values(
            std::env::var("DATABASE_URL").ok().as_deref(),
            std::env::var("STHENO_DB_POOL_SIZE").ok().as_deref(),
        )
    }

    /// Parses configuration from already-read values.
    ///
    /// Kept separate from `from_env` so it can be tested without mutating
    /// process environment -- `set_var` is global, and tests that write it race
    /// every other test that reads it.
    ///
    /// A missing `DATABASE_URL` is an error, never a fallback: booting without
    /// one is how a deployment loses every product on restart.
    pub fn from_values(database_url: Option<&str>, pool_size: Option<&str>) -> Result<Self, Error> {
        let database_url = database_url.ok_or(Error::Invalid {
            code: "database_url_missing",
            message: "DATABASE_URL is not set; the server has no durable storage to boot against"
                .to_string(),
            field: Some("DATABASE_URL".to_string()),
        })?;

        if database_url.trim().is_empty() {
            return Err(Error::Invalid {
                code: "database_url_empty",
                message: "DATABASE_URL is set but empty".to_string(),
                field: Some("DATABASE_URL".to_string()),
            });
        }

        let pool_size = match pool_size {
            None => DEFAULT_POOL_SIZE,
            Some(raw) => {
                raw.trim()
                    .parse::<usize>()
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or(Error::Invalid {
                        code: "invalid_pool_size",
                        message: format!(
                            "STHENO_DB_POOL_SIZE must be a positive integer, got '{raw}'"
                        ),
                        field: Some("STHENO_DB_POOL_SIZE".to_string()),
                    })?
            }
        };

        Ok(Self {
            database_url: database_url.to_string(),
            pool_size,
        })
    }

    pub fn build_pool(&self) -> Result<DbPool, Error> {
        create_pool(&self.database_url, self.pool_size)
    }
}

/// Pool saturation, surfaced on `/health` so a deployment running out of
/// connections is visible before it starts timing out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolHealth {
    pub size: usize,
    pub available: usize,
    pub max_size: usize,
}

impl PoolHealth {
    pub fn of(pool: &DbPool) -> Self {
        let status = pool.status();
        Self {
            size: status.size,
            available: status.available,
            max_size: status.max_size,
        }
    }

    /// Fraction of the pool currently checked out, in `0.0..=1.0`.
    ///
    /// `size` is how many connections the pool has actually created and
    /// `available` how many of those are idle, so connections in use are the
    /// difference. Deriving it from `max_size` instead would report a freshly
    /// built pool -- which has created nothing yet -- as fully saturated.
    pub fn saturation(&self) -> f64 {
        if self.max_size == 0 {
            return 1.0;
        }
        let in_use = self.size.saturating_sub(self.available);
        in_use as f64 / self.max_size as f64
    }
}

/// Verifies the pool can actually reach the database.
///
/// `create_pool` is lazy: it validates the URL and returns without dialling, so
/// a boot that only builds a pool has proven nothing about connectivity.
pub async fn check_connectivity(pool: &DbPool, timeout: Duration) -> Result<(), Error> {
    let conn = tokio::time::timeout(timeout, pool.get())
        .await
        .map_err(|_| Error::Database(format!("database unreachable after {timeout:?}")))?
        .map_err(|e| Error::Database(e.to_string()))?;
    // Returned to the pool immediately; holding it would starve the caller.
    drop(conn);
    Ok(())
}
