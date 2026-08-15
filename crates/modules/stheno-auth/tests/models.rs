//! Test P7-01: Auth entities and models.

use stheno_auth::{AuthService, CreateIdentityInput};

#[tokio::test]
async fn test_auth_models_and_identity_lifecycle() {
    let service = AuthService::new(None);

    let identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "admin@stheno.store".to_string(),
            password: Some("SecurePassword123!".to_string()),
            actor_id: Some("user_01HXYZ".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(identity.provider_id, "emailpass");
    assert_eq!(identity.entity_id, "admin@stheno.store");
    assert_eq!(identity.actor_id, Some("user_01HXYZ".to_string()));
    assert_eq!(identity.actor_type, Some("user".to_string()));
    assert_eq!(identity.token_version, 1);
    assert!(!identity.is_locked);
}
