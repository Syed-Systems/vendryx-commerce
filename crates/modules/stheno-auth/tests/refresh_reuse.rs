//! Test P7-07: Refresh token rotation with automated reuse detection.

use stheno_auth::{AuthService, AuthenticateInput, CreateIdentityInput};

#[tokio::test]
async fn test_refresh_token_rotation_and_reuse_revocation() {
    let service = AuthService::new(None);

    let _identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "bob@stheno.store".to_string(),
            password: Some("BobSecretPass123!".to_string()),
            actor_id: Some("user_bob_789".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    // 1. Initial login -> Generation 0 tokens
    let (_, tokens_g0) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "bob@stheno.store".to_string(),
            password: Some("BobSecretPass123!".to_string()),
        })
        .await
        .unwrap();

    let refresh_g0 = tokens_g0.refresh_token.clone();

    // 2. Legitimate rotation: use refresh_g0 -> receive Generation 1 tokens
    let tokens_g1 = service.refresh_tokens(&refresh_g0).await.unwrap();
    assert_ne!(tokens_g1.refresh_token, refresh_g0);
    assert_ne!(tokens_g1.access_token, tokens_g0.access_token);

    // 3. Legitimate rotation: use refresh_g1 -> receive Generation 2 tokens
    let tokens_g2 = service
        .refresh_tokens(&tokens_g1.refresh_token)
        .await
        .unwrap();
    assert_ne!(tokens_g2.refresh_token, tokens_g1.refresh_token);

    // 4. REUSE ATTACK: Attacker / replay tries to use the already consumed refresh_g0!
    let reuse_attempt = service.refresh_tokens(&refresh_g0).await;
    assert!(reuse_attempt.is_err());
    let err_msg = reuse_attempt.unwrap_err().to_string();
    assert!(err_msg.contains("Refresh token reuse detected"));

    // 5. Subsequent legitimate use of refresh_g2 also fails because the ENTIRE family was revoked!
    let subsequent_attempt = service.refresh_tokens(&tokens_g2.refresh_token).await;
    assert!(subsequent_attempt.is_err());
}
