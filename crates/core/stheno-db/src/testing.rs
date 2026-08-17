//! Isolated Postgres databases for tests.
//!
//! Each test gets its own database, created from `DATABASE_URL` and dropped on
//! teardown. Migrations create schemas and tables, so tests sharing one database
//! would collide the moment two of them ran in parallel -- which cargo does by
//! default.
//!
//! Behind the `testing` feature so it is never compiled into a release binary.
//! Lives in the library rather than a `tests/` directory because
//! `stheno-server` needs the same harness (P11-06, P11-16) and integration test
//! helpers cannot be shared across crates.

use diesel::sql_types::{BigInt, Bool, Text};
use diesel::QueryableByName;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, SimpleAsyncConnection};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(QueryableByName)]
struct BoolRow {
    #[diesel(sql_type = Bool)]
    value: bool,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    value: i64,
}

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = Text)]
    value: String,
}

pub struct TestDb {
    pub url: String,
    pub name: String,
    admin_url: String,
}

impl TestDb {
    /// Creates a fresh, empty database and returns a handle to it.
    ///
    /// Panics rather than skipping when `DATABASE_URL` is absent: a persistence
    /// test that silently passes without a database is precisely the failure
    /// Phase 2 exists to eliminate (PHASE2_PLAN.md 1.3).
    pub async fn create() -> Self {
        let admin_url = std::env::var("DATABASE_URL").expect(
            "DATABASE_URL must be set to run persistence tests; see README.md 'Running the tests'",
        );

        // Unique per process and per call, so parallel tests and concurrent
        // cargo invocations cannot land on the same name.
        let name = format!(
            "stheno_test_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );

        let mut admin = AsyncPgConnection::establish(&admin_url)
            .await
            .expect("connect to the administrative database");
        // One statement per call: CREATE/DROP DATABASE cannot run inside a
        // transaction block, and `batch_execute` opens one.
        // The drop comes first in case a previous run died before teardown.
        diesel::sql_query(format!(r#"DROP DATABASE IF EXISTS "{name}" WITH (FORCE)"#))
            .execute(&mut admin)
            .await
            .expect("drop any leftover test database");
        diesel::sql_query(format!(r#"CREATE DATABASE "{name}""#))
            .execute(&mut admin)
            .await
            .expect("create the test database");

        Self {
            url: replace_database(&admin_url, &name),
            name,
            admin_url,
        }
    }

    /// Creates a fresh database with every migration already applied.
    pub async fn migrated() -> Self {
        let db = Self::create().await;
        crate::migrations::run(&db.url)
            .await
            .expect("migrations must apply to a fresh database");
        db
    }

    pub async fn conn(&self) -> AsyncPgConnection {
        AsyncPgConnection::establish(&self.url)
            .await
            .expect("connect to the test database")
    }

    pub async fn execute(&self, sql: &str) {
        let mut conn = self.conn().await;
        conn.batch_execute(sql)
            .await
            .unwrap_or_else(|e| panic!("failed to execute {sql:?}: {e}"));
    }

    /// Runs a query returning a single boolean.
    pub async fn query_bool(&self, sql: &str) -> bool {
        let mut conn = self.conn().await;
        let row: BoolRow = diesel::sql_query(format!("SELECT ({sql}) AS value"))
            .get_result(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("failed to query {sql:?}: {e}"));
        row.value
    }

    /// Runs a query returning a single count.
    pub async fn count(&self, sql: &str) -> i64 {
        let mut conn = self.conn().await;
        let row: CountRow = diesel::sql_query(format!("SELECT ({sql}) AS value"))
            .get_result(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("failed to query {sql:?}: {e}"));
        row.value
    }

    /// Runs a query returning a single text value.
    pub async fn query_text(&self, sql: &str) -> String {
        let mut conn = self.conn().await;
        let row: TextRow = diesel::sql_query(format!("SELECT ({sql})::text AS value"))
            .get_result(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("failed to query {sql:?}: {e}"));
        row.value
    }

    /// Counts rows in the administrative database rather than this one, so a
    /// test can assert about its own database after dropping it.
    pub async fn count_on_admin(admin_url: &str, sql: &str) -> i64 {
        let mut conn = AsyncPgConnection::establish(admin_url)
            .await
            .expect("connect to the administrative database");
        let row: CountRow = diesel::sql_query(format!("SELECT ({sql}) AS value"))
            .get_result(&mut conn)
            .await
            .unwrap_or_else(|e| panic!("failed to query {sql:?}: {e}"));
        row.value
    }

    /// Drops the database. Call at the end of a test.
    ///
    /// Not a `Drop` impl: dropping a database is async, and a blocking teardown
    /// inside `Drop` would deadlock the runtime that is executing the test.
    pub async fn drop(self) {
        let mut admin = match AsyncPgConnection::establish(&self.admin_url).await {
            Ok(c) => c,
            // Teardown failures must not mask the assertion that actually failed.
            Err(e) => {
                eprintln!("warning: could not connect to drop {}: {e}", self.name);
                return;
            }
        };
        if let Err(e) = diesel::sql_query(format!(
            r#"DROP DATABASE IF EXISTS "{}" WITH (FORCE)"#,
            self.name
        ))
        .execute(&mut admin)
        .await
        {
            eprintln!("warning: could not drop {}: {e}", self.name);
        }
    }
}

/// Swaps the database name in a `postgres://host/dbname` URL.
fn replace_database(url: &str, database: &str) -> String {
    // Split on the last '/' that precedes any query string, so that passwords
    // or hosts containing '/' cannot be mistaken for the database segment.
    let (base, query) = match url.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (url, None),
    };
    let trimmed = base.trim_end_matches('/');
    let cut = trimmed
        .rfind('/')
        .expect("DATABASE_URL must name a database");
    let mut out = format!("{}/{}", &trimmed[..cut], database);
    if let Some(q) = query {
        out.push('?');
        out.push_str(q);
    }
    out
}

#[cfg(test)]
mod url_tests {
    use super::replace_database;

    #[test]
    fn swaps_the_database_segment() {
        assert_eq!(
            replace_database("postgres://u:p@localhost:5432/stheno", "t1"),
            "postgres://u:p@localhost:5432/t1"
        );
    }

    #[test]
    fn preserves_query_parameters() {
        assert_eq!(
            replace_database("postgres://u:p@h:5432/stheno?sslmode=require", "t1"),
            "postgres://u:p@h:5432/t1?sslmode=require"
        );
    }
}
