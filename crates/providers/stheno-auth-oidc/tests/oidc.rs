//! Test P7-05: Clean-room OIDC Provider (ADR-008).

use serde_json::json;
use std::sync::Arc;
use stheno_auth::{AuthProviderService, AuthService};
use stheno_auth_oidc::{OidcAuthProvider, OidcConfig};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_oidc_auth_provider_discovery_and_token_validation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let auth_service = Arc::new(AuthService::new(pool, None));
    let config = OidcConfig {
        issuer: "https://auth.enterprise.org".to_string(),
        client_id: "stheno_oidc_client_app".to_string(),
        client_secret: Some("secret_client_key_99".to_string()),
        jwks_uri: Some("https://auth.enterprise.org/.well-known/jwks.json".to_string()),
        authorization_endpoint: Some("https://auth.enterprise.org/oauth/authorize".to_string()),
        token_endpoint: Some("https://auth.enterprise.org/oauth/token".to_string()),
        userinfo_endpoint: Some("https://auth.enterprise.org/userinfo".to_string()),
    };

    let provider = OidcAuthProvider::new(auth_service, config);
    assert_eq!(provider.provider_id(), "oidc");

    let now = chrono::Utc::now().timestamp();
    let valid_claims = json!({
        "iss": "https://auth.enterprise.org",
        "sub": "oidc_subject_987654",
        "aud": "stheno_oidc_client_app",
        "exp": now + 3600,
        "iat": now,
        "email": "enterprise_user@enterprise.org",
        "email_verified": true,
        "name": "Enterprise User",
        "picture": "https://enterprise.org/avatars/user.png",
        "nonce": "secure_nonce_12345",
        "actor_id": "user_ent_1",
        "actor_type": "user"
    });

    // 1. Authenticate with valid OIDC token claims
    let credentials = json!({
        "id_token_claims": valid_claims,
        "nonce": "secure_nonce_12345",
        "actor_id": "user_ent_1",
        "actor_type": "user"
    });

    let identity = provider.authenticate(&credentials).await.unwrap();
    assert_eq!(identity.provider_id, "oidc");
    assert_eq!(identity.entity_id, "oidc_subject_987654");

    // 2. Issuer mismatch fails
    let bad_issuer_claims = json!({
        "iss": "https://rogue-auth.evil.org",
        "sub": "oidc_subject_987654",
        "aud": "stheno_oidc_client_app",
        "exp": now + 3600,
        "iat": now
    });
    assert!(provider
        .authenticate(&json!({ "id_token_claims": bad_issuer_claims }))
        .await
        .is_err());

    // 3. Audience mismatch fails
    let bad_aud_claims = json!({
        "iss": "https://auth.enterprise.org",
        "sub": "oidc_subject_987654",
        "aud": "different_app_client",
        "exp": now + 3600,
        "iat": now
    });
    assert!(provider
        .authenticate(&json!({ "id_token_claims": bad_aud_claims }))
        .await
        .is_err());

    // 4. Expired token fails
    let expired_claims = json!({
        "iss": "https://auth.enterprise.org",
        "sub": "oidc_subject_987654",
        "aud": "stheno_oidc_client_app",
        "exp": now - 100,
        "iat": now - 3700
    });
    assert!(provider
        .authenticate(&json!({ "id_token_claims": expired_claims }))
        .await
        .is_err());
}
