//! `stheno-db::tenant`: the ambient tenant a database operation runs as.
//!
//! # Why a task-local and not a parameter
//!
//! Services are constructed once at boot with a cloned pool (`AppServer::new`), and
//! `PgCrudRepository<T>` holds only a pool and a `PhantomData`. There is no channel from
//! an HTTP handler down to a repository other than a parameter on every method — which
//! would mean touching `CrudRepository`, the `stheno_service_crud!` macro, and every
//! hand-written service method in every module. That is the cost ADR-009 priced at
//! "+12 weeks touching all 252 tasks".
//!
//! A task-local is inherited by every `.await` beneath the scope, so the middleware sets
//! it once per request and nothing else changes signature.
//!
//! # The honest cost
//!
//! Correctness now depends on invisible context. Two things contain that:
//!
//! 1. [`current`] returns `Err` when unset — a scoping bug is a 500, never another
//!    tenant's data.
//! 2. Postgres RLS is the silent second line: an unset `app.tenant_id` makes the policy
//!    predicate `NULL`, so a query returns zero rows rather than everything.
//!
//! A task-local does **not** cross `tokio::spawn`. Detached work must re-enter a scope
//! explicitly — see the subscriber registry.

use stheno_types::{Error, Result};

tokio::task_local! {
    static TENANT: String;
}

/// Process-wide fallback used only by tests.
///
/// 192 test files drive services directly with no HTTP layer and therefore no middleware
/// to establish a scope. Rather than rewrite all of them, the test harness sets this
/// once. It is `testing`-feature gated and `scripts/check-no-tenant-default.sh` asserts
/// the shipped server never enables that feature — a production binary must have no way
/// to reach a default tenant.
#[cfg(feature = "testing")]
static PROCESS_DEFAULT: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Sets the test-only process default. Ignored if already set.
#[cfg(feature = "testing")]
pub fn set_process_default(tenant: impl Into<String>) {
    let _ = PROCESS_DEFAULT.set(tenant.into());
}

#[cfg(feature = "testing")]
fn process_default() -> Option<String> {
    PROCESS_DEFAULT.get().cloned()
}

#[cfg(not(feature = "testing"))]
fn process_default() -> Option<String> {
    None
}

/// The tenant this task is running as.
///
/// Fails closed: an operation with no established tenant is a bug, and surfacing it as an
/// error is the only way to keep it from being silently served as "no rows".
pub fn current() -> Result<String> {
    TENANT
        .try_with(|t| t.clone())
        .ok()
        .or_else(process_default)
        .ok_or_else(|| Error::Invalid {
            code: "tenant_not_established",
            message: "no tenant is in scope for this operation".to_string(),
            field: None,
        })
}

/// Whether a tenant is in scope, without producing an error.
pub fn is_set() -> bool {
    TENANT.try_with(|_| ()).is_ok() || process_default().is_some()
}

/// Runs `f` with `tenant` established for the whole future.
pub async fn scope<F, R>(tenant: impl Into<String>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    TENANT.scope(tenant.into(), f).await
}

/// Rejects a tenant id that is not a plain identifier.
///
/// The id reaches Postgres through `set_config`, which is parameterised and so not
/// injectable — but it is also compared inside an RLS policy and appears in error
/// messages, and an id containing whitespace or quotes is a bug worth catching at the
/// boundary rather than debugging later.
pub fn validate(tenant: &str) -> Result<&str> {
    let ok = !tenant.is_empty()
        && tenant.len() <= 64
        && tenant
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

    if ok {
        Ok(tenant)
    } else {
        Err(Error::Invalid {
            code: "invalid_tenant_id",
            message: format!("{tenant:?} is not a valid tenant id"),
            field: Some("tenant_id".to_string()),
        })
    }
}

/// The scope used when reading the tenant registry itself.
///
/// Resolving "which tenant is this request for?" has to happen before a tenant is known,
/// so those reads cannot be scoped to one. The `platform` schema carries no RLS policy
/// for exactly that reason; this value exists only to satisfy [`apply_scope`], not to
/// filter anything.
///
/// It is deliberately not a real tenant id, so a query that accidentally runs under it
/// against a tenant-scoped table matches nothing rather than someone's data.
pub const PLATFORM_SCOPE: &str = "ten_platform_registry";

