//! Test P7-06: JWT issuance, verification, claims validation.

use stheno_auth::{AuthService, AuthenticateInput, CreateIdentityInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_jwt_issuance_and_claims_verification() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = AuthService::new(pool, None);

    let identity = service
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "alice@stheno.store".to_string(),
            password: Some("Passw0rdAlice!".to_string()),
            actor_id: Some("user_alice_456".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .unwrap();

    let (_, tokens) = service
        .authenticate(AuthenticateInput {
            provider_id: "emailpass".to_string(),
            entity_id: "alice@stheno.store".to_string(),
            password: Some("Passw0rdAlice!".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(tokens.token_type, "Bearer");
    assert!(tokens.expires_in > 0);

    // Verify claims from the JWT
    let claims = service.verify_jwt(&tokens.access_token).await.unwrap();
    assert_eq!(claims.sub, identity.id);
    assert_eq!(claims.actor_id, Some("user_alice_456".to_string()));
    assert_eq!(claims.actor_type, Some("user".to_string()));
    assert_eq!(claims.token_version, 1);
    assert!(claims.exp > claims.iat);

    db.drop().await;
}
