//! P11-10: `FilterSpec` -> SQL for `in`, `like`, range, and negation operators.

mod common;

use common::{widget_db, Widget};
use stheno_db::filter::MAX_IN_ELEMENTS;
use stheno_db::{create_pool, CrudRepository, FilterOp, FilterSpec, PgCrudRepository};

async fn seeded() -> (stheno_db::testing::TestDb, PgCrudRepository<Widget>) {
    let db = widget_db().await;
    let repo = PgCrudRepository::new(create_pool(&db.url, 5).expect("pool"));

    repo.create_many(
        None,
        vec![
            Widget::new("wid_1", "Pro Desk")
                .with_status("published")
                .with_quantity(10),
            Widget::new("wid_2", "Pro Chair")
                .with_status("draft")
                .with_quantity(20),
            Widget::new("wid_3", "Basic Lamp")
                .with_status("archived")
                .with_quantity(30),
            Widget::new("wid_4", "Professional Rug")
                .with_status("published")
                .with_quantity(40),
        ],
    )
    .await
    .expect("seed");

    (db, repo)
}

async fn ids(repo: &PgCrudRepository<Widget>, filter: FilterSpec) -> Vec<String> {
    let mut out: Vec<String> = repo
        .list(None, &filter)
        .await
        .expect("list")
        .into_iter()
        .map(|w| w.id)
        .collect();
    out.sort();
    out
}

#[tokio::test]
async fn in_matches_any_listed_value() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate(
            "status",
            FilterOp::In(vec!["published".into(), "archived".into()]),
        ),
    )
    .await;
    assert_eq!(got, vec!["wid_1", "wid_3", "wid_4"]);
    db.drop().await;
}

#[tokio::test]
async fn in_on_an_integer_column_coerces_each_element() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("quantity", FilterOp::In(vec!["10".into(), "30".into()])),
    )
    .await;
    assert_eq!(got, vec!["wid_1", "wid_3"]);
    db.drop().await;
}

#[tokio::test]
async fn in_with_no_values_matches_nothing() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("status", FilterOp::In(vec![])),
    )
    .await;
    assert!(
        got.is_empty(),
        "an empty $in must match nothing, not everything"
    );
    db.drop().await;
}

#[tokio::test]
async fn in_at_the_element_limit_is_accepted() {
    let (db, repo) = seeded().await;
    let mut values: Vec<String> = (0..MAX_IN_ELEMENTS - 1).map(|i| format!("s{i}")).collect();
    values.push("published".into());

    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("status", FilterOp::In(values)),
    )
    .await;
    assert_eq!(got, vec!["wid_1", "wid_4"]);
    db.drop().await;
}

#[tokio::test]
async fn in_over_the_element_limit_is_rejected() {
    let (db, repo) = seeded().await;
    let values: Vec<String> = (0..=MAX_IN_ELEMENTS).map(|i| format!("s{i}")).collect();
    let filter = FilterSpec::new().with_predicate("status", FilterOp::In(values));

    assert!(
        repo.list(None, &filter).await.is_err(),
        "an oversized $in must be rejected, not executed"
    );
    db.drop().await;
}

#[tokio::test]
async fn like_matches_a_trailing_wildcard() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("title", FilterOp::Like("Pro%".into())),
    )
    .await;
    assert_eq!(
        got,
        vec!["wid_1", "wid_2", "wid_4"],
        "prefix LIKE did not match every prefixed title"
    );
    db.drop().await;
}

#[tokio::test]
async fn like_without_a_wildcard_is_an_exact_match() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("title", FilterOp::Like("Pro Desk".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_1"]);
    db.drop().await;
}

#[tokio::test]
async fn a_leading_wildcard_is_rejected_to_protect_the_index() {
    // A leading `%` cannot use a btree index and turns every list into a
    // sequential scan, so it is refused rather than served slowly.
    let (db, repo) = seeded().await;

    for pattern in ["%Desk", "%Pro%", "*Desk"] {
        let filter = FilterSpec::new().with_predicate("title", FilterOp::Like(pattern.into()));
        assert!(
            repo.list(None, &filter).await.is_err(),
            "leading wildcard {pattern:?} was not rejected"
        );
    }
    db.drop().await;
}

#[tokio::test]
async fn like_metacharacters_in_the_value_do_not_escape_the_pattern() {
    let (db, repo) = seeded().await;
    // Underscore is a single-character wildcard in LIKE. It must still be bound
    // as a value, not spliced into SQL.
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("title", FilterOp::Like("Pro_Desk".into())),
    )
    .await;
    assert_eq!(
        got,
        vec!["wid_1"],
        "the pattern was not bound as a parameter"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_range_applies_both_bounds_on_one_column() {
    // gte and lte on the same column is the shape every price and date filter
    // in the admin API uses. While `predicates` was keyed by column the second
    // bound silently replaced the first, so every range filter returned a
    // half-open set (P11-12).
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new()
            .with_predicate("quantity", FilterOp::Gte("20".into()))
            .with_predicate("quantity", FilterOp::Lte("30".into())),
    )
    .await;

    assert_eq!(
        got,
        vec!["wid_2", "wid_3"],
        "both bounds of the range must be applied"
    );
    db.drop().await;
}

#[tokio::test]
async fn negation_combines_with_a_set_filter() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new()
            .with_predicate("status", FilterOp::Ne("draft".into()))
            .with_predicate("quantity", FilterOp::Gte("30".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_3", "wid_4"]);
    db.drop().await;
}

#[tokio::test]
async fn too_many_predicates_are_rejected() {
    let (db, repo) = seeded().await;
    let mut filter = FilterSpec::new();
    for i in 0..=stheno_db::filter::MAX_PREDICATES {
        filter = filter.with_predicate(format!("col_{i}"), FilterOp::Eq("x".into()));
    }
    assert!(
        repo.list(None, &filter).await.is_err(),
        "a filter over the predicate limit must be rejected"
    );
    db.drop().await;
}
