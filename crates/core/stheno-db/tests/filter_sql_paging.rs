//! P11-11: `FilterSpec` -> SQL for ordering, offset pagination, and `list_and_count`.

mod common;

use common::{widget_db, Widget};
use stheno_db::filter::{DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};
use stheno_db::{
    create_pool, CrudRepository, FilterOp, FilterSpec, OrderDirection, PgCrudRepository,
};

/// 30 widgets: quantity 0..30, alternating status.
async fn seeded() -> (stheno_db::testing::TestDb, PgCrudRepository<Widget>) {
    let db = widget_db().await;
    let repo = PgCrudRepository::new(create_pool(&db.url, 5).expect("pool"));

    let batch: Vec<Widget> = (0..30)
        .map(|i| {
            Widget::new(&format!("wid_{i:02}"), &format!("Widget {i:02}"))
                .with_quantity(i)
                .with_status(if i % 2 == 0 { "published" } else { "draft" })
        })
        .collect();
    repo.create_many(None, batch).await.expect("seed");

    (db, repo)
}

async fn ids(repo: &PgCrudRepository<Widget>, filter: &FilterSpec) -> Vec<String> {
    repo.list(None, filter)
        .await
        .expect("list")
        .into_iter()
        .map(|w| w.id)
        .collect()
}

#[tokio::test]
async fn order_by_ascending_and_descending() {
    let (db, repo) = seeded().await;

    let asc = ids(
        &repo,
        &FilterSpec::new()
            .with_order("quantity", OrderDirection::Asc)
            .with_pagination(0, 3),
    )
    .await;
    assert_eq!(asc, vec!["wid_00", "wid_01", "wid_02"]);

    let desc = ids(
        &repo,
        &FilterSpec::new()
            .with_order("quantity", OrderDirection::Desc)
            .with_pagination(0, 3),
    )
    .await;
    assert_eq!(desc, vec!["wid_29", "wid_28", "wid_27"]);

    db.drop().await;
}

#[tokio::test]
async fn multiple_order_keys_are_applied_in_sequence() {
    let (db, repo) = seeded().await;

    let got = ids(
        &repo,
        &FilterSpec::new()
            .with_order("status", OrderDirection::Asc)
            .with_order("quantity", OrderDirection::Desc)
            .with_pagination(0, 2),
    )
    .await;
    // 'draft' sorts before 'published', then highest quantity first.
    assert_eq!(got, vec!["wid_29", "wid_27"]);

    db.drop().await;
}

#[tokio::test]
async fn an_injected_order_column_is_rejected() {
    let (db, repo) = seeded().await;
    let filter =
        FilterSpec::new().with_order("quantity; DROP TABLE catalog.widget", OrderDirection::Asc);

    assert!(
        repo.list(None, &filter).await.is_err(),
        "an ORDER BY column name must be identifier-checked too"
    );
    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(rows, 30);

    db.drop().await;
}

#[tokio::test]
async fn offset_pagination_walks_the_whole_set_without_gaps_or_repeats() {
    let (db, repo) = seeded().await;

    let mut seen = Vec::new();
    for page in 0..3 {
        let got = ids(
            &repo,
            &FilterSpec::new()
                .with_order("quantity", OrderDirection::Asc)
                .with_pagination(page * 10, 10),
        )
        .await;
        assert_eq!(got.len(), 10, "page {page} was not full");
        seen.extend(got);
    }

    assert_eq!(seen.len(), 30);
    let mut deduped = seen.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(deduped.len(), 30, "pagination repeated a row");

    db.drop().await;
}

#[tokio::test]
async fn an_offset_past_the_end_returns_nothing() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        &FilterSpec::new()
            .with_order("quantity", OrderDirection::Asc)
            .with_pagination(500, 10),
    )
    .await;
    assert!(got.is_empty());
    db.drop().await;
}

#[tokio::test]
async fn the_default_page_limit_applies_when_none_is_given() {
    let (db, repo) = seeded().await;
    let got = ids(&repo, &FilterSpec::new()).await;
    assert_eq!(
        got.len(),
        DEFAULT_PAGE_LIMIT,
        "an unpaginated list must not return the whole table"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_page_limit_over_the_maximum_is_clamped() {
    let (db, repo) = seeded().await;
    let filter = FilterSpec::new().with_pagination(0, MAX_PAGE_LIMIT * 10);
    let got = ids(&repo, &filter).await;

    // Only 30 rows exist, so the clamp is asserted on the rendered SQL.
    let rendered = filter.to_sql("catalog.widget").expect("render");
    assert!(
        rendered.clause.contains(&format!("LIMIT {MAX_PAGE_LIMIT}")),
        "page limit was not clamped: {}",
        rendered.clause
    );
    assert_eq!(got.len(), 30);

    db.drop().await;
}

#[tokio::test]
async fn list_and_count_returns_the_page_and_the_unpaginated_total() {
    let (db, repo) = seeded().await;

    let filter = FilterSpec::new()
        .with_predicate("status", FilterOp::Eq("published".into()))
        .with_order("quantity", OrderDirection::Asc)
        .with_pagination(0, 5);

    let (page, total) = repo
        .list_and_count(None, &filter)
        .await
        .expect("list_and_count");

    assert_eq!(page.len(), 5, "the page should honour the limit");
    assert_eq!(
        total, 15,
        "the count must ignore pagination and report every matching row"
    );
    assert_eq!(page[0].id, "wid_00");

    db.drop().await;
}

#[tokio::test]
async fn list_and_count_total_ignores_offset() {
    let (db, repo) = seeded().await;

    let filter = FilterSpec::new()
        .with_order("quantity", OrderDirection::Asc)
        .with_pagination(25, 10);
    let (page, total) = repo
        .list_and_count(None, &filter)
        .await
        .expect("list_and_count");

    assert_eq!(page.len(), 5, "only 5 rows remain past offset 25");
    assert_eq!(total, 30, "the total must not be reduced by the offset");

    db.drop().await;
}

#[tokio::test]
async fn list_and_count_on_an_empty_result_reports_zero() {
    let (db, repo) = seeded().await;

    let filter = FilterSpec::new().with_predicate("status", FilterOp::Eq("nonexistent".into()));
    let (page, total) = repo
        .list_and_count(None, &filter)
        .await
        .expect("list_and_count");

    assert!(page.is_empty());
    assert_eq!(total, 0);

    db.drop().await;
}

#[tokio::test]
async fn the_count_query_carries_no_limit_or_offset() {
    let filter = FilterSpec::new()
        .with_predicate("status", FilterOp::Eq("published".into()))
        .with_order("quantity", OrderDirection::Asc)
        .with_pagination(10, 5);

    let counted = filter.to_count_sql("catalog.widget").expect("render");
    assert!(!counted.clause.contains("LIMIT"), "count query has a LIMIT");
    assert!(
        !counted.clause.contains("OFFSET"),
        "count query has an OFFSET"
    );
    assert!(
        !counted.clause.contains("ORDER BY"),
        "count query orders pointlessly"
    );
    assert!(
        counted.clause.contains("WHERE"),
        "count query lost its predicates"
    );
}
