//! P11-08: `PgCrudRepository` soft_delete, restore, upsert, create_many.

mod common;

use common::{widget_db, Widget};
use stheno_db::{create_pool, CrudRepository, FilterSpec, PgCrudRepository};

async fn repo(url: &str) -> PgCrudRepository<Widget> {
    PgCrudRepository::new(create_pool(url, 5).expect("pool"))
}

#[tokio::test]
async fn soft_delete_hides_the_row_but_keeps_it() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_1", "Retired"))
        .await
        .expect("create");
    repo.soft_delete(None, "wid_1").await.expect("soft delete");

    assert!(
        repo.retrieve(None, "wid_1").await.is_err(),
        "a soft-deleted row must not be retrievable"
    );

    // The distinction from a hard delete: the row is still there.
    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(rows, 1, "soft delete removed the row entirely");

    let marked = db
        .count("SELECT count(*) FROM catalog.widget WHERE deleted_at IS NOT NULL")
        .await;
    assert_eq!(marked, 1, "soft delete did not set deleted_at");

    db.drop().await;
}

#[tokio::test]
async fn soft_deleted_rows_are_excluded_from_list() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_1", "Live"))
        .await
        .expect("create");
    repo.create(None, Widget::new("wid_2", "Gone"))
        .await
        .expect("create");
    repo.soft_delete(None, "wid_2").await.expect("soft delete");

    let filter = FilterSpec::new().with_predicate("deleted_at", stheno_db::FilterOp::IsNull);
    let listed = repo.list(None, &filter).await.expect("list");
    assert_eq!(listed.len(), 1, "a soft-deleted row was listed");
    assert_eq!(listed[0].id, "wid_1");

    db.drop().await;
}

#[tokio::test]
async fn restore_brings_a_soft_deleted_row_back() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_1", "Returning"))
        .await
        .expect("create");
    repo.soft_delete(None, "wid_1").await.expect("soft delete");
    repo.restore(None, "wid_1").await.expect("restore");

    let fetched = repo.retrieve(None, "wid_1").await.expect("retrieve");
    assert_eq!(fetched.deleted_at, None, "restore left deleted_at set");
    assert_eq!(fetched.title, "Returning");

    db.drop().await;
}

#[tokio::test]
async fn soft_delete_and_restore_of_an_unknown_id_are_not_found() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    assert!(repo.soft_delete(None, "wid_missing").await.is_err());
    assert!(repo.restore(None, "wid_missing").await.is_err());

    db.drop().await;
}

#[tokio::test]
async fn upsert_inserts_when_absent() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let out = repo
        .upsert(None, Widget::new("wid_1", "Fresh").with_quantity(5))
        .await
        .expect("upsert");
    assert_eq!(out.quantity, 5);

    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(rows, 1);

    db.drop().await;
}

#[tokio::test]
async fn upsert_updates_when_present_without_duplicating() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.upsert(None, Widget::new("wid_1", "First").with_quantity(1))
        .await
        .expect("first upsert");
    let second = repo
        .upsert(None, Widget::new("wid_1", "Second").with_quantity(2))
        .await
        .expect("second upsert");

    assert_eq!(second.title, "Second");
    assert_eq!(second.quantity, 2);

    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(rows, 1, "upsert inserted a duplicate instead of updating");

    db.drop().await;
}

#[tokio::test]
async fn create_many_inserts_every_item() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let batch = vec![
        Widget::new("wid_1", "One"),
        Widget::new("wid_2", "Two"),
        Widget::new("wid_3", "Three"),
    ];
    let created = repo.create_many(None, batch).await.expect("create_many");
    assert_eq!(created.len(), 3);

    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(rows, 3);

    db.drop().await;
}

#[tokio::test]
async fn create_many_is_all_or_nothing() {
    // A batch with one duplicate must insert none of it. A partial batch leaves
    // the caller unable to retry without first working out what landed.
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    repo.create(None, Widget::new("wid_2", "Existing"))
        .await
        .expect("seed");

    let batch = vec![
        Widget::new("wid_1", "New"),
        Widget::new("wid_2", "Duplicate"),
        Widget::new("wid_3", "Also New"),
    ];
    assert!(
        repo.create_many(None, batch).await.is_err(),
        "a batch containing a duplicate must fail"
    );

    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(
        rows, 1,
        "a failed batch left rows behind; it was not atomic"
    );

    db.drop().await;
}

#[tokio::test]
async fn create_many_of_nothing_is_a_no_op() {
    let db = widget_db().await;
    let repo = repo(&db.url).await;

    let created = repo
        .create_many(None, Vec::new())
        .await
        .expect("empty batch");
    assert!(created.is_empty());

    db.drop().await;
}
