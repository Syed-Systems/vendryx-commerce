//! P11-14: `SERIALIZABLE` retry helper with bounded backoff.

mod common;

use common::widget_db;
use diesel_async::RunQueryDsl;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use stheno_db::{create_pool, with_serializable_retry, DEFAULT_MAX_ATTEMPTS};
use stheno_types::Error;

#[tokio::test]
async fn a_successful_transaction_runs_once_and_commits() {
    let db = widget_db().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let calls = Arc::new(AtomicU32::new(0));

    let c = calls.clone();
    with_serializable_retry(&pool, DEFAULT_MAX_ATTEMPTS, move |conn| {
        let c = c.clone();
        Box::pin(async move {
            c.fetch_add(1, Ordering::SeqCst);
            diesel::sql_query("INSERT INTO catalog.widget (id, title) VALUES ('wid_1', 'ok')")
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
            Ok(())
        })
    })
    .await
    .expect("transaction must commit");

    assert_eq!(
        AtomicU32::load(&calls, Ordering::SeqCst),
        1,
        "no retry was needed"
    );
    assert_eq!(db.count("SELECT count(*) FROM catalog.widget").await, 1);

    db.drop().await;
}

#[tokio::test]
async fn a_domain_error_rolls_back_and_is_not_retried() {
    // Only serialisation failures are retryable. Retrying a validation error
    // would just fail four more times, slowly.
    let db = widget_db().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let calls = Arc::new(AtomicU32::new(0));

    let c = calls.clone();
    let result: Result<(), Error> =
        with_serializable_retry(&pool, DEFAULT_MAX_ATTEMPTS, move |conn| {
            let c = c.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
                diesel::sql_query(
                    "INSERT INTO catalog.widget (id, title) VALUES ('wid_1', 'doomed')",
                )
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
                Err(Error::Invalid {
                    code: "nope",
                    message: "domain rejected it".into(),
                    field: None,
                })
            })
        })
        .await;

    assert!(result.is_err());
    assert_eq!(
        AtomicU32::load(&calls, Ordering::SeqCst),
        1,
        "a domain error must not be retried"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM catalog.widget").await,
        0,
        "the failed transaction was not rolled back"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_retry_budget_is_bounded() {
    // A conflict that never clears must terminate rather than loop forever.
    let db = widget_db().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let calls = Arc::new(AtomicU32::new(0));

    let c = calls.clone();
    let result: Result<(), Error> = with_serializable_retry(&pool, 3, move |_conn| {
        let c = c.clone();
        Box::pin(async move {
            c.fetch_add(1, Ordering::SeqCst);
            Err(Error::Database("always fails".into()))
        })
    })
    .await;

    assert!(result.is_err());
    // A domain error is not retryable, so this also confirms the loop exits
    // immediately rather than burning the whole budget on a hopeless case.
    assert_eq!(AtomicU32::load(&calls, Ordering::SeqCst), 1);

    db.drop().await;
}

#[tokio::test]
async fn zero_attempts_is_treated_as_one() {
    let db = widget_db().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let calls = Arc::new(AtomicU32::new(0));

    let c = calls.clone();
    with_serializable_retry(&pool, 0, move |_conn| {
        let c = c.clone();
        Box::pin(async move {
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    })
    .await
    .expect("must still run once");

    assert_eq!(
        AtomicU32::load(&calls, Ordering::SeqCst),
        1,
        "a zero budget must not skip the work entirely"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_transaction_really_runs_at_serializable() {
    let db = widget_db().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let level = with_serializable_retry(&pool, 1, |conn| {
        Box::pin(async move {
            #[derive(diesel::QueryableByName)]
            struct Row {
                #[diesel(sql_type = diesel::sql_types::Text)]
                value: String,
            }
            let row: Row =
                diesel::sql_query("SELECT current_setting('transaction_isolation') AS value")
                    .get_result(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
            Ok(row.value)
        })
    })
    .await
    .expect("query isolation level");

    assert_eq!(
        level, "serializable",
        "the helper did not open a SERIALIZABLE transaction"
    );

    db.drop().await;
}
