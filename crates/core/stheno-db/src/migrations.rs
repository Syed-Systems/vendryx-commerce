//! Embedded migration runner (P11-05, P11-06).
//!
//! Migrations live in one directory at the repository root, named
//! `<timestamp>_<module>_<name>`. They are embedded in the binary, so a
//! deployed image can migrate itself with no SQL files alongside it.
//!
//! # Why one directory and not one per module
//!
//! Ordering is global. `order` references tables that `product` and `cart`
//! create, and link tables reference both. A per-module directory cannot express
//! "after product, before order" without a second ordering mechanism on top, so
//! the module lives in the directory *name* and the timestamp does the ordering.
//!
//! # Why the connection is wrapped
//!
//! `diesel_migrations` drives a synchronous `Connection`, and this workspace
//! builds diesel with `postgres_backend` rather than `postgres` -- there is no
//! libpq and therefore no sync `PgConnection`. `AsyncConnectionWrapper` presents
//! the async connection through the sync trait, run on a blocking thread so it
//! never stalls the reactor.

use diesel::migration::MigrationVersion;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use stheno_types::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../../migrations");

/// Names of migrations that exist in the binary but are absent from the
/// database. Empty means the schema matches.
pub async fn pending(database_url: &str) -> Result<Vec<String>, Error> {
    with_sync_conn(database_url, |conn| {
        let versions = conn
            .pending_migrations(MIGRATIONS)
            .map_err(|e| Error::Database(e.to_string()))?
            .iter()
            .map(|m| m.name().to_string())
            .collect();
        Ok(versions)
    })
    .await
}

/// Applies every pending migration in order, returning the ones applied.
pub async fn run(database_url: &str) -> Result<Vec<String>, Error> {
    with_sync_conn(database_url, |conn| {
        let applied: Vec<MigrationVersion> = conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(applied.iter().map(|v| v.to_string()).collect())
    })
    .await
}

/// Reverts the most recent migration. Used by the drift test to prove the
/// runner detects a schema that is behind the binary.
pub async fn revert_last(database_url: &str) -> Result<String, Error> {
    with_sync_conn(database_url, |conn| {
        conn.revert_last_migration(MIGRATIONS)
            .map(|v| v.to_string())
            .map_err(|e| Error::Database(e.to_string()))
    })
    .await
}

/// Opens a dedicated connection, hands it to `f` through the synchronous
/// harness on a blocking thread, and returns the result.
///
/// Deliberately not a pooled connection: migrations take schema locks and can
/// run for a long time, so borrowing a pool slot for the duration would starve
/// the pool exactly when the deployment is least able to tolerate it.
async fn with_sync_conn<T, F>(database_url: &str, f: F) -> Result<T, Error>
where
    T: Send + 'static,
    F: FnOnce(&mut AsyncConnectionWrapper<AsyncPgConnection>) -> Result<T, Error> + Send + 'static,
{
    let conn = AsyncPgConnection::establish(database_url)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    tokio::task::spawn_blocking(move || {
        let mut wrapper = AsyncConnectionWrapper::<AsyncPgConnection>::from(conn);
        f(&mut wrapper)
    })
    .await
    .map_err(|e| Error::Database(format!("migration task panicked: {e}")))?
}
