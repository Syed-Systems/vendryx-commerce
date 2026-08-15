use serde_json::json;
use stheno_workflow::{TransactionState, WorkflowStore};

#[test]
fn test_workflow_execution_schema_and_store() {
    let store = WorkflowStore::new();
    let exec = store.get_or_create(
        "tx_100",
        "apply-discount-code",
        json!({ "cart_id": "cart_1" }),
    );

    assert_eq!(exec.transaction_id, "tx_100");
    assert_eq!(exec.workflow_name, "apply-discount-code");
    assert_eq!(exec.state, TransactionState::NotStarted);
}
