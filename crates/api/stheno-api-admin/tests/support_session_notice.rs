//! P34-05: The merchant sees that a support session happened, in their own dashboard.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewOperatorAudit, PlatformService};

#[tokio::test]
async fn merchant_support_session_notice_recorded_and_visible() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let platform = PlatformService::new(pool);

    // Operator starts impersonation session on tenant
    platform
        .record_operator_action(NewOperatorAudit {
            actor_id: "op_support_agent".to_string(),
            actor_name: "Support Agent".to_string(),
            action: "impersonation.start".to_string(),
            entity_type: "tenant".to_string(),
            entity_id: "ten_merchant_notified".to_string(),
            diff: Some(serde_json::json!({
                "reason": "Merchant ticket #4812",
                "session_type": "support"
            })),
        })
        .await
        .expect("audit record written");

    // Fetch tenant-scoped audit entries
    let entries = platform
        .list_operator_audit(Some("ten_merchant_notified"), 10, None)
        .await
        .expect("audit entries fetched");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "impersonation.start");
    assert_eq!(entries[0].entity_id, "ten_merchant_notified");
}
