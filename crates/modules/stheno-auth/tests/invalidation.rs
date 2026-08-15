//! Test P7-08: Token version invalidation on password/MFA/email change (F13).

use stheno_auth::{AuthService, AuthenticateInput, CreateIdentityInput, UpdatePasswordInput};

#[tokio::test]
async fn test_token_version_invalidation_revokes_active_jwts() {
    let service = AuthService::new(None);

    let identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "charlie@stheno.store".to_string(),
            password: Some("OldPassword123!".to_string()),
            actor_id: Some("user_charlie".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    // 1. Authenticate -> JWT with token_version = 1
    let (_, tokens) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "charlie@stheno.store".to_string(),
            password: Some("OldPassword123!".to_string()),
        })
        .await
        .unwrap();

    // Verify token works before password change
    let claims = service.verify_jwt(&tokens.access_token).await.unwrap();
    assert_eq!(claims.token_version, 1);

    // 2. User changes their password (or updates MFA/email)
    let updated_identity = service
        .update_password(UpdatePasswordInput {
            identity_id: identity.id.clone(),
            old_password: "OldPassword123!".to_string(),
            new_password: "NewSuperPassword456!".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(updated_identity.token_version, 2);

    // 3. The old JWT (version 1) is immediately rejected!
    let verification_result = service.verify_jwt(&tokens.access_token).await;
    assert!(verification_result.is_err());
    assert!(verification_result
        .unwrap_err()
        .to_string()
        .contains("Token version has been invalidated"));

    // 4. Authenticate with new password -> JWT with token_version = 2
    let (_, new_tokens) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "charlie@stheno.store".to_string(),
            password: Some("NewSuperPassword456!".to_string()),
        })
        .await
        .unwrap();

    let new_claims = service.verify_jwt(&new_tokens.access_token).await.unwrap();
    assert_eq!(new_claims.token_version, 2);
}
