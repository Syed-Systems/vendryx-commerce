//! Test P7-18: Clean-room RBAC policy engine, inheritance closure, deny-beats-allow.

use stheno_rbac::{CreatePolicyInput, CreateRoleInput, EvaluatePermissionInput, RbacService};

#[tokio::test]
async fn test_rbac_inheritance_closure_and_deny_beats_allow() {
    let service = RbacService::new();

    // 1. Setup custom role hierarchy: custom_lead inherits manager (which inherits member)
    service
        .create_role(CreateRoleInput {
            name: "support_lead".to_string(),
            description: Some("Support team lead".to_string()),
            parent_roles: Some(vec!["member".to_string()]),
            is_system_role: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // 2. Add policies
    // Member can read orders
    service
        .create_policy(CreatePolicyInput {
            role: "member".to_string(),
            resource: "order:*".to_string(),
            action: "read".to_string(),
            effect: "allow".to_string(),
            is_sensitive: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // Support lead can write customer notes
    service
        .create_policy(CreatePolicyInput {
            role: "support_lead".to_string(),
            resource: "customer:notes".to_string(),
            action: "write".to_string(),
            effect: "allow".to_string(),
            is_sensitive: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // Explicit DENY on deleting orders for support_lead
    service
        .create_policy(CreatePolicyInput {
            role: "support_lead".to_string(),
            resource: "order:*".to_string(),
            action: "delete".to_string(),
            effect: "deny".to_string(),
            is_sensitive: Some(false),
            metadata: None,
        })
        .await
        .unwrap();

    // 3. Test inheritance closure: support_lead has "order:*" read permission from "member"
    let read_order_dec = service.evaluate(EvaluatePermissionInput {
        roles: vec!["support_lead".to_string()],
        resource: "order:123".to_string(),
        action: "read".to_string(),
        actor_id: Some("user_support".to_string()),
        mfa_verified: None,
    });
    assert!(read_order_dec.is_allowed);

    // 4. Test direct permission: support_lead can write customer notes
    let write_note_dec = service.evaluate(EvaluatePermissionInput {
        roles: vec!["support_lead".to_string()],
        resource: "customer:notes".to_string(),
        action: "write".to_string(),
        actor_id: Some("user_support".to_string()),
        mfa_verified: None,
    });
    assert!(write_note_dec.is_allowed);

    // 5. Test DENY BEATS ALLOW: even if user also had an admin role allowing delete, explicit deny wins!
    let delete_order_dec = service.evaluate(EvaluatePermissionInput {
        roles: vec!["support_lead".to_string(), "superadmin".to_string()],
        resource: "order:123".to_string(),
        action: "delete".to_string(),
        actor_id: Some("user_support".to_string()),
        mfa_verified: None,
    });
    assert!(!delete_order_dec.is_allowed);
    assert!(delete_order_dec.reason.contains("deny-beats-allow"));
}
