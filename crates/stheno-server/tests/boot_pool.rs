//! P11-03: the boot path builds a real connection pool from `DATABASE_URL`.
//!
//! Configuration parsing is tested through `DbConfig::from_values` rather than
//! by setting environment variables: `set_var` is process-global, and a test
//! that writes `DATABASE_URL` races every other test that reads it.

use std::time::Duration;
use stheno_server::db::{check_connectivity, DbConfig, PoolHealth, DEFAULT_POOL_SIZE};

const URL: &str = "postgres://u:p@localhost:5432/db";

#[test]
fn database_url_is_read_and_pool_size_defaults() {
    let cfg = DbConfig::from_values(Some(URL), None).expect("config must resolve");
    assert_eq!(cfg.database_url, URL);
    assert_eq!(cfg.pool_size, DEFAULT_POOL_SIZE);
}

#[test]
fn a_missing_database_url_fails_boot_rather_than_falling_back() {
    let err =
        DbConfig::from_values(None, None).expect_err("boot must not proceed without DATABASE_URL");
    assert!(
        err.to_string().contains("DATABASE_URL"),
        "the error must name the missing variable, got: {err}"
    );
}

#[test]
fn an_empty_database_url_is_rejected() {
    let err = DbConfig::from_values(Some("   "), None)
        .expect_err("a blank DATABASE_URL is not a configuration");
    assert!(err.to_string().contains("empty"), "got: {err}");
}

#[test]
fn pool_size_is_configurable() {
    let cfg = DbConfig::from_values(Some(URL), Some("42")).expect("config must resolve");
    assert_eq!(cfg.pool_size, 42);
}

#[test]
fn a_nonsense_pool_size_is_rejected_not_silently_defaulted() {
    for bad in ["0", "-1", "many", "", "3.5"] {
        let err = DbConfig::from_values(Some(URL), Some(bad))
            .unwrap_err_or_else_msg(&format!("pool size '{bad}' must be rejected"));
        assert!(
            err.contains("STHENO_DB_POOL_SIZE"),
            "pool size '{bad}' should be rejected by name, got: {err}"
        );
    }
}

/// Small helper so the loop above reads as one assertion per case.
trait UnwrapErrMsg {
    fn unwrap_err_or_else_msg(self, msg: &str) -> String;
}

impl UnwrapErrMsg for Result<DbConfig, stheno_types::Error> {
    fn unwrap_err_or_else_msg(self, msg: &str) -> String {
        match self {
            Ok(cfg) => panic!("{msg}, but it parsed as pool_size {}", cfg.pool_size),
            Err(e) => e.to_string(),
        }
    }
}

#[test]
fn the_pool_is_built_with_the_configured_size() {
    let cfg = DbConfig::from_values(Some(URL), Some("7")).expect("config");
    let pool = cfg.build_pool().expect("pool must build");
    let health = PoolHealth::of(&pool);

    assert_eq!(
        health.max_size, 7,
        "pool did not honour the configured size"
    );
    // A freshly built pool has created no connections, so none are in use.
    assert_eq!(
        health.saturation(),
        0.0,
        "a pool that has never been used must not report as saturated"
    );
}

#[test]
fn saturation_reports_checked_out_connections() {
    let full = PoolHealth {
        size: 4,
        available: 0,
        max_size: 4,
    };
    assert_eq!(
        full.saturation(),
        1.0,
        "a fully checked-out pool is saturated"
    );

    let half = PoolHealth {
        size: 4,
        available: 2,
        max_size: 4,
    };
    assert_eq!(half.saturation(), 0.5);

    let idle = PoolHealth {
        size: 4,
        available: 4,
        max_size: 4,
    };
    assert_eq!(idle.saturation(), 0.0);
}

/// The pool is lazy, so only a real round trip proves the boot path works.
#[tokio::test]
async fn the_pool_reaches_a_real_database() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        panic!("DATABASE_URL must be set to run the suite; see README.md");
    };

    let cfg = DbConfig::from_values(Some(&url), Some("2")).expect("config");
    let pool = cfg.build_pool().expect("pool must build");

    check_connectivity(&pool, Duration::from_secs(10))
        .await
        .expect("boot path must reach the configured database");

    // A round trip has happened, so the pool has a live connection to report.
    let health = PoolHealth::of(&pool);
    assert!(health.size >= 1, "pool created no connections");
    assert_eq!(health.max_size, 2);
}

#[tokio::test]
async fn connectivity_failure_is_reported_not_swallowed() {
    // Port 1 is reserved and never listening.
    let cfg =
        DbConfig::from_values(Some("postgres://u:p@127.0.0.1:1/db"), Some("1")).expect("config");
    let pool = cfg
        .build_pool()
        .expect("pool builds lazily even for a bad host");

    let err = check_connectivity(&pool, Duration::from_secs(5))
        .await
        .expect_err("an unreachable database must surface as an error");
    assert!(!err.to_string().is_empty());
}
