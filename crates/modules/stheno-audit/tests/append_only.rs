use serde_json::json;
use stheno_audit::{AuditService, CreateAuditEntry};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_audit_log_append_only_writer() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AuditService::new(pool);

    let entry1 = service
        .append(CreateAuditEntry {
            actor_id: "usr_admin_1".to_string(),
            actor_type: "user".to_string(),
            action: "create".to_string(),
            entity_type: "product".to_string(),
            entity_id: "prod_100".to_string(),
            diff: Some(
                json!({ "title": "Sneakers", "price": 1000, "secret_token": "supersecret123" }),
            ),
        })
        .await
        .expect("must append entry");

    assert_eq!(entry1.action, "create");
    assert_eq!(entry1.entity_id, "prod_100");
    // Verify PII redaction
    assert_eq!(entry1.diff.unwrap()["secret_token"], json!("[REDACTED]"));

    let entry2 = service
        .append(CreateAuditEntry {
            actor_id: "usr_admin_1".to_string(),
            actor_type: "user".to_string(),
            action: "update".to_string(),
            entity_type: "product".to_string(),
            entity_id: "prod_100".to_string(),
            diff: Some(json!({ "price": 1200 })),
        })
        .await
        .expect("must append entry");

    assert_eq!(entry2.action, "update");

    let logs = service
        .list_by_entity("product", "prod_100")
        .await
        .expect("must list logs");
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].id, entry1.id);
    assert_eq!(logs[1].id, entry2.id);

    db.drop().await;
}
