//! P14-03: Token version invalidation persisted; a revoked token stays revoked after restart.

use stheno_auth::{AuthService, CreateIdentityInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_token_revocation_durability_across_restarts() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let service1 = AuthService::new(pool.clone(), None);

    // 1. Create identity and issue tokens
    let identity = service1
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "durability_user@example.com".to_string(),
            password: Some("SuperSecretP@ss123!".to_string()),
            actor_id: Some("usr_01durability".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .expect("create identity");

    let tokens = service1
        .issue_tokens_for_identity(&identity)
        .await
        .expect("issue tokens");

    // 2. Verify token is valid initially
    let claims = service1
        .verify_jwt(&tokens.access_token)
        .await
        .expect("verify jwt before revocation");
    assert_eq!(claims.sub, identity.id);

    // 3. Invalidate tokens
    service1
        .invalidate_tokens(&identity.id)
        .await
        .expect("invalidate tokens");

    // 4. Simulate complete service restart by creating a fresh AuthService instance
    let service2 = AuthService::new(pool, None);

    // 5. Verify the token is REJECTED by the new instance because token_version was persisted to Postgres
    let err = service2
        .verify_jwt(&tokens.access_token)
        .await
        .expect_err("token must be rejected after restart");

    assert!(
        err.to_string()
            .contains("Token version has been invalidated")
            || err.to_string().contains("Unauthorized"),
        "Revoked token must be rejected with Unauthorized error: {err}"
    );

    db.drop().await;
}
