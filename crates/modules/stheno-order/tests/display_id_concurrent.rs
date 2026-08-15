use std::collections::HashSet;
use std::sync::Arc;
use stheno_order::OrderService;
use tokio::task;

#[tokio::test]
async fn test_sequential_gap_free_display_id_under_50_way_concurrency() {
    let service = Arc::new(OrderService::new());
    let mut handles = Vec::new();

    // Spawn 50 parallel concurrent tasks acquiring display_ids
    for _ in 0..50 {
        let s = service.clone();
        handles.push(task::spawn(async move { s.next_display_id() }));
    }

    let mut ids = Vec::new();
    for handle in handles {
        let id = handle.await.expect("task join");
        ids.push(id);
    }

    assert_eq!(ids.len(), 50);

    // Verify all 50 IDs are strictly unique (no duplicate)
    let unique_set: HashSet<i64> = ids.iter().cloned().collect();
    assert_eq!(unique_set.len(), 50);

    // Verify they form a strictly gap-free sequential interval [min..min+50)
    ids.sort();
    let min_id = ids[0];
    for (idx, &id) in ids.iter().enumerate() {
        assert_eq!(
            id,
            min_id + idx as i64,
            "gap detected in sequential display_id"
        );
    }
}
