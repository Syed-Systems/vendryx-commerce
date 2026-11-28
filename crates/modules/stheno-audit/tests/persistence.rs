//! Audit entries survive the service instance that wrote them.

use serde_json::json;
use stheno_audit::{AuditService, CreateAuditEntry};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = AuditService::new(pool.clone());
    let written = writer
        .append(CreateAuditEntry {
            actor_id: "usr_outlive".to_string(),
            actor_type: "user".to_string(),
            action: "create".to_string(),
            entity_type: "product".to_string(),
            entity_id: "prod_outlive".to_string(),
            diff: Some(json!({ "title": "Kept" })),
        })
        .await
        .expect("append");
    drop(writer);

    let reader = AuditService::new(pool);
    let logs = reader
        .list_by_entity("product", "prod_outlive")
        .await
        .expect("list");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].id, written.id);
    assert_eq!(logs[0].actor_id, "usr_outlive");
    assert_eq!(logs[0].action, "create");
    assert_eq!(logs[0].diff, Some(json!({ "title": "Kept" })));

    db.drop().await;
}
