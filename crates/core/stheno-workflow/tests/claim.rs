use serde_json::json;
use stheno_workflow::WorkflowStore;

#[test]
fn test_skip_locked_claim_loop_and_lease() {
    let store = WorkflowStore::new();
    let _ = store.get_or_create("tx_claim_1", "order_flow", json!({}));

    // Worker 1 claims with 5s lease
    assert!(store.try_claim("tx_claim_1", "worker_1", 5));

    // Worker 2 attempts concurrent claim while lease active -> fails
    assert!(!store.try_claim("tx_claim_1", "worker_2", 5));
}
