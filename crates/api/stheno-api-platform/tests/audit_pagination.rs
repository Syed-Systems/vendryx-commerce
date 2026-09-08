//! The audit trail pages without losing or repeating history.
//!
//! An audit trail is the one list where a pagination bug is not cosmetic. A skipped row
//! is a change nobody can see happened; a repeated row is a change that looks like it
//! happened twice. Both are worse than a slow page.
//!
//! The trail also grows at exactly the end being read, which is what rules out `OFFSET`:
//! by the time page 2 is fetched, rows have been inserted above it and everything has
//! shifted down by that many. These assert the cursor does not have that property.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;
use stheno_api_core::TierConfig;
use stheno_api_platform::{platform_routes, PlatformApiContext};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::PlatformService;
use tower::ServiceExt;

const BOOTSTRAP: &str = "pagination-bootstrap-secret";
const TIERS_TOML: &str = include_str!("../../../../config/tiers.toml");

async fn app() -> (TestDb, axum::Router) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let router = platform_routes(PlatformApiContext {
        platform: Arc::new(PlatformService::new(pool)),
        tier_config: Arc::new(TierConfig::from_toml(TIERS_TOML).expect("tiers.toml parses")),
        operator_token: Some(BOOTSTRAP.to_string()),
        payment_service: None,
        fulfillment_service: None,
    });
    (db, router)
}

async fn call(
    router: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {BOOTSTRAP}"));
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };

    let response = router
        .clone()
        .oneshot(builder.body(body).expect("request builds"))
        .await
        .expect("router responds");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body reads");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Writes `n` audited actions, oldest first, numbered from `start` so two batches in one
/// test do not collide on a tenant id.
async fn write_actions(router: &axum::Router, start: usize, n: usize) {
    for i in start..start + n {
        let (status, body) = call(
            router,
            "POST",
            "/platform/tenants",
            Some(json!({ "id": format!("ten_p{i:03}"), "name": "P", "tier": "free" })),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "provisioning ten_p{i:03}: {body}"
        );
    }
}

/// One page: `(ids, next_cursor)`.
async fn page(router: &axum::Router, query: &str) -> (Vec<String>, Option<String>) {
    let (status, body) = call(router, "GET", &format!("/platform/audit?{query}"), None).await;
    assert_eq!(status, StatusCode::OK, "reading ?{query}: {body}");
    let ids = body["entries"]
        .as_array()
        .expect("entries is an array")
        .iter()
        .map(|e| e["id"].as_str().expect("row id").to_string())
        .collect();
    (ids, body["next_cursor"].as_str().map(|c| c.to_string()))
}

