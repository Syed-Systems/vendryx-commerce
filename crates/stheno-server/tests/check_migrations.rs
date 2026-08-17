//! P11-06: `--check-migrations` exits non-zero on schema drift.
//!
//! Drives the real binary rather than the library function, because the exit
//! code is the contract: CI and the deploy pipeline read it, not a Rust value.

use std::process::Command;
use stheno_db::testing::TestDb;

/// Runs the server binary with the given args against `url`.
fn run(url: &str, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_stheno-server"))
        .args(args)
        .env("DATABASE_URL", url)
        // The binary loads tier config relative to the repo root.
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .output()
        .expect("failed to run stheno-server")
}

#[tokio::test]
async fn check_migrations_fails_on_an_unmigrated_database() {
    let db = TestDb::create().await;

    let out = run(&db.url, &["--check-migrations"]);
    assert!(
        !out.status.success(),
        "an unmigrated database must be reported as drifted, got exit {:?}\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("drift"),
        "the failure should say what is wrong"
    );

    db.drop().await;
}

#[tokio::test]
async fn check_migrations_passes_on_a_migrated_database() {
    let db = TestDb::create().await;

    let migrate = run(&db.url, &["--migrate", "--check-migrations"]);
    assert!(
        migrate.status.success(),
        "migrating then checking must succeed\nstderr: {}",
        String::from_utf8_lossy(&migrate.stderr),
    );

    db.drop().await;
}

#[tokio::test]
async fn check_migrations_detects_a_deliberately_drifted_schema() {
    // The P11 exit-gate assertion: drift is detected, not assumed absent.
    let db = TestDb::migrated().await;
    assert!(
        run(&db.url, &["--check-migrations"]).status.success(),
        "a freshly migrated database must be clean"
    );

    stheno_db::migrations::revert_last(&db.url)
        .await
        .expect("revert one migration to create drift");

    let out = run(&db.url, &["--check-migrations"]);
    assert!(
        !out.status.success(),
        "a reverted migration must be reported as drift, got exit {:?}",
        out.status.code(),
    );

    db.drop().await;
}

#[tokio::test]
async fn boot_refuses_to_start_without_a_database_url() {
    let out = Command::new(env!("CARGO_BIN_EXE_stheno-server"))
        .arg("--check-migrations")
        .env_remove("DATABASE_URL")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .output()
        .expect("failed to run stheno-server");

    assert!(
        !out.status.success(),
        "the server must not start without DATABASE_URL"
    );
}
