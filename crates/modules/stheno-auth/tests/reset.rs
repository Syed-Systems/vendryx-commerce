//! Test P7-10: Single-use, expiring, hashed password reset tokens.

use stheno_auth::{
    AuthService, AuthenticateInput, CreateIdentityInput, PasswordResetConfirmInput,
    PasswordResetRequestInput,
};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_password_reset_token_lifecycle() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AuthService::new(pool, None);

    let _identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "david@stheno.store".to_string(),
            password: Some("InitialPass123!".to_string()),
            actor_id: Some("user_david".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    // 1. Request password reset -> receives plain token
    let token_opt = service
        .request_password_reset(PasswordResetRequestInput {
            entity_id: "david@stheno.store".to_string(),
        })
        .await
        .unwrap();

    assert!(token_opt.is_some());
    let token = token_opt.unwrap();

    // 2. Confirm password reset with the token
    let updated_identity = service
        .confirm_password_reset(PasswordResetConfirmInput {
            token: token.clone(),
            new_password: "ResetSuccessPass456!".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(updated_identity.token_version, 2);

    // 3. Authenticate with new password
    let (auth_id, _) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "david@stheno.store".to_string(),
            password: Some("ResetSuccessPass456!".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(auth_id.id, updated_identity.id);

    // 4. Replay of the used reset token fails (single-use)!
    let replay_attempt = service
        .confirm_password_reset(PasswordResetConfirmInput {
            token,
            new_password: "AnotherNewPass789!".to_string(),
        })
        .await;

    assert!(replay_attempt.is_err());

    db.drop().await;
}
