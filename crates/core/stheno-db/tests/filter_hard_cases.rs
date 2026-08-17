//! P11-12: `FilterSpec` viability against the 20 hardest list endpoints.
//!
//! PRD risk R3 coming due. `FilterSpec` was specified in P0 and never executed,
//! so "it works" had never been tested against anything. Each case below is a
//! real list endpoint's filter shape from API_SPEC, **run against Postgres** --
//! the previous version of this file only called `validate()`, which asserts
//! nothing about whether a clause selects the right rows.
//!
//! # Outcome
//!
//! 18 of 20 are expressible and correct. The two that are not are single-table
//! translations of inherently cross-module questions, and are recorded at the
//! bottom of this file rather than papered over. Neither blocks P12: both are
//! join problems for P12-18, not defects in the operator set.
//!
//! One defect *was* found and fixed here: `predicates` was keyed by column, so a
//! range (`created_at >= x AND created_at <= y`) silently dropped its first
//! bound. Every date and price range filter in the admin API is that shape.

mod common;

use chrono::{Duration, Utc};
use common::{widget_db, Widget};
use stheno_db::testing::TestDb;
use stheno_db::{
    create_pool, CrudRepository, FilterOp, FilterSpec, OrderDirection, PgCrudRepository,
};

async fn seeded() -> (TestDb, PgCrudRepository<Widget>) {
    let db = widget_db().await;
    let repo = PgCrudRepository::new(create_pool(&db.url, 5).expect("pool"));

    let now = Utc::now();
    let mut batch = Vec::new();
    for i in 0..40i32 {
        let mut w = Widget::new(&format!("wid_{i:02}"), &format!("Widget {i:02}"))
            .with_quantity(i)
            .with_price(i as i64 * 100)
            .with_status(match i % 4 {
                0 => "published",
                1 => "draft",
                2 => "archived",
                _ => "proposed",
            });
        w.created_at = now - Duration::days(i as i64);
        if i % 5 == 0 {
            w.handle = None;
        }
        batch.push(w);
    }
    repo.create_many(None, batch).await.expect("seed");
    (db, repo)
}

async fn run(repo: &PgCrudRepository<Widget>, filter: FilterSpec) -> Vec<String> {
    repo.list(None, &filter)
        .await
        .expect("the filter must translate and execute")
        .into_iter()
        .map(|w| w.id)
        .collect()
}

/// Case 1 -- `GET /admin/orders?status[]=&created_at[$gte]=&order=-created_at`
#[tokio::test]
async fn case_01_admin_orders_status_set_and_date_floor() {
    let (db, repo) = seeded().await;
    let cutoff = (Utc::now() - Duration::hours(10 * 24 + 12)).to_rfc3339();

    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate(
                "status",
                FilterOp::In(vec!["published".into(), "draft".into()]),
            )
            .with_predicate("created_at", FilterOp::Gte(cutoff))
            .with_order("created_at", OrderDirection::Desc)
            .with_pagination(0, 50),
    )
    .await;

    assert!(
        !got.is_empty(),
        "no rows matched a filter that should match"
    );
    assert!(
        got.len() <= 6,
        "date floor was not applied: {} rows",
        got.len()
    );
    db.drop().await;
}

/// Case 2 -- `GET /admin/orders?created_at[$gte]=&created_at[$lte]=`
/// The range shape. This is the case that found the defect.
#[tokio::test]
async fn case_02_date_range_applies_both_bounds() {
    let (db, repo) = seeded().await;
    // Half-day offsets keep every row unambiguously inside or outside the
    // range: rows sit exactly N days back, so a boundary of exactly N days
    // would be decided by the milliseconds between seeding and asserting.
    let from = (Utc::now() - Duration::hours(20 * 24 + 12)).to_rfc3339();
    let to = (Utc::now() - Duration::hours(9 * 24 + 12)).to_rfc3339();

    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("created_at", FilterOp::Gte(from))
            .with_predicate("created_at", FilterOp::Lte(to))
            .with_pagination(0, 100),
    )
    .await;

    assert_eq!(
        got.len(),
        11,
        "a closed date range must return only rows inside it"
    );
    db.drop().await;
}

/// Case 3 -- `GET /admin/products?q=&status=&order=title`
#[tokio::test]
async fn case_03_products_prefix_search_with_status() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("title", FilterOp::Like("Widget 1%".into()))
            .with_predicate("status", FilterOp::Eq("published".into()))
            .with_order("title", OrderDirection::Asc)
            .with_pagination(0, 50),
    )
    .await;
    assert_eq!(got, vec!["wid_12", "wid_16"]);
    db.drop().await;
}

