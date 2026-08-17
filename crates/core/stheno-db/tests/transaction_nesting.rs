//! P11-13: `with_savepoint` under three levels of nesting with partial rollback.
//!
//! The previous version of this file built a connection pool and asserted the
//! builder returned `Ok`. It opened no transaction and touched no database.

mod common;

use common::{widget_db, Widget};
use stheno_db::{with_savepoint, with_transaction, CrudRepository, InMemoryCrudStore};
use stheno_types::Error;

/// Inserts a widget on the given connection, inside whatever transaction is open.
async fn insert(conn: &mut diesel_async::AsyncPgConnection, id: &str) -> Result<(), Error> {
    use diesel_async::RunQueryDsl;
    diesel::sql_query(format!(
        "INSERT INTO catalog.widget (id, title) VALUES ('{id}', '{id}')"
    ))
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(|e| Error::Database(e.to_string()))
}

#[tokio::test]
async fn a_released_savepoint_keeps_its_writes() {
    let db = widget_db().await;
    let mut conn = db.conn().await;

    with_transaction(&mut conn, |c| {
        Box::pin(async move {
            insert(c, "outer").await?;
            with_savepoint(c, "sp1", |c| {
                Box::pin(async move { insert(c, "inner").await })
            })
            .await?;
            Ok(())
        })
    })
    .await
    .expect("transaction");

    assert_eq!(db.count("SELECT count(*) FROM catalog.widget").await, 2);
    db.drop().await;
}

#[tokio::test]
async fn a_rolled_back_savepoint_discards_only_its_own_writes() {
    // The whole point of a savepoint: the outer work survives the inner failure.
    let db = widget_db().await;
    let mut conn = db.conn().await;

    with_transaction(&mut conn, |c| {
        Box::pin(async move {
            insert(c, "outer").await?;

            let inner = with_savepoint(c, "sp1", |c| {
                Box::pin(async move {
                    insert(c, "doomed").await?;
                    Err::<(), Error>(Error::Database("deliberate failure".into()))
                })
            })
            .await;
            assert!(inner.is_err(), "the inner block should have failed");

            // The outer transaction is still usable after the rollback.
            insert(c, "after").await?;
            Ok(())
        })
    })
    .await
    .expect("outer transaction must still commit");

    let ids = db
        .count("SELECT count(*) FROM catalog.widget WHERE id IN ('outer', 'after')")
        .await;
    assert_eq!(ids, 2, "outer writes were lost");

    let doomed = db
        .count("SELECT count(*) FROM catalog.widget WHERE id = 'doomed'")
        .await;
    assert_eq!(doomed, 0, "the rolled-back write survived");

    db.drop().await;
}

#[tokio::test]
async fn three_levels_of_nesting_roll_back_independently() {
    let db = widget_db().await;
    let mut conn = db.conn().await;

    with_transaction(&mut conn, |c| {
        Box::pin(async move {
            insert(c, "level0").await?;

            with_savepoint(c, "sp1", |c| {
                Box::pin(async move {
                    insert(c, "level1").await?;

                    with_savepoint(c, "sp2", |c| {
                        Box::pin(async move {
                            insert(c, "level2").await?;

                            // Level 3 fails; levels 0, 1 and 2 must survive.
                            let deepest = with_savepoint(c, "sp3", |c| {
                                Box::pin(async move {
                                    insert(c, "level3").await?;
                                    Err::<(), Error>(Error::Database("deepest fails".into()))
                                })
                            })
                            .await;
                            assert!(deepest.is_err());
                            Ok::<(), Error>(())
                        })
                    })
                    .await?;
                    Ok::<(), Error>(())
                })
            })
            .await?;
            Ok(())
        })
    })
    .await
    .expect("outer transaction");

    for kept in ["level0", "level1", "level2"] {
        let n = db
            .count(&format!(
                "SELECT count(*) FROM catalog.widget WHERE id = '{kept}'"
            ))
            .await;
        assert_eq!(n, 1, "{kept} should have survived");
    }
    let discarded = db
        .count("SELECT count(*) FROM catalog.widget WHERE id = 'level3'")
        .await;
    assert_eq!(discarded, 0, "level3 should have been rolled back");

    db.drop().await;
}

#[tokio::test]
async fn a_failing_outer_transaction_discards_committed_savepoints_too() {
    // A released savepoint is not a commit. If the outer transaction aborts,
    // everything goes -- which is what makes a SAGA's compensation stack the
    // right tool for work that must survive that.
    let db = widget_db().await;
    let mut conn = db.conn().await;

    let result = with_transaction(&mut conn, |c| {
        Box::pin(async move {
            insert(c, "outer").await?;
            with_savepoint(c, "sp1", |c| {
                Box::pin(async move { insert(c, "inner").await })
            })
            .await?;
            Err::<(), Error>(Error::Database("outer fails".into()))
        })
    })
    .await;

    assert!(result.is_err());
    assert_eq!(
        db.count("SELECT count(*) FROM catalog.widget").await,
        0,
        "an aborted outer transaction left rows behind"
    );

    db.drop().await;
}

#[tokio::test]
async fn an_injected_savepoint_name_is_rejected() {
    let db = widget_db().await;
    let mut conn = db.conn().await;

    let result = with_savepoint(&mut conn, "sp1; DROP TABLE catalog.widget", |c| {
        Box::pin(async move { insert(c, "x").await })
    })
    .await;

    assert!(result.is_err(), "a hostile savepoint name must be rejected");
    // The table is still there.
    assert_eq!(db.count("SELECT count(*) FROM catalog.widget").await, 0);

    db.drop().await;
}

#[tokio::test]
async fn the_in_memory_store_still_satisfies_the_trait() {
    // `InMemoryCrudStore` remains the fixture backend for pure domain tests, so
    // it must keep working while modules migrate.
    let store: InMemoryCrudStore<Widget> = InMemoryCrudStore::default();
    let created = store
        .create(None, Widget::new("wid_1", "In memory"))
        .await
        .expect("create");
    assert_eq!(created.id, "wid_1");
    assert!(store.retrieve(None, "wid_1").await.is_ok());
}
