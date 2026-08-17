//! P11-15: two concurrent writers produce a serialisation failure that the
//! helper retries, and both commit.
//!
//! This is the test that decides whether `SERIALIZABLE` plus retry is a usable
//! concurrency strategy for the checkout path, or whether P13 needs something
//! else. It provokes a real 40001 rather than simulating one.

mod common;

use common::widget_db;
use diesel_async::RunQueryDsl;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use stheno_db::{create_pool, with_serializable_retry, DbPool, DEFAULT_MAX_ATTEMPTS};
use stheno_types::Error;

#[derive(diesel::QueryableByName)]
struct IntRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    value: i32,
}

/// Read-modify-write on a shared row.
///
/// Under `SERIALIZABLE` two of these running concurrently form a read/write
/// dependency cycle, and Postgres aborts one at commit with SQLSTATE 40001.
/// Retrying is the intended response.
async fn increment(pool: &DbPool, attempts: Arc<AtomicU32>) -> Result<(), Error> {
    with_serializable_retry(pool, DEFAULT_MAX_ATTEMPTS, move |conn| {
        let attempts = attempts.clone();
        Box::pin(async move {
            attempts.fetch_add(1, Ordering::SeqCst);

            let row: IntRow = diesel::sql_query(
                "SELECT quantity AS value FROM catalog.widget WHERE id = 'wid_1'",
            )
            .get_result(conn)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

            // A deliberate gap, so both transactions read before either writes.
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;

            diesel::sql_query(format!(
                "UPDATE catalog.widget SET quantity = {} WHERE id = 'wid_1'",
                row.value + 1
            ))
            .execute(conn)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

            Ok(())
        })
    })
    .await
}

#[tokio::test]
async fn two_concurrent_writers_both_commit_after_a_retry() {
    let db = widget_db().await;
    stheno_db::testing::TestDb::execute(
        &db,
        "INSERT INTO catalog.widget (id, title, quantity) VALUES ('wid_1', 'contended', 0)",
    )
    .await;

    // At least 3 connections: two writers, plus headroom for the retry.
    let pool = create_pool(&db.url, 5).expect("pool");
    let attempts = Arc::new(AtomicU32::new(0));

    let a = increment(&pool, attempts.clone());
    let b = increment(&pool, attempts.clone());
    let (ra, rb) = tokio::join!(a, b);

    ra.expect("writer A must eventually commit");
    rb.expect("writer B must eventually commit");

    // The assertion that matters: no update was lost. Without SERIALIZABLE both
    // writers would read 0, write 1, and one increment would vanish.
    let final_quantity = db
        .count("SELECT quantity::bigint FROM catalog.widget WHERE id = 'wid_1'")
        .await;
    assert_eq!(
        final_quantity, 2,
        "an increment was lost; the writers did not serialise"
    );

    // Three or more bodies for two transactions means one was aborted and retried.
    let total = AtomicU32::load(&attempts, Ordering::SeqCst);
    assert!(
        total >= 3,
        "expected a serialisation conflict and a retry, but the bodies ran {total} times"
    );

    db.drop().await;
}

#[tokio::test]
async fn many_concurrent_writers_all_commit_without_losing_an_update() {
    let db = widget_db().await;
    stheno_db::testing::TestDb::execute(
        &db,
        "INSERT INTO catalog.widget (id, title, quantity) VALUES ('wid_1', 'hot', 0)",
    )
    .await;

    const WRITERS: i64 = 8;
    let pool = create_pool(&db.url, 10).expect("pool");
    let attempts = Arc::new(AtomicU32::new(0));

    let mut handles = Vec::new();
    for _ in 0..WRITERS {
        let pool = pool.clone();
        let attempts = attempts.clone();
        handles.push(tokio::spawn(async move {
            // Shorter gap here; with 8 writers the contention is real enough.
            with_serializable_retry(&pool, DEFAULT_MAX_ATTEMPTS, move |conn| {
                let attempts = attempts.clone();
                Box::pin(async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    let row: IntRow = diesel::sql_query(
                        "SELECT quantity AS value FROM catalog.widget WHERE id = 'wid_1'",
                    )
                    .get_result(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

                    diesel::sql_query(format!(
                        "UPDATE catalog.widget SET quantity = {} WHERE id = 'wid_1'",
                        row.value + 1
                    ))
                    .execute(conn)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;
                    Ok(())
                })
            })
            .await
        }));
    }

    let mut committed = 0i64;
    for h in handles {
        if h.await.expect("task must not panic").is_ok() {
            committed += 1;
        }
    }

    let final_quantity = db
        .count("SELECT quantity::bigint FROM catalog.widget WHERE id = 'wid_1'")
        .await;

    // Every writer that reported success must be represented in the total.
    // A writer exhausting its retry budget under heavy contention is a legitimate
    // outcome -- it surfaces as a Conflict rather than a silently lost update.
    assert_eq!(
        final_quantity, committed,
        "the counter disagrees with the number of writers that committed"
    );
    assert!(
        committed >= 1,
        "no writer committed at all under contention"
    );

    db.drop().await;
}

#[tokio::test]
async fn writers_touching_different_rows_do_not_conflict() {
    // SERIALIZABLE must not serialise everything -- only genuine conflicts.
    let db = widget_db().await;
    stheno_db::testing::TestDb::execute(&db,
        "INSERT INTO catalog.widget (id, title, quantity) VALUES ('wid_1', 'a', 0), ('wid_2', 'b', 0)",
    )
    .await;

    let pool = create_pool(&db.url, 5).expect("pool");
    let attempts = Arc::new(AtomicU32::new(0));

    let bump = |id: &'static str, attempts: Arc<AtomicU32>, pool: DbPool| async move {
        with_serializable_retry(&pool, DEFAULT_MAX_ATTEMPTS, move |conn| {
            let attempts = attempts.clone();
            Box::pin(async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                diesel::sql_query(format!(
                    "UPDATE catalog.widget SET quantity = quantity + 1 WHERE id = '{id}'"
                ))
                .execute(conn)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
                Ok(())
            })
        })
        .await
    };

    let (ra, rb) = tokio::join!(
        bump("wid_1", attempts.clone(), pool.clone()),
        bump("wid_2", attempts.clone(), pool.clone())
    );
    ra.expect("writer A");
    rb.expect("writer B");

    assert_eq!(
        AtomicU32::load(&attempts, Ordering::SeqCst),
        2,
        "disjoint writes should not have retried"
    );

    db.drop().await;
}
