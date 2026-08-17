//! P11-07: `PgCrudRepository` retrieve, create, update, delete against real Postgres.

mod common;

use common::{widget_db, Widget};
use stheno_db::{create_pool, CrudRepository, PgCrudRepository};

async fn repo(url: &str) -> PgCrudRepository<Widget> {
    PgCrudRepository::new(create_pool(url, 5).expect("pool"))
}

#[tokio::test]
async fn create_then_retrieve_returns_the_same_row() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let created = repo
        .create(None, Widget::new("wid_1", "Desk Lamp").with_price(1250))
        .await
        .expect("create");

    assert_eq!(created.id, "wid_1");
    assert_eq!(created.title, "Desk Lamp");

    let fetched = repo.retrieve(None, "wid_1").await.expect("retrieve");
    assert_eq!(
        fetched, created,
        "the row read back differs from the row written"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_row_is_actually_in_the_database() {
    // Guards against a repository that round-trips through its own cache.
    let db = widget_db().await;
    let repo = repo(&db.url).await;
    repo.create(None, Widget::new("wid_1", "Desk Lamp"))
        .await
        .expect("create");

    let count = db
        .count("SELECT count(*) FROM catalog.widget WHERE id = 'wid_1'")
        .await;
    assert_eq!(count, 1, "create did not write a row");

    db.drop().await;
}

#[tokio::test]
async fn retrieving_an_unknown_id_is_not_found() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let err = repo
        .retrieve(None, "wid_missing")
        .await
        .expect_err("must not invent a row");
    assert!(
        err.to_string().contains("wid_missing"),
        "the error should name the id, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn creating_a_duplicate_id_is_a_conflict_not_a_crash() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_1", "First"))
        .await
        .expect("first create");
    let err = repo
        .create(None, Widget::new("wid_1", "Second"))
        .await
        .expect_err("a duplicate id must be rejected");

    // A duplicate id is a caller mistake: 409, never 500.
    assert!(
        matches!(err, stheno_types::Error::Conflict { .. }),
        "expected a Conflict, got: {err:?}"
    );

    db.drop().await;
}

#[tokio::test]
async fn update_changes_every_field_and_returns_the_new_row() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let original = repo
        .create(None, Widget::new("wid_1", "Old Title").with_quantity(1))
        .await
        .expect("create");

    let mut changed = original.clone();
    changed.title = "New Title".to_string();
    changed.quantity = 99;
    changed.status = "published".to_string();

    let updated = repo.update(None, "wid_1", changed).await.expect("update");
    assert_eq!(updated.title, "New Title");
    assert_eq!(updated.quantity, 99);
    assert_eq!(updated.status, "published");

    let refetched = repo.retrieve(None, "wid_1").await.expect("retrieve");
    assert_eq!(refetched, updated, "the update did not persist");

    db.drop().await;
}

#[tokio::test]
async fn updating_an_unknown_id_is_not_found() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let err = repo
        .update(None, "wid_missing", Widget::new("wid_missing", "Ghost"))
        .await
        .expect_err("updating a missing row must fail");
    assert!(err.to_string().contains("wid_missing"), "got: {err}");

    // And it must not have inserted anything as a side effect.
    let count = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(count, 0, "a failed update inserted a row");

    db.drop().await;
}

#[tokio::test]
async fn delete_removes_the_row() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_1", "Doomed"))
        .await
        .expect("create");
    repo.delete(None, "wid_1").await.expect("delete");

    let count = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(count, 0, "delete left the row behind");

    assert!(
        repo.retrieve(None, "wid_1").await.is_err(),
        "a deleted row must not be retrievable"
    );

    db.drop().await;
}

#[tokio::test]
async fn deleting_an_unknown_id_is_not_found() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let err = repo
        .delete(None, "wid_missing")
        .await
        .expect_err("deleting nothing must not report success");
    assert!(err.to_string().contains("wid_missing"), "got: {err}");

    db.drop().await;
}

#[tokio::test]
async fn money_round_trips_without_precision_loss() {
    // ADR-006: money is decimal. A repository that serialises through f64 loses
    // this, and a payment path is the worst place to discover it.
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let mut w = Widget::new("wid_1", "Precise");
    w.price = "1234567.891234".parse().expect("decimal");

    let created = repo.create(None, w.clone()).await.expect("create");
    assert_eq!(created.price, w.price);

    let fetched = repo.retrieve(None, "wid_1").await.expect("retrieve");
    assert_eq!(
        fetched.price.to_string(),
        "1234567.891234",
        "numeric precision was lost through the database"
    );

    db.drop().await;
}

#[tokio::test]
async fn nulls_round_trip_as_none() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_1", "No Handle").with_handle(None))
        .await
        .expect("create");

    let fetched = repo.retrieve(None, "wid_1").await.expect("retrieve");
    assert_eq!(
        fetched.handle, None,
        "a NULL column came back as something else"
    );

    db.drop().await;
}
