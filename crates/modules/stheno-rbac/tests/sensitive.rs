//! Test P7-19, P14-06: Sensitive permissions requiring MFA and audit logging on Postgres.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_rbac::{CreatePolicyInput, EvaluatePermissionInput, RbacService};

#[tokio::test]
async fn test_sensitive_permission_mfa_requirement_and_audit_log() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = RbacService::new(pool);
    service.seed_default_roles().await.unwrap();

    // 1. Create a sensitive policy (e.g. Export all customer PII)
    service
        .create_policy(CreatePolicyInput {
            role: "admin".to_string(),
            resource: "customer:export_pii".to_string(),
            action: "execute".to_string(),
            effect: "allow".to_string(),
            is_sensitive: Some(true),
            metadata: None,
        })
        .await
        .unwrap();

    // 2. Evaluation without MFA verification -> DENIED
    let no_mfa_decision = service
        .evaluate(EvaluatePermissionInput {
            roles: vec!["admin".to_string()],
            resource: "customer:export_pii".to_string(),
            action: "execute".to_string(),
            actor_id: Some("user_admin_1".to_string()),
            mfa_verified: Some(false),
        })
        .await
        .unwrap();

    assert!(!no_mfa_decision.is_allowed);
    assert!(no_mfa_decision.requires_mfa);
    assert!(no_mfa_decision.reason.contains("requires MFA"));

    // 3. Evaluation WITH MFA verification -> ALLOWED
    let with_mfa_decision = service
        .evaluate(EvaluatePermissionInput {
            roles: vec!["admin".to_string()],
            resource: "customer:export_pii".to_string(),
            action: "execute".to_string(),
            actor_id: Some("user_admin_1".to_string()),
            mfa_verified: Some(true),
        })
        .await
        .unwrap();

    assert!(with_mfa_decision.is_allowed);

    // 4. Verify audit ledger records both attempts
    let audits = service
        .get_audit_records(Some("user_admin_1"))
        .await
        .unwrap();
    assert_eq!(audits.len(), 2);
    assert_eq!(audits[0].decision, "denied");
    assert!(!audits[0].mfa_verified);
    assert_eq!(audits[1].decision, "allowed");
    assert!(audits[1].mfa_verified);

    db.drop().await;
}
