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
static TEST_DB_CREATE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
        let _create_guard = TEST_DB_CREATE_LOCK.lock().await;
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

        // `drop()` is skipped whenever a test panics, so failed runs leak a database
        // each. Left alone they accumulate across runs — 500 of them exhausted the
        // server's resources and put it into recovery mid-suite, which then looked like
        // dozens of unrelated test failures. Sweeping other processes' leftovers on
        // startup keeps a failing suite from poisoning the next one.
        //
        // Only databases belonging to processes that no longer exist are removed, so
        // concurrent `cargo test` invocations cannot delete each other's.
        Self::sweep_orphaned(&mut admin).await;
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

        let db = Self {
            url: replace_database(&admin_url, &name),
            name,
            admin_url,
        };

        // Every test database needs the tenancy preconditions (ADR-012), not just the
        // migrated ones: fixtures like `widget_db` create ad-hoc tables against a bare
        // database and still go through `PgCrudRepository`, which now assumes a tenant
        // scope and the `stheno_app` role.
        //
        // `ALTER DEFAULT PRIVILEGES` is what makes this work without touching each
        // fixture — tables those fixtures create afterwards are granted automatically.
        Self::execute(
            &db,
            "DO $$ BEGIN
                 IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'stheno_app') THEN
                     CREATE ROLE stheno_app NOLOGIN NOSUPERUSER NOBYPASSRLS;
                 END IF;
             END $$;
             DO $$ BEGIN
                 EXECUTE format('GRANT stheno_app TO %I', current_user);
             EXCEPTION WHEN duplicate_object THEN NULL;
             END $$;
             GRANT USAGE, CREATE ON SCHEMA public TO stheno_app;
             GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO stheno_app;
             ALTER DEFAULT PRIVILEGES IN SCHEMA public
                 GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO stheno_app;",
        )
        .await;

        crate::tenant::set_process_default(Self::TEST_TENANT);
        db
    }

    /// The tenant every test runs as unless it establishes its own scope.
    pub const TEST_TENANT: &'static str = "ten_test";

    /// Creates a fresh database with every migration already applied.
    ///
    /// Also registers [`Self::TEST_TENANT`] and makes it the process default, because the
    /// 190-odd test files drive services directly with no HTTP layer and therefore no
    /// middleware to establish a tenant scope (ADR-012). Without this every one of them
    /// would fail closed.
    ///
    /// The process default is `testing`-feature gated and
    /// `scripts/check-no-tenant-default.sh` asserts the shipped server never enables that
    /// feature — a production binary must have no way to reach a default tenant.
    pub async fn migrated() -> Self {
        let db = Self::create().await;
        crate::migrations::run(&db.url)
            .await
            .expect("migrations must apply to a fresh database");

        // UFCS: `RunQueryDsl` is in scope in this module and its `execute` would
        // otherwise win method resolution.
        Self::execute(
            &db,
            &format!(
                "INSERT INTO platform.tenant (id, name) VALUES ('{t}', 'Test')
                 ON CONFLICT (id) DO NOTHING;
                 INSERT INTO platform.subscription (tenant_id, tier) VALUES ('{t}', 'pro')
                 ON CONFLICT (tenant_id) DO NOTHING;",
                t = Self::TEST_TENANT
            ),
        )
        .await;

        crate::tenant::set_process_default(Self::TEST_TENANT);
        db
    }

    /// A connection with the test tenant already established.
    ///
    /// Fixtures insert rows with raw SQL through the helpers below, which do not go
    /// through `PgCrudRepository::with_conn` and so would leave `app.tenant_id` unset —
    /// making every `tenant_id` DEFAULT resolve to NULL and violate NOT NULL (ADR-012).
    ///
    /// Session-level rather than `SET LOCAL`, because these helpers run single statements
    /// outside a transaction. That is safe here and only here: the connection is not
    /// pooled, it belongs to one test, and it is dropped with it.
    pub async fn conn(&self) -> AsyncPgConnection {
        let mut conn = AsyncPgConnection::establish(&self.url)
            .await
            .expect("connect to the test database");
        conn.batch_execute(&format!("SET app.tenant_id = '{}'", Self::TEST_TENANT))
            .await
            .expect("establish the test tenant on the connection");
        conn
    }

    pub async fn execute(&self, sql: &str) {
        let mut conn = self.conn().await;
        conn.batch_execute(sql)
            .await
            .unwrap_or_else(|e| panic!("failed to execute {sql:?}: {e}"));

        // Fixtures create their ad-hoc schemas and tables through this method, and
        // `PgCrudRepository` now queries as `stheno_app` (ADR-012). Re-granting here is
        // what keeps every existing fixture working without edits — a fixture that
        // creates a `catalog` schema gets access to it without knowing the role exists.
        Self::grant_app_access(&mut conn).await;
    }

    /// Grants the application role access to everything currently in the database.
    ///
    /// Idempotent and test-only. Production grants are made once by the `app_role`
    /// migration, which also sets default privileges for tables added later.
    async fn grant_app_access(conn: &mut AsyncPgConnection) {
        let sql = "DO $$
             DECLARE s text;
             BEGIN
                 IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'stheno_app') THEN
                     RETURN;
                 END IF;
                 FOR s IN SELECT nspname FROM pg_namespace
                           WHERE nspname NOT LIKE 'pg\\_%'
                             AND nspname <> 'information_schema'
                 LOOP
                     EXECUTE format('GRANT USAGE ON SCHEMA %I TO stheno_app', s);
                     EXECUTE format(
                         'GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA %I TO stheno_app', s);
                     EXECUTE format(
                         'GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA %I TO stheno_app', s);
                 END LOOP;
             END $$;";
        conn.batch_execute(sql)
            .await
            .unwrap_or_else(|e| panic!("failed to grant app role access: {e}"));
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

    /// Drops test databases left behind by processes that have exited.
    ///
    /// Runs at most once per process. Failures are ignored: this is hygiene, and a
    /// sweep that cannot run must never fail the test that triggered it.
    async fn sweep_orphaned(admin: &mut AsyncPgConnection) {
        use std::sync::Once;
        static SWEPT: Once = Once::new();
        let mut should_sweep = false;
        SWEPT.call_once(|| should_sweep = true);
        if !should_sweep {
            return;
        }

        let me = std::process::id();
        // `datname LIKE 'stheno_test_<pid>_%'` identifies the owning process. Anything
        // whose pid is not ours is from a previous run: this process is the only one
        // that could still be using its own.
        let sql = format!(
            "DO $$
             DECLARE d text;
             BEGIN
                 FOR d IN SELECT datname FROM pg_database
                           WHERE datname LIKE 'stheno\\_test\\_%'
                             AND datname NOT LIKE 'stheno\\_test\\_{me}\\_%'
                 LOOP
                     BEGIN
                         EXECUTE format('DROP DATABASE IF EXISTS %I WITH (FORCE)', d);
                     EXCEPTION WHEN OTHERS THEN NULL;
                     END;
                 END LOOP;
             END $$;"
        );
        let _ = admin.batch_execute(&sql).await;
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
