use std::collections::HashSet;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::tenant;
use stheno_db::testing::TestDb;
use stheno_order::OrderService;
use tokio::task;

/// Order numbers must be unique, gap-free, and survive a restart.
///
/// The original of this test spawned 50 tasks against an `Arc<AtomicI64>` and proved the
/// atomic worked. It could not fail, because the thing it exercised was one `fetch_add` —
/// while the actual defect was that the counter lived in process memory at all. A restart
/// reset it to 1000 and the next order collided with one already stored, which surfaced as
/// a duplicate-key error on checkout.
#[tokio::test]
async fn display_ids_are_unique_and_gap_free_under_50_way_concurrency() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 10).expect("pool");
    let service = Arc::new(OrderService::new(pool));

    let mut handles = Vec::new();
    for _ in 0..50 {
        let s = service.clone();
        handles.push(task::spawn(tenant::scope(
            tenant::current().unwrap_or_else(|_| "ten_default".to_string()),
            async move { s.next_display_id().await },
        )));
    }

    let mut ids = Vec::new();
    for handle in handles {
        ids.push(handle.await.expect("task join").expect("display id"));
    }

    assert_eq!(ids.len(), 50);

    // Two orders sharing a number is the failure this exists to catch.
    let unique: HashSet<i64> = ids.iter().copied().collect();
    assert_eq!(unique.len(), 50, "duplicate display_id issued");

    // Gap-free: the row lock must serialise the 50 callers, not let them interleave.
    ids.sort_unstable();
    let min = ids[0];
    for (index, &id) in ids.iter().enumerate() {
        assert_eq!(id, min + index as i64, "gap in the display_id sequence");
    }
}

/// The point of moving the counter into the database.
#[tokio::test]
async fn display_ids_do_not_restart_when_the_process_does() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let first = {
        let service = OrderService::new(pool.clone());
        let mut issued = Vec::new();
        for _ in 0..3 {
            issued.push(service.next_display_id().await.expect("display id"));
        }
        issued
    };

    // A second service is what a restarted process looks like from the counter's side:
    // fresh in-memory state, same database.
    let service = OrderService::new(pool);
    let next = service.next_display_id().await.expect("display id");

    assert_eq!(
        next,
        first[2] + 1,
        "a new OrderService reissued an order number that was already used"
    );
}
