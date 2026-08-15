//! Test P7-13: Auth brute-force limits and account lockout.

use stheno_auth::{AuthService, AuthenticateInput, CreateIdentityInput};

#[tokio::test]
async fn test_auth_brute_force_lockout_and_unlock() {
    let service = AuthService::new(None);

    let identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "target@stheno.store".to_string(),
            password: Some("CorrectPassword123!".to_string()),
            actor_id: Some("user_target".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    // 1. Attempt 4 consecutive failed authentications (under threshold 5)
    for _ in 0..4 {
        let res = service
            .authenticate(AuthenticateInput {
                provider_id: "emailpass".to_string(),
                entity_id: "target@stheno.store".to_string(),
                password: Some("WrongPassword".to_string()),
            })
            .await;
        assert!(res.is_err());
    }

    // Still not locked
    let current_state = service.get_identity(&identity.id).await.unwrap();
    assert!(!current_state.is_locked);

    // 2. 5th failed attempt triggers account lockout!
    let fifth_res = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "target@stheno.store".to_string(),
            password: Some("WrongPassword".to_string()),
        })
        .await;
    assert!(fifth_res.is_err());

    let locked_state = service.get_identity(&identity.id).await.unwrap();
    assert!(locked_state.is_locked);

    // 3. Even with the CORRECT password, authentication is rejected while locked
    let attempt_while_locked = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "target@stheno.store".to_string(),
            password: Some("CorrectPassword123!".to_string()),
        })
        .await;
    assert!(attempt_while_locked.is_err());
    assert!(attempt_while_locked
        .unwrap_err()
        .to_string()
        .contains("Account is temporarily locked"));
}