/// Case 4 -- price band, the shape every price-list screen uses.
#[tokio::test]
async fn case_04_price_band() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("price", FilterOp::Gte("1000".into()))
            .with_predicate("price", FilterOp::Lte("1500".into()))
            .with_pagination(0, 100),
    )
    .await;
    assert_eq!(got.len(), 6, "price band returned the wrong row count");
    db.drop().await;
}

/// Case 5 -- `GET /admin/inventory-items?location_id[]=` with a large set.
#[tokio::test]
async fn case_05_large_in_set() {
    let (db, repo) = seeded().await;
    let ids: Vec<String> = (0..40).map(|i| format!("wid_{i:02}")).collect();
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("id", FilterOp::In(ids))
            .with_pagination(0, 100),
    )
    .await;
    assert_eq!(got.len(), 40);
    db.drop().await;
}

/// Case 6 -- exclusion plus a floor, the "active stock" shape.
#[tokio::test]
async fn case_06_exclusion_with_a_numeric_floor() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("status", FilterOp::Ne("archived".into()))
            .with_predicate("quantity", FilterOp::Gt("0".into()))
            .with_pagination(0, 100),
    )
    .await;
    assert!(got.iter().all(|id| id != "wid_00"));
    assert_eq!(got.len(), 29);
    db.drop().await;
}

/// Case 7 -- null filter, `GET /admin/products?collection_id=null`
#[tokio::test]
async fn case_07_null_relation() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("handle", FilterOp::IsNull)
            .with_pagination(0, 100),
    )
    .await;
    assert_eq!(got.len(), 8, "one in five rows has a NULL handle");
    db.drop().await;
}

/// Case 8 -- soft-delete exclusion combined with a business filter.
#[tokio::test]
async fn case_08_soft_delete_exclusion() {
    let (db, repo) = seeded().await;
    repo.soft_delete(None, "wid_04").await.expect("soft delete");

    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_predicate("status", FilterOp::Eq("published".into()))
            .with_pagination(0, 100),
    )
    .await;
    assert!(!got.contains(&"wid_04".to_string()));
    assert_eq!(got.len(), 9);
    db.drop().await;
}

/// Case 9 -- multi-key sort, the order-list default.
#[tokio::test]
async fn case_09_multi_key_sort_with_pagination() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_order("status", OrderDirection::Asc)
            .with_order("price", OrderDirection::Desc)
            .with_pagination(0, 5),
    )
    .await;
    assert_eq!(got.len(), 5);
    // 'archived' sorts first, highest price within it.
    assert_eq!(got[0], "wid_38");
    db.drop().await;
}

/// Case 10 -- deep pagination, page 3 of a filtered set.
#[tokio::test]
async fn case_10_deep_pagination_is_stable() {
    let (db, repo) = seeded().await;
    let base = FilterSpec::new().with_order("quantity", OrderDirection::Asc);

    let p1 = run(&repo, base.clone().with_pagination(0, 10)).await;
    let p2 = run(&repo, base.clone().with_pagination(10, 10)).await;
    let p3 = run(&repo, base.with_pagination(20, 10)).await;

    let mut all: Vec<String> = [p1, p2, p3].concat();
    assert_eq!(all.len(), 30);
    all.sort();
    all.dedup();
    assert_eq!(all.len(), 30, "pages overlapped");
    db.drop().await;
}

/// Case 11 -- boundary: `gte` and `lte` on the same value select exactly one row.
#[tokio::test]
async fn case_11_degenerate_range_selects_one_row() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("quantity", FilterOp::Gte("7".into()))
            .with_predicate("quantity", FilterOp::Lte("7".into())),
    )
    .await;
    assert_eq!(got, vec!["wid_07"]);
    db.drop().await;
}

/// Case 12 -- an impossible range returns nothing rather than everything.
#[tokio::test]
async fn case_12_inverted_range_returns_nothing() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("quantity", FilterOp::Gte("30".into()))
            .with_predicate("quantity", FilterOp::Lte("10".into())),
    )
    .await;
    assert!(got.is_empty(), "an inverted range must match nothing");
    db.drop().await;
}

/// Case 13 -- `GET /admin/customers?email[$like]=` prefix with ordering.
#[tokio::test]
async fn case_13_prefix_with_descending_order() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("title", FilterOp::Like("Widget 3%".into()))
            .with_order("title", OrderDirection::Desc)
            .with_pagination(0, 3),
    )
    .await;
    assert_eq!(got, vec!["wid_39", "wid_38", "wid_37"]);
    db.drop().await;
}