/// The role the application queries as (ADR-012).
///
/// Postgres exempts superusers from row-level security unconditionally, and the default
/// `DATABASE_URL` connects as one. Switching into a `NOSUPERUSER NOBYPASSRLS` role for
/// the duration of the transaction is what makes the policies apply at all.
const APP_ROLE: &str = "stheno_app";

/// Establishes the tenant scope on `conn` for the remainder of the current transaction.
///
/// Both settings are `LOCAL`, so they revert on COMMIT and a connection returned to the
/// pool carries neither. Callers must already be inside a transaction — `with_conn` is
/// what guarantees that.
pub async fn apply_scope(conn: &mut diesel_async::AsyncPgConnection) -> Result<()> {
    use crate::diesel_async::RunQueryDsl;
    use diesel::sql_types::Text;

    let tenant = current()?;
    validate(&tenant)?;

    // One statement, one round trip. `set_config(..., is_local => true)` is `SET LOCAL`,
    // and `role` is an ordinary GUC — `set_config('role', 'stheno_app', true)` is exactly
    // `SET LOCAL ROLE stheno_app`, verified against Postgres 16: `current_user` becomes
    // the app role, RLS applies, and both settings revert on COMMIT.
    //
    // Why one statement rather than two: `with_conn`'s non-transactional path already
    // costs BEGIN + scope + query + COMMIT, and under a transaction-mode pooler the
    // duration a transaction holds its server connection is what caps concurrency
    // (throughput = pool_size / transaction duration). Removing a round trip here removes
    // it from every read and write the engine performs.
    //
    // Evaluation order within the target list is not guaranteed by SQL, and does not need
    // to be: setting the role first still permits setting `app.tenant_id`, because custom
    // GUCs are not privileged. Both orders were tested.
    //
    // Both values are bind parameters now, so APP_ROLE is no longer formatted into SQL.
    diesel::sql_query("SELECT set_config('role', $1, true), set_config('app.tenant_id', $2, true)")
        .bind::<Text, _>(APP_ROLE)
        .bind::<Text, _>(tenant)
        .execute(conn)
        .await
        .map_err(|e| {
            Error::Database(format!("could not establish tenant scope as {APP_ROLE}: {e}"))
        })?;

    Ok(())
}

/// Checks out a pooled connection, opens a transaction, establishes the tenant scope, and
/// runs `f`.
///
/// This is the escape hatch for the handful of call sites that build raw SQL and so do
/// not go through `PgCrudRepository::with_conn`. Those sites must use this rather than
/// `pool.get()`, or they run with no tenant scope — which under RLS means they silently
/// see nothing, and on insert means a NOT NULL violation on `tenant_id`.
/// `scripts/check-pool-checkout.sh` enforces that.
pub async fn with_tenant_conn<T, F>(pool: &crate::DbPool, f: F) -> Result<T>
where
    T: Send + 'static,
    // `'static` is forced by the HRTB on `with_transaction`: the closure is moved into a
    // future that must be valid for any `'c`, so anything it captures must outlive them
    // all. Callers with `&str` parameters own them into a local first.
    F: for<'c> FnOnce(
            &'c mut diesel_async::AsyncPgConnection,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'c>,
        > + Send
        + 'static,
{
    let mut conn = pool
        .get()
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    crate::transaction::with_transaction(&mut conn, |c| {
        Box::pin(async move {
            apply_scope(c).await?;
            f(c).await
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_scope_is_visible_to_everything_beneath_it() {
        scope("ten_a", async {
            assert_eq!(current().unwrap(), "ten_a");
            // Nested awaits inherit it without being passed anything.
            async {
                assert_eq!(current().unwrap(), "ten_a");
            }
            .await;
        })
        .await;
    }

    #[tokio::test]
    async fn scopes_nest_and_restore() {
        scope("outer", async {
            scope("inner", async {
                assert_eq!(current().unwrap(), "inner");
            })
            .await;
            assert_eq!(current().unwrap(), "outer");
        })
        .await;
    }

    #[test]
    fn validate_rejects_anything_that_is_not_a_plain_identifier() {
        assert!(validate("ten_01ARZ3NDEKTSV4RRFFQ69G5FAV").is_ok());
        assert!(validate("").is_err());
        assert!(validate("ten 01").is_err());
        assert!(validate("ten';DROP").is_err());
        assert!(validate(&"x".repeat(65)).is_err());
    }
}
