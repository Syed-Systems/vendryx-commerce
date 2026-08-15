use serde_json::json;
use stheno_workflow::WorkflowStore;

#[test]
fn test_workflow_retention_and_pii_redaction() {
    let store = WorkflowStore::new();
    let exec = store.get_or_create(
        "tx_retain_1",
        "checkout_flow",
        json!({ "customer_email": "alice@example.com", "card_token": "tok_123" }),
    );
    assert_eq!(exec.context_payload["customer_email"], "alice@example.com");

    store.apply_retention_policy("tx_retain_1");

    let updated = store.get_or_create("tx_retain_1", "checkout_flow", json!({}));
    assert!(updated.context_payload.get("[REDACTED]").is_some());
}