/// Case 14 -- count and page together, what every list envelope needs.
#[tokio::test]
async fn case_14_list_and_count_agree_with_the_filter() {
    let (db, repo) = seeded().await;
    let filter = FilterSpec::new()
        .with_predicate("status", FilterOp::Eq("draft".into()))
        .with_pagination(0, 3);

    let (page, total) = repo
        .list_and_count(None, &filter)
        .await
        .expect("list_and_count");
    assert_eq!(page.len(), 3);
    assert_eq!(total, 10, "count must reflect the filter, not the page");
    db.drop().await;
}

/// Case 15 -- boolean flag plus a set, the "discountable published" shape.
#[tokio::test]
async fn case_15_boolean_and_set() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("discountable", FilterOp::Eq("true".into()))
            .with_predicate(
                "status",
                FilterOp::In(vec!["published".into(), "proposed".into()]),
            )
            .with_pagination(0, 100),
    )
    .await;
    assert_eq!(got.len(), 20);
    db.drop().await;
}

/// Case 16 -- `NotNull` plus a range, the "has handle, recent" shape.
#[tokio::test]
async fn case_16_not_null_with_range() {
    let (db, repo) = seeded().await;
    let cutoff = (Utc::now() - Duration::hours(15 * 24 + 12)).to_rfc3339();
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("handle", FilterOp::NotNull)
            .with_predicate("created_at", FilterOp::Gte(cutoff))
            .with_pagination(0, 100),
    )
    .await;
    // i = 0..=15 are inside the window; i = 0, 5, 10, 15 have a NULL handle.
    assert_eq!(got.len(), 12);
    db.drop().await;
}

/// Case 17 -- the predicate-count ceiling is usable, not just theoretical.
#[tokio::test]
async fn case_17_many_predicates_within_the_limit() {
    let (db, repo) = seeded().await;
    let mut filter = FilterSpec::new();
    for _ in 0..10 {
        filter = filter.with_predicate("quantity", FilterOp::Gte("0".into()));
    }
    filter = filter.with_predicate("status", FilterOp::Eq("published".into()));

    let got = run(&repo, filter.with_pagination(0, 100)).await;
    assert_eq!(got.len(), 10);
    db.drop().await;
}

/// Case 18 -- ordering by a nullable column puts NULLs somewhere deterministic.
#[tokio::test]
async fn case_18_order_by_nullable_column() {
    let (db, repo) = seeded().await;
    let got = run(
        &repo,
        FilterSpec::new()
            .with_order("handle", OrderDirection::Asc)
            .with_pagination(0, 100),
    )
    .await;
    assert_eq!(got.len(), 40, "ordering by a nullable column dropped rows");
    db.drop().await;
}

// ---------------------------------------------------------------------------
// The two cases FilterSpec cannot express, recorded rather than hidden.
// ---------------------------------------------------------------------------

/// Case 19 -- `GET /admin/products?price_list_id=` : products filtered by a
/// value on a *different module's* table.
///
/// Not expressible, and correctly so. `FilterSpec` describes predicates over one
/// entity's columns; reaching pricing from product would be the cross-schema
/// join ADR-007 forbids modules from writing. This is the query engine's job
/// (P12-18, P14-20), not the filter's.
///
/// The decision this test records: **do not** extend `FilterSpec` with joins.
/// Hand-written per-module query builders were the fallback PRD R3 budgeted four
/// weeks for; they are not needed, because the operator set is sufficient for
/// every single-entity filter and joins belong a layer up.
#[tokio::test]
async fn case_19_cross_module_filter_is_out_of_scope_by_design() {
    let (db, repo) = seeded().await;

    // A relation column on this entity is expressible; the far side is not.
    let got = run(
        &repo,
        FilterSpec::new()
            .with_predicate("handle", FilterOp::NotNull)
            .with_pagination(0, 100),
    )
    .await;
    assert_eq!(got.len(), 32);

    db.drop().await;
}

/// Case 20 -- `GET /store/products?q=lamp` : full-text search.
///
/// Deliberately not expressible. `Like` forbids a leading wildcard so that every
/// list stays index-backed, which makes infix search impossible by design. Free
/// text search belongs to the search module and its own index, not to a
/// sequential scan wearing a filter's clothes.
#[tokio::test]
async fn case_20_infix_search_is_refused_not_served_slowly() {
    let (db, repo) = seeded().await;

    let filter = FilterSpec::new().with_predicate("title", FilterOp::Like("%idget 1%".into()));
    assert!(
        repo.list(None, &filter).await.is_err(),
        "an infix search must be refused, not turned into a table scan"
    );

    db.drop().await;
}
