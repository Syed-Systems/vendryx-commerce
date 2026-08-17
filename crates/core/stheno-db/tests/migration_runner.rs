//! P11-05: migrations are embedded in the binary and run against a real database.
//!
//! Every test here provisions its own Postgres database so that applying and
//! reverting migrations cannot disturb a concurrently running test.

use stheno_db::migrations;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn migrations_are_embedded_in_the_binary() {
    // No files on disk are consulted; a deployed image carries its own schema.
    let db = TestDb::create().await;
    let pending = migrations::pending(&db.url).await.expect("list pending");
    assert!(
        !pending.is_empty(),
        "no migrations were embedded; check the embed_migrations! path"
    );
    db.drop().await;
}

#[tokio::test]
async fn running_migrations_applies_every_pending_one() {
    let db = TestDb::create().await;

    let before = migrations::pending(&db.url).await.expect("list pending");
    let applied = migrations::run(&db.url).await.expect("run migrations");
    assert_eq!(
        applied.len(),
        before.len(),
        "every pending migration should have been applied"
    );

    let after = migrations::pending(&db.url).await.expect("list pending");
    assert!(
        after.is_empty(),
        "migrations still pending after a successful run: {after:?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn running_migrations_twice_is_a_no_op() {
    let db = TestDb::create().await;

    migrations::run(&db.url).await.expect("first run");
    let second = migrations::run(&db.url).await.expect("second run");
    assert!(
        second.is_empty(),
        "a second run should apply nothing, applied: {second:?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn migrations_actually_create_the_module_schemas() {
    // Proves the runner executed the SQL rather than only recording versions.
    let db = TestDb::create().await;
    migrations::run(&db.url).await.expect("run migrations");

    for schema in [
        "product",
        "cart",
        "payment",
        "promotion",
        "fulfillment",
        "order",
    ] {
        let exists = db
            .query_bool(&format!(
                "SELECT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = '{schema}')"
            ))
            .await;
        assert!(exists, "migration did not create schema '{schema}'");
    }

    let has_product_table = db
        .query_bool(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
             WHERE table_schema = 'product' AND table_name = 'product')",
        )
        .await;
    assert!(has_product_table, "product.product table was not created");

    db.drop().await;
}

#[tokio::test]
async fn a_reverted_migration_shows_up_as_pending_again() {
    // This is the mechanism `--check-migrations` relies on (P11-06).
    let db = TestDb::create().await;
    migrations::run(&db.url).await.expect("run migrations");
    assert!(migrations::pending(&db.url).await.unwrap().is_empty());

    migrations::revert_last(&db.url).await.expect("revert");

    let pending = migrations::pending(&db.url).await.expect("list pending");
    assert_eq!(
        pending.len(),
        1,
        "reverting one migration should leave exactly one pending, got {pending:?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn migration_versions_are_unique() {
    // Diesel keys applied migrations by the version prefix. Two directories
    // sharing one version means the second is silently never applied.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations");
    let mut versions: Vec<String> = std::fs::read_dir(&dir)
        .expect("migrations directory must exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| {
            e.file_name()
                .to_string_lossy()
                .split('_')
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .collect();

    let total = versions.len();
    assert!(total > 0, "no migration directories found in {dir:?}");
    versions.sort();
    versions.dedup();
    assert_eq!(
        versions.len(),
        total,
        "duplicate migration version numbers found; each must be unique"
    );
}

#[tokio::test]
async fn every_migration_has_an_up_and_a_down() {
    // A migration without a down.sql cannot be reverted, which breaks both the
    // drift check and any rollback.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../migrations");
    for entry in std::fs::read_dir(&dir).expect("migrations directory") {
        let path = entry.expect("dir entry").path();
        if !path.is_dir() {
            continue;
        }
        assert!(path.join("up.sql").is_file(), "{path:?} has no up.sql");
        assert!(path.join("down.sql").is_file(), "{path:?} has no down.sql");
    }
}
