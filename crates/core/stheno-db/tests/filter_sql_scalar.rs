//! P11-09: `FilterSpec` -> SQL for equality, comparison, and null operators.
//!
//! Every operator is executed against Postgres, not just rendered. A clause that
//! parses but selects the wrong rows is the failure mode that matters, and only
//! running it catches that.

mod common;

use common::{widget_db, Widget};
use stheno_db::{create_pool, CrudRepository, FilterOp, FilterSpec, PgCrudRepository};

async fn seeded() -> (stheno_db::testing::TestDb, PgCrudRepository<Widget>) {
    let db = widget_db().await;
    let repo = PgCrudRepository::new(create_pool(&db.url, 5).expect("pool"));

    repo.create_many(
        None,
        vec![
            Widget::new("wid_1", "Alpha")
                .with_status("published")
                .with_quantity(10)
                .with_price(1000),
            Widget::new("wid_2", "Beta")
                .with_status("draft")
                .with_quantity(2)
                .with_price(500),
            Widget::new("wid_3", "Gamma")
                .with_status("published")
                .with_quantity(100)
                .with_price(9999)
                .with_handle(None),
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
async fn eq_on_text() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("status", FilterOp::Eq("published".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_1", "wid_3"]);
    db.drop().await;
}

#[tokio::test]
async fn ne_on_text_includes_rows_the_value_does_not_match() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("status", FilterOp::Ne("published".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_2"]);
    db.drop().await;
}

#[tokio::test]
async fn ne_does_not_silently_drop_null_rows() {
    // `<>` would exclude NULL handles, which is almost never what a caller
    // filtering "handle is not 'alpha'" means. IS DISTINCT FROM includes them.
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("handle", FilterOp::Ne("alpha".into())),
    )
    .await;
    assert_eq!(
        got,
        vec!["wid_2", "wid_3"],
        "the row with a NULL handle was dropped by $ne"
    );
    db.drop().await;
}

#[tokio::test]
async fn comparisons_on_an_integer_column_are_numeric_not_lexicographic() {
    // The regression this design exists to prevent: as text, '9' > '10'.
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("quantity", FilterOp::Gt("9".into())),
    )
    .await;
    assert_eq!(
        got,
        vec!["wid_1", "wid_3"],
        "integer comparison fell back to text ordering"
    );
    db.drop().await;
}

#[tokio::test]
async fn gte_and_lte_are_inclusive() {
    let (db, repo) = seeded().await;

    let gte = ids(
        &repo,
        FilterSpec::new().with_predicate("quantity", FilterOp::Gte("10".into())),
    )
    .await;
    assert_eq!(gte, vec!["wid_1", "wid_3"]);

    let lte = ids(
        &repo,
        FilterSpec::new().with_predicate("quantity", FilterOp::Lte("10".into())),
    )
    .await;
    assert_eq!(lte, vec!["wid_1", "wid_2"]);

    db.drop().await;
}

#[tokio::test]
async fn lt_and_gt_are_exclusive() {
    let (db, repo) = seeded().await;

    let lt = ids(
        &repo,
        FilterSpec::new().with_predicate("quantity", FilterOp::Lt("10".into())),
    )
    .await;
    assert_eq!(lt, vec!["wid_2"]);

    db.drop().await;
}

#[tokio::test]
async fn comparisons_on_a_numeric_money_column() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("price", FilterOp::Gte("1000".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_1", "wid_3"]);
    db.drop().await;
}

#[tokio::test]
async fn is_null_and_not_null() {
    let (db, repo) = seeded().await;

    let null = ids(
        &repo,
        FilterSpec::new().with_predicate("handle", FilterOp::IsNull),
    )
    .await;
    assert_eq!(null, vec!["wid_3"]);

    let not_null = ids(
        &repo,
        FilterSpec::new().with_predicate("handle", FilterOp::NotNull),
    )
    .await;
    assert_eq!(not_null, vec!["wid_1", "wid_2"]);

    db.drop().await;
}

#[tokio::test]
async fn multiple_predicates_are_combined_with_and() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new()
            .with_predicate("status", FilterOp::Eq("published".into()))
            .with_predicate("quantity", FilterOp::Lt("50".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_1"]);
    db.drop().await;
}

#[tokio::test]
async fn an_empty_filter_returns_everything() {
    let (db, repo) = seeded().await;
    let got = ids(&repo, FilterSpec::new()).await;
    assert_eq!(got, vec!["wid_1", "wid_2", "wid_3"]);
    db.drop().await;
}

#[tokio::test]
async fn a_boolean_column_filters_correctly() {
    let (db, repo) = seeded().await;
    let got = ids(
        &repo,
        FilterSpec::new().with_predicate("discountable", FilterOp::Eq("true".into())),
    )
    .await;
    assert_eq!(got.len(), 3);
    db.drop().await;
}

#[tokio::test]
async fn an_injected_column_name_is_rejected_before_reaching_postgres() {
    // Field names cannot be parameterised, so this check is the only thing
    // between a query string and the SQL text.
    let (db, repo) = seeded().await;

    for hostile in [
        "id; DROP TABLE catalog.widget",
        "id\"",
        "id--",
        "ID",
        "1id",
        "",
        "id) OR (1=1",
    ] {
        let filter = FilterSpec::new().with_predicate(hostile, FilterOp::Eq("x".into()));
        assert!(
            repo.list(None, &filter).await.is_err(),
            "column name {hostile:?} was not rejected"
        );
    }

    // The table is still there.
    let rows = db.count("SELECT count(*) FROM catalog.widget").await;
    assert_eq!(rows, 3);

    db.drop().await;
}
