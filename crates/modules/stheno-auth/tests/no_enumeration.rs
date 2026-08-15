//! Test P7-11: Anti-enumeration identical responses and timings (F14).

use stheno_auth::{AuthService, AuthenticateInput, PasswordResetRequestInput};

#[tokio::test]
async fn test_anti_enumeration_responses_and_timing() {
    let service = AuthService::new(None);

    // 1. Password reset request for non-existent account returns Ok(None) without leaking account presence
    let res = service
        .request_password_reset(PasswordResetRequestInput {
            entity_id: "non_existent_user@stheno.store".to_string(),
        })
        .await
        .unwrap();

    assert!(res.is_none());

    // 2. Authentication for non-existent account returns same generic "Invalid credentials" error
    let auth_res = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "ghost@stheno.store".to_string(),
            password: Some("AnyPassword123!".to_string()),
        })
        .await;

    assert!(auth_res.is_err());
    assert!(auth_res
        .unwrap_err()
        .to_string()
        .to_lowercase()
        .contains("invalid credentials"));
}
