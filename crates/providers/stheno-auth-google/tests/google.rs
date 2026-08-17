//! Test P7-04: Google OAuth Provider.

use serde_json::json;
use std::sync::Arc;
use stheno_auth::{AuthProviderService, AuthService};
use stheno_auth_google::GoogleAuthProvider;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_google_oauth_provider_flow() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let auth_service = Arc::new(AuthService::new(pool, None));
    let provider = GoogleAuthProvider::new(
        auth_service,
        "google_client_12345.apps.googleusercontent.com",
        "secret_xyz",
    );

    assert_eq!(provider.provider_id(), "google");

    let google_payload = json!({
        "sub": "google_uid_998877",
        "email": "alex.google@example.com",
        "name": "Alex Google",
        "picture": "https://lh3.googleusercontent.com/a/photo.jpg",
        "email_verified": true,
        "actor_id": "cust_alex",
        "actor_type": "customer"
    });

    // 1. First authenticate -> creates identity
    let id1 = provider.authenticate(&google_payload).await.unwrap();
    assert_eq!(id1.provider_id, "google");
    assert_eq!(id1.entity_id, "google_uid_998877");

    // 2. Second authenticate -> retrieves existing identity
    let id2 = provider.authenticate(&google_payload).await.unwrap();
    assert_eq!(id2.id, id1.id);
}
