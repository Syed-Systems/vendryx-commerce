//! P13-03: Lease acquisition, contention, renewal, and expiry across two independent processes/workers.

use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_workflow::{TransactionState, WorkflowStore};

#[tokio::test]
async fn lease_acquisition_and_expiry_contention() {
    let db = TestDb::migrated().await;
    let pool_a = create_pool(&db.url, 5).expect("create pool A");
    let pool_b = create_pool(&db.url, 5).expect("create pool B");

    // Two independent worker stores simulating separate processes
    let worker_a = WorkflowStore::from_pool(pool_a);
    let worker_b = WorkflowStore::from_pool(pool_b);

    let tx_id = "tx_contention_1";
    worker_a
        .get_or_create_async(tx_id, "batch-job", json!({ "batch_id": "b1" }))
        .await
        .expect("create execution");
    worker_a
        .update_state_async(tx_id, TransactionState::Invoking)
        .await
        .expect("invoking state");

    // 1. Worker A acquires 2-second lease
    let claim_a = worker_a
        .try_claim_async(tx_id, "worker_proc_a", 2)
        .await
        .expect("claim A");
    assert!(claim_a, "Worker A must acquire free lease");

    // 2. Worker B attempts to claim while lease is held by A -> must fail
    let claim_b = worker_b
        .try_claim_async(tx_id, "worker_proc_b", 2)
        .await
        .expect("claim B");
    assert!(
        !claim_b,
        "Worker B must be rejected while lease is held by A"
    );

    // 3. Worker A renews its own lease
    let renew_a = worker_a
        .try_claim_async(tx_id, "worker_proc_a", 1)
        .await
        .expect("renew A");
    assert!(renew_a, "Worker A must be able to renew its own lease");

    // 4. Wait for 1-second lease to expire
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    // 5. Worker B claims the expired lease -> must succeed
    let claim_b_expired = worker_b
        .try_claim_async(tx_id, "worker_proc_b", 5)
        .await
        .expect("claim B after expiry");
    assert!(
        claim_b_expired,
        "Worker B must acquire lease once Worker A's lease expired"
    );

    // 6. Worker A cannot claim or renew now because Worker B owns it
    let claim_a_now = worker_a
        .try_claim_async(tx_id, "worker_proc_a", 5)
        .await
        .expect("claim A");
    assert!(
        !claim_a_now,
        "Worker A must be rejected now that Worker B owns the lease"
    );

    // 7. Verify persisted state in database
    let exec = worker_b
        .get_execution_async(tx_id)
        .await
        .expect("get exec")
        .expect("exists");
    assert_eq!(exec.lease_owner.as_deref(), Some("worker_proc_b"));

    db.drop().await;
}

#[tokio::test]
async fn completed_or_failed_workflows_cannot_be_claimed() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("create pool");
    let store = WorkflowStore::from_pool(pool);

    let tx_id = "tx_terminal_1";
    store
        .get_or_create_async(tx_id, "quick-job", json!({}))
        .await
        .expect("create");

    store
        .update_state_async(tx_id, TransactionState::Invoking)
        .await
        .expect("invoking");
    store
        .update_state_async(tx_id, TransactionState::Done)
        .await
        .expect("done");

    let claimed = store
        .try_claim_async(tx_id, "worker_1", 30)
        .await
        .expect("claim");
    assert!(
        !claimed,
        "Terminal state (Done) workflows must never be claimed"
    );

    db.drop().await;
}
