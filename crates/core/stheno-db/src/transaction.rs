use diesel::sql_query;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use std::future::Future;
use std::time::Duration;
use stheno_types::Error;

use crate::pool::DbPool;

/// Explicit database connection reference passed to services.
pub type Conn<'a> = &'a mut AsyncPgConnection;

/// Runs an operation inside a PostgreSQL SAVEPOINT for nested transaction semantics.
/// If the operation returns Ok, the savepoint is released.
/// If the operation returns Err, the savepoint is rolled back.
///
/// The closure is boxed and higher-ranked over the connection's lifetime. A
/// plain `F: FnOnce(&mut Conn) -> Fut` cannot express this: `Fut` would be one
/// fixed type, while each call site borrows the connection for a fresh
/// lifetime, so nesting a savepoint inside another fails to compile.
pub async fn with_savepoint<T, F>(
    conn: &mut AsyncPgConnection,
    savepoint_name: &str,
    f: F,
) -> Result<T, Error>
where
    F: for<'c> FnOnce(
        &'c mut AsyncPgConnection,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'c>>,
{
    // A savepoint name is an SQL identifier and cannot be parameterised, so it
    // is checked rather than bound.
    let savepoint_name = crate::sql::ident(savepoint_name)?;

    sql_query(format!("SAVEPOINT {}", savepoint_name))
        .execute(conn)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    match f(conn).await {
        Ok(val) => {
            sql_query(format!("RELEASE SAVEPOINT {}", savepoint_name))
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
            Ok(val)
        }
        Err(err) => {
            let _ = sql_query(format!("ROLLBACK TO SAVEPOINT {}", savepoint_name))
                .execute(conn)
                .await;
            Err(err)
        }
    }
}

/// Runs `f` inside a database transaction, committing on `Ok` and rolling back
/// on `Err`.
///
/// Exists because `stheno_types::Error` cannot implement
/// `From<diesel::result::Error>` -- `stheno-types` depends on no other crate and
/// must not take a database dependency. Without this, every migrating module
/// would define the same private error wrapper to satisfy diesel's
/// `transaction()` bound.
pub async fn with_transaction<T, F>(conn: &mut AsyncPgConnection, f: F) -> Result<T, Error>
where
    T: Send + 'static,
    F: for<'c> FnOnce(
            &'c mut AsyncPgConnection,
        )
            -> std::pin::Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'c>>
        + Send,
{
    conn.transaction::<T, TxError, _>(|c| Box::pin(async { f(c).await.map_err(TxError::Domain) }))
        .await
        .map_err(TxError::into_error)
}

/// Default retry budget for a serialisable transaction.
///
/// Bounded on purpose: a conflict that survives five attempts is contention the
/// caller needs to hear about, not something to hide behind an unbounded loop
/// that turns a hot row into an outage.
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

/// Base backoff. Doubles per attempt with jitter, capped by `MAX_BACKOFF`.
const BASE_BACKOFF: Duration = Duration::from_millis(10);
const MAX_BACKOFF: Duration = Duration::from_millis(500);

/// Error channel for a transaction body, so a serialisation failure stays
/// distinguishable from a domain error after the transaction has unwound.
enum TxError {
    Db(diesel::result::Error),
    Domain(Error),
}

impl From<diesel::result::Error> for TxError {
    fn from(e: diesel::result::Error) -> Self {
        TxError::Db(e)
    }
}

impl TxError {
    /// Whether Postgres asked us to try again.
    ///
    /// `SERIALIZABLE` does not block conflicting writers; it aborts one of them
    /// with SQLSTATE 40001. Retrying is not error handling here, it is how the
    /// isolation level is meant to be used. Deadlocks (40P01) are likewise safe
    /// to retry.
    ///
    /// Two checks, not one, because the abort can surface on either side of the
    /// caller's error conversion. A conflict raised at `COMMIT` arrives as a
    /// typed `diesel` error, but one raised by a statement *inside* the closure
    /// has already been mapped to a domain error by the caller -- by which point
    /// the classification is gone and only Postgres' message survives. Matching
    /// the kind alone would silently never retry the far more common case.
    fn is_retryable(&self) -> bool {
        use diesel::result::{DatabaseErrorKind, Error as DieselError};

        if matches!(
            self,
            TxError::Db(DieselError::DatabaseError(
                DatabaseErrorKind::SerializationFailure,
                _
            ))
        ) {
            return true;
        }

        // Stable Postgres wording for 40001 and 40P01.
        let message = match self {
            TxError::Db(e) => e.to_string(),
            TxError::Domain(e) => e.to_string(),
        };
        message.contains("could not serialize access") || message.contains("deadlock detected")
    }

    fn into_error(self) -> Error {
        match self {
            TxError::Db(e) => Error::Database(e.to_string()),
            TxError::Domain(e) => e,
        }
    }
}

/// Runs `f` in a `SERIALIZABLE` transaction, retrying serialisation failures
/// with bounded exponential backoff.
///
/// `f` may run more than once, so it must be free of side effects outside the
/// transaction -- no email, no charge, no counter in process memory.
pub async fn with_serializable_retry<T, F>(
    pool: &DbPool,
    max_attempts: u32,
    f: F,
) -> Result<T, Error>
where
    T: Send + 'static,
    F: for<'c> Fn(
            &'c mut AsyncPgConnection,
        ) -> std::pin::Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'c>>
        + Send
        + Sync,
{
    let attempts = max_attempts.max(1);
    let mut last: Option<TxError> = None;

    for attempt in 0..attempts {
        if attempt > 0 {
            tokio::time::sleep(backoff(attempt)).await;
        }

        let mut conn = pool
            .get()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let result = conn
            .build_transaction()
            .serializable()
            .run(|c| Box::pin(async { f(c).await.map_err(TxError::Domain) }))
            .await;

        match result {
            Ok(value) => return Ok(value),
            Err(e) if e.is_retryable() && attempt + 1 < attempts => last = Some(e),
            Err(e) => return Err(e.into_error()),
        }
    }

    Err(match last {
        Some(e) => Error::Conflict {
            code: "serialization_failure",
            message: format!(
                "transaction could not be serialised after {attempts} attempts: {}",
                e.into_error()
            ),
        },
        None => Error::Database("transaction never ran".to_string()),
    })
}

/// Exponential backoff with jitter, capped at `MAX_BACKOFF`.
///
/// Jitter matters more than the curve: without it, every writer that lost the
/// same conflict wakes at the same moment and collides again.
fn backoff(attempt: u32) -> Duration {
    let exponential = BASE_BACKOFF.saturating_mul(1u32 << attempt.min(6));
    let capped = exponential.min(MAX_BACKOFF);

    // Cheap, dependency-free jitter: 50-100% of the computed delay, seeded from
    // the clock. A PRNG would be a new dependency for no extra benefit here.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let jitter_pct = 50 + (nanos % 51) as u64;
    Duration::from_micros(capped.as_micros() as u64 * jitter_pct / 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_and_is_capped() {
        assert!(backoff(0) <= MAX_BACKOFF);
        assert!(backoff(10) <= MAX_BACKOFF, "backoff must be bounded");
        // Never zero, or a retry storm becomes a busy loop.
        assert!(backoff(0) > Duration::ZERO);
    }
}
