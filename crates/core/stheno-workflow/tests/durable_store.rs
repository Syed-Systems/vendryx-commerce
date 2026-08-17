//! P13-02: WorkflowStore is backed by Postgres, and workflow state outlives the store instance that wrote it.

use serde_json::json;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_workflow::{StepState, TransactionState, WorkflowStore};

#[tokio::test]
async fn workflow_execution_persists_across_store_instances() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("create pool");

    let store1 = WorkflowStore::from_pool(pool.clone());
    let initial_payload = json!({ "order_id": "ord_123", "amount": "99.99" });

    let created = store1
        .get_or_create_async("tx_durable_1", "complete-cart", initial_payload.clone())
        .await
        .expect("create execution");
    assert_eq!(created.transaction_id, "tx_durable_1");
    assert_eq!(created.state, TransactionState::NotStarted);

    store1
        .update_state_async("tx_durable_1", TransactionState::Invoking)
        .await
        .expect("transition to invoking");

    store1
        .set_step_state_async("tx_durable_1", "step_0_inventory", StepState::Running)
        .await
        .expect("step running");

    store1
        .set_step_state_async("tx_durable_1", "step_0_inventory", StepState::Completed)
        .await
        .expect("step completed");

    // Drop first store to prove data is not in memory
    drop(store1);

    // Second store reading the same database
    let store2 = WorkflowStore::from_pool(pool);
    let loaded = store2
        .get_execution_async("tx_durable_1")
        .await
        .expect("query execution")
        .expect("execution must exist in database");

    assert_eq!(loaded.transaction_id, "tx_durable_1");
    assert_eq!(loaded.workflow_name, "complete-cart");
    assert_eq!(loaded.state, TransactionState::Invoking);
    assert_eq!(
        loaded.step_states.get("step_0_inventory"),
        Some(&StepState::Completed)
    );
    assert_eq!(loaded.context_payload, initial_payload);

    db.drop().await;
}

#[tokio::test]
async fn workflow_leases_persist_and_enforce_mutual_exclusion() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("create pool");

    let store = WorkflowStore::from_pool(pool.clone());
    store
        .get_or_create_async("tx_lease_1", "sync-catalog", json!({}))
        .await
        .expect("create execution");
    store
        .update_state_async("tx_lease_1", TransactionState::Invoking)
        .await
        .expect("update state");

    // Worker 1 acquires lease for 60s
    let claimed = store
        .try_claim_async("tx_lease_1", "worker_1", 60)
        .await
        .expect("claim lease");
    assert!(claimed, "worker_1 should acquire the lease");

    // Worker 2 attempts to claim active lease -> rejected
    let claimed_2 = store
        .try_claim_async("tx_lease_1", "worker_2", 60)
        .await
        .expect("claim lease");
    assert!(!claimed_2, "worker_2 must not acquire active lease");

    // Worker 1 can renew its own lease
    let renewed = store
        .try_claim_async("tx_lease_1", "worker_1", 60)
        .await
        .expect("renew lease");
    assert!(renewed, "worker_1 should be allowed to renew its lease");

    let loaded = store
        .get_execution_async("tx_lease_1")
        .await
        .expect("load execution")
        .expect("execution exists");
    assert_eq!(loaded.lease_owner.as_deref(), Some("worker_1"));
    assert!(loaded.lease_expires_at.is_some());

    db.drop().await;
}

#[tokio::test]
async fn workflow_list_executions_returns_persisted_rows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("create pool");

    let store = WorkflowStore::from_pool(pool);
    store
        .get_or_create_async("tx_list_1", "flow_a", json!({ "n": 1 }))
        .await
        .expect("create 1");
    store
        .get_or_create_async("tx_list_2", "flow_b", json!({ "n": 2 }))
        .await
        .expect("create 2");

    let list = store
        .list_executions_async()
        .await
        .expect("list executions");
    assert!(list.len() >= 2);
    assert!(list.iter().any(|e| e.transaction_id == "tx_list_1"));
    assert!(list.iter().any(|e| e.transaction_id == "tx_list_2"));

    db.drop().await;
}

#[tokio::test]
async fn workflow_retention_policy_redacts_context_in_database() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("create pool");

    let store = WorkflowStore::from_pool(pool.clone());
    store
        .get_or_create_async(
            "tx_retention_1",
            "payment-flow",
            json!({ "customer_email": "alice@example.com" }),
        )
        .await
        .expect("create");

    store
        .apply_retention_policy_async("tx_retention_1")
        .await
        .expect("apply retention");

    drop(store);

    let store2 = WorkflowStore::from_pool(pool);
    let loaded = store2
        .get_execution_async("tx_retention_1")
        .await
        .expect("query")
        .expect("exists");

    assert_eq!(
        loaded.context_payload,
        json!({ "[REDACTED]": "retention_policy_applied" })
    );

    db.drop().await;
}
