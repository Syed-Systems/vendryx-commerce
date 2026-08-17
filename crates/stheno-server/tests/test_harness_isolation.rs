//! P11-16: `TestApp::spawn()` provisions a real, isolated, migrated backend.
//!
//! Phase 1's harness generated a schema name and then ignored it, so every test
//! shared one in-memory backend and "isolation" was a string comparison. These
//! tests assert against the database itself.

use stheno_db::testing::TestDb;
use stheno_server::TestApp;

#[tokio::test]
async fn spawn_provisions_a_dedicated_database() {
    let app = TestApp::spawn().await;

    let connected_to = app.db().query_text("SELECT current_database()").await;
    assert_eq!(
        connected_to, app.database_name,
        "the harness reported a database it is not actually connected to"
    );

    app.teardown().await;
}

#[tokio::test]
async fn spawn_migrates_the_database_it_provisions() {
    let app = TestApp::spawn().await;

    // A migrated database has the module schemas; an unmigrated one has none.
    let schemas = app
        .db()
        .count(
            "SELECT count(*) FROM information_schema.schemata \
             WHERE schema_name IN ('product', 'cart', 'order', 'payment')",
        )
        .await;
    assert_eq!(schemas, 4, "TestApp did not migrate its database");

    app.teardown().await;
}

#[tokio::test]
async fn two_apps_do_not_share_state() {
    let a = TestApp::spawn().await;
    let b = TestApp::spawn().await;

    assert_ne!(
        a.database_name, b.database_name,
        "two harnesses landed on the same database"
    );

    // Writing through one must be invisible to the other. This is the assertion
    // the previous harness could not make.
    a.db()
        .execute(
            "INSERT INTO product.product (id, title, handle) \
             VALUES ('prod_isolation_probe', 'probe', 'probe-handle')",
        )
        .await;

    let in_a = a
        .db()
        .count("SELECT count(*) FROM product.product WHERE id = 'prod_isolation_probe'")
        .await;
    let in_b = b
        .db()
        .count("SELECT count(*) FROM product.product WHERE id = 'prod_isolation_probe'")
        .await;

    assert_eq!(in_a, 1, "app A lost its own write");
    assert_eq!(
        in_b, 0,
        "app B saw app A's write; the harness is not isolated"
    );

    a.teardown().await;
    b.teardown().await;
}

#[tokio::test]
async fn teardown_drops_the_database() {
    let app = TestApp::spawn().await;
    let name = app.database_name.clone();
    let admin_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    app.teardown().await;

    let remaining = TestDb::count_on_admin(
        &admin_url,
        &format!("SELECT count(*) FROM pg_database WHERE datname = '{name}'"),
    )
    .await;
    assert_eq!(remaining, 0, "teardown left database '{name}' behind");
}
