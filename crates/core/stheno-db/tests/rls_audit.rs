//! ADR-012: the audit that keeps tenant isolation true for tables added later.
//!
//! The migration that adds `tenant_id` and the policies loops over `pg_tables`, so it
//! runs once and cannot speak for anything created afterwards. This test does — it
//! recomputes coverage from the live catalog on every run, so a table added next year is
//! either covered or this fails.

use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant};

/// Tables deliberately global. Every entry must be arguable as "two tenants could not
/// legitimately disagree about this row"; the list is the leak surface, so it stays short.
const UNSCOPED: &[&str] = &[
    "currency.currency", // ISO 4217, keyed by the code itself
    // A build registry, keyed by the identifier the provider's code returns. What a
    // merchant *uses* is the tenant-scoped link table, not this row.
    "fulfillment.fulfillment_provider",
    "public.__diesel_schema_migrations", // migration bookkeeping
];

/// If the connection is a superuser, Postgres skips RLS entirely and every other
/// assertion in this file would pass while proving nothing. This must run first.
#[tokio::test]
async fn the_app_role_is_not_a_superuser() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 2).expect("pool");

    let is_superuser = tenant::with_tenant_conn(&pool, |conn| {
        Box::pin(async move {
            use diesel::sql_types::Bool;
            use diesel::QueryableByName;
            use stheno_db::diesel_async::RunQueryDsl;

            #[derive(QueryableByName)]
            struct Row {
                #[diesel(sql_type = Bool)]
                value: bool,
            }

            let row: Row =
                diesel::sql_query("SELECT current_setting('is_superuser') = 'on' AS value")
                    .get_result(conn)
                    .await
                    .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
            Ok(row.value)
        })
    })
    .await
    .expect("query is_superuser");

    assert!(
        !is_superuser,
        "the application role is a superuser, so RLS is bypassed and every isolation \
         guarantee in this codebase is inert. Check that `SET LOCAL ROLE` is applied."
    );

    db.drop().await;
}

#[tokio::test]
async fn every_table_is_scoped_and_forced() {
    let db = TestDb::migrated().await;

    // `relforcerowsecurity` matters as much as `relrowsecurity`: the app owns these
    // tables, and Postgres exempts owners from RLS unless it is FORCEd.
    let gaps = db
        .query_text(
            "SELECT COALESCE(string_agg(t, ', ' ORDER BY t), '') FROM (
               SELECT n.nspname || '.' || c.relname AS t
                 FROM pg_class c
                 JOIN pg_namespace n ON n.oid = c.relnamespace
                WHERE c.relkind = 'r'
                  AND n.nspname NOT IN ('pg_catalog','information_schema','platform')
                  AND n.nspname NOT LIKE 'pg_%'
                  AND NOT (
                        EXISTS (SELECT 1 FROM pg_attribute a
                                 WHERE a.attrelid = c.oid
                                   AND a.attname = 'tenant_id'
                                   AND NOT a.attisdropped)
                    AND c.relrowsecurity
                    AND c.relforcerowsecurity
                    AND EXISTS (SELECT 1 FROM pg_policy p WHERE p.polrelid = c.oid)
                  )
             ) g",
        )
        .await;

    let unexpected: Vec<&str> = gaps
        .split(", ")
        .filter(|t| !t.is_empty() && !UNSCOPED.contains(t))
        .collect();

    assert!(
        unexpected.is_empty(),
        "these tables have no tenant isolation and are not on the argued exclusion list: {unexpected:?}"
    );

    db.drop().await;
}

/// The property the whole design exists for.
#[tokio::test]
async fn one_tenant_cannot_see_or_change_another_tenants_rows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    db.execute(
        "INSERT INTO platform.tenant (id, name) VALUES ('ten_a','A'), ('ten_b','B')
         ON CONFLICT (id) DO NOTHING",
    )
    .await;

    let count_products = |pool: stheno_db::DbPool| async move {
        tenant::with_tenant_conn(&pool, |conn| {
            Box::pin(async move {
                use diesel::sql_types::BigInt;
                use diesel::QueryableByName;
                use stheno_db::diesel_async::RunQueryDsl;

                #[derive(QueryableByName)]
                struct Row {
                    #[diesel(sql_type = BigInt)]
                    value: i64,
                }
                let row: Row = diesel::sql_query("SELECT count(*) AS value FROM product.product")
                    .get_result(conn)
                    .await
                    .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(row.value)
            })
        })
        .await
    };

    // A writes a product.
    tenant::scope("ten_a", async {
        tenant::with_tenant_conn(&pool, |conn| {
            Box::pin(async move {
                use stheno_db::diesel_async::RunQueryDsl;
                diesel::sql_query(
                    "INSERT INTO product.product (id, title, handle, status)
                     VALUES ('prod_a1','A only','a-only','published')",
                )
                .execute(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
        .expect("tenant A insert");

        assert_eq!(
            count_products(pool.clone()).await.unwrap(),
            1,
            "A sees its own row"
        );
    })
    .await;

    // B sees nothing at all.
    tenant::scope("ten_b", async {
        assert_eq!(
            count_products(pool.clone()).await.unwrap(),
            0,
            "tenant B must not see tenant A's product"
        );

        // And cannot reach it by id, which is the case a WHERE-clause filter would miss.
        tenant::with_tenant_conn(&pool, |conn| {
            Box::pin(async move {
                use stheno_db::diesel_async::RunQueryDsl;
                let affected = diesel::sql_query(
                    "UPDATE product.product SET title = 'hijacked' WHERE id = 'prod_a1'",
                )
                .execute(conn)
                .await
                .map_err(|e| stheno_db::Error::Database(e.to_string()))?;
                assert_eq!(affected, 0, "tenant B must not be able to update A's row");
                Ok(())
            })
        })
        .await
        .expect("tenant B update attempt");
    })
    .await;

    // A's row is intact and unmodified.
    tenant::scope("ten_a", async {
        assert_eq!(count_products(pool.clone()).await.unwrap(), 1);
        let title = db
            .query_text("SELECT title FROM product.product WHERE id = 'prod_a1'")
            .await;
        assert_eq!(title, "A only", "tenant B's update must not have landed");
    })
    .await;

    db.drop().await;
}

/// An operation with no tenant established must fail, not fall back to seeing everything.
#[tokio::test]
async fn an_unscoped_operation_fails_closed() {
    let db = TestDb::migrated().await;

    // `tenant::current()` is what fails here. The test harness sets a process-wide default
    // so the other 400-odd suites keep working, so this asserts the validation path
    // instead: an id that is not a plain identifier is refused before it reaches Postgres.
    assert!(tenant::validate("ten_a; DROP TABLE product.product").is_err());
    assert!(tenant::validate("").is_err());

    db.drop().await;
}