#[tokio::test]
async fn paging_the_whole_trail_yields_every_row_exactly_once() {
    let (db, router) = app().await;
    write_actions(&router, 0, 25).await;

    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages = 0;

    loop {
        let query = match &cursor {
            Some(c) => format!("limit=7&before={c}"),
            None => "limit=7".to_string(),
        };
        let (ids, next) = page(&router, &query).await;
        seen.extend(ids);
        pages += 1;
        assert!(pages < 20, "pagination did not terminate");

        match next {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    assert_eq!(
        seen.len(),
        25,
        "25 audited writes must page out as 25 rows, got {}",
        seen.len()
    );
    let unique: HashSet<&String> = seen.iter().collect();
    assert_eq!(
        unique.len(),
        seen.len(),
        "a repeated row makes one change look like two"
    );
    assert_eq!(pages, 4, "25 rows at 7 per page is 4 pages, got {pages}");

    db.drop().await;
}

#[tokio::test]
async fn a_row_written_between_pages_does_not_shift_the_next_one() {
    let (db, router) = app().await;
    write_actions(&router, 0, 10).await;

    let (first, cursor) = page(&router, "limit=5").await;
    assert_eq!(first.len(), 5);
    let cursor = cursor.expect("a full page hands back a cursor");

    // The exact race an OFFSET page has no answer for: five more rows arrive above the
    // cursor while the operator is reading page one. With `OFFSET 5` the next page would
    // re-serve rows the operator has already seen.
    write_actions(&router, 10, 5).await;

    let (second, _) = page(&router, &format!("limit=5&before={cursor}")).await;

    let first_set: HashSet<&String> = first.iter().collect();
    assert!(
        second.iter().all(|id| !first_set.contains(id)),
        "page two re-served a row from page one after new rows arrived; the cursor is \
         drifting.\npage one: {first:?}\npage two: {second:?}"
    );
    assert_eq!(
        second.len(),
        5,
        "the rows below the cursor are unaffected by rows written above it"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_trail_reads_newest_first() {
    let (db, router) = app().await;
    write_actions(&router, 0, 5).await;

    let (status, body) = call(&router, "GET", "/platform/audit", None).await;
    assert_eq!(status, StatusCode::OK);
    let entities: Vec<&str> = body["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|e| e["entity_id"].as_str().expect("entity_id"))
        .collect();

    assert_eq!(
        entities.first(),
        Some(&"ten_p004"),
        "the most recent action must be first; an operator opening this page is asking \
         what just happened, not what happened first. Got {entities:?}"
    );

    // Ids sort descending, which is the property the cursor depends on.
    let ids: Vec<&str> = body["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|e| e["id"].as_str().expect("id"))
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(
        ids, sorted,
        "rows must come back in descending id order, or `id < cursor` is not a boundary"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_short_page_hands_back_no_cursor() {
    let (db, router) = app().await;
    write_actions(&router, 0, 3).await;

    let (ids, cursor) = page(&router, "limit=50").await;
    assert_eq!(ids.len(), 3);
    assert!(
        cursor.is_none(),
        "a page that came back short is the end of the trail; handing back a cursor \
         there makes the caller fetch an empty page to learn what this response knew"
    );

    // The exact-fit case is the one that legitimately hands back a cursor it does not
    // need — a following empty page is the honest cost of not counting the whole table.
    let (ids, cursor) = page(&router, "limit=3").await;
    assert_eq!(ids.len(), 3);
    let cursor = cursor.expect("a full page always offers a cursor");
    let (ids, next) = page(&router, &format!("limit=3&before={cursor}")).await;
    assert!(ids.is_empty(), "the trail is exhausted");
    assert!(next.is_none());

    db.drop().await;
}

#[tokio::test]
async fn a_page_size_beyond_the_cap_is_clamped_not_refused() {
    let (db, router) = app().await;
    write_actions(&router, 0, 3).await;

    // MAX_PAGE_LIMIT is 100. An operator asking for 10_000 gets a capped page, not a 500
    // and not the whole table — the cap is a defence against one request reading an
    // unbounded trail into memory.
    let (status, body) = call(&router, "GET", "/platform/audit?limit=10000", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entries"].as_array().expect("entries").len(), 3);

    // Zero would otherwise be an infinite loop for any caller paging until empty.
    let (status, body) = call(&router, "GET", "/platform/audit?limit=0", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["entries"].as_array().expect("entries").len(),
        1,
        "limit=0 is clamped up to 1, never served as an empty page forever"
    );

    db.drop().await;
}

#[tokio::test]
async fn narrowing_and_paging_compose() {
    let (db, router) = app().await;

    // Six writes against one merchant, interleaved with writes against another, so a
    // filter that leaked would be caught by the count rather than by luck.
    for id in ["ten_alpha", "ten_beta"] {
        call(
            &router,
            "POST",
            "/platform/tenants",
            Some(json!({ "id": id, "name": id, "tier": "free" })),
        )
        .await;
    }
    for i in 0..5 {
        let tier = if i % 2 == 0 { "pro" } else { "free" };
        for id in ["ten_alpha", "ten_beta"] {
            call(
                &router,
                "POST",
                &format!("/platform/tenants/{id}/tier"),
                Some(json!({ "tier": tier })),
            )
            .await;
        }
    }

    let mut seen = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let query = match &cursor {
            Some(c) => format!("entity_id=ten_alpha&limit=2&before={c}"),
            None => "entity_id=ten_alpha&limit=2".to_string(),
        };
        let (status, body) = call(&router, "GET", &format!("/platform/audit?{query}"), None).await;
        assert_eq!(status, StatusCode::OK);

        for entry in body["entries"].as_array().expect("entries") {
            assert_eq!(
                entry["entity_id"], "ten_alpha",
                "paging a narrowed trail must never spill another merchant's history"
            );
            seen.push(entry["id"].as_str().expect("id").to_string());
        }

        match body["next_cursor"].as_str() {
            Some(next) => cursor = Some(next.to_string()),
            None => break,
        }
        assert!(seen.len() < 50, "pagination did not terminate");
    }

    assert_eq!(
        seen.len(),
        6,
        "one create plus five tier moves is six rows for ten_alpha, got {}",
        seen.len()
    );

    db.drop().await;
}
