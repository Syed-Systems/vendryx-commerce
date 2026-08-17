//! Test P7-04: GitHub OAuth Provider.

use serde_json::json;
use std::sync::Arc;
use stheno_auth::{AuthProviderService, AuthService};
use stheno_auth_github::GitHubAuthProvider;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_github_oauth_provider_flow() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let auth_service = Arc::new(AuthService::new(pool, None));
    let provider =
        GitHubAuthProvider::new(auth_service, "github_client_id_999", "github_secret_888");

    assert_eq!(provider.provider_id(), "github");

    let github_payload = json!({
        "id": 12345678,
        "login": "octocat",
        "name": "The Octocat",
        "email": "octocat@github.com",
        "avatar_url": "https://github.com/images/error/octocat_happy.gif",
        "actor_id": "user_octo",
        "actor_type": "user"
    });

    // 1. Authenticate / register
    let id1 = provider.authenticate(&github_payload).await.unwrap();
    assert_eq!(id1.provider_id, "github");
    assert_eq!(id1.entity_id, "12345678");

    // 2. Second authenticate retrieves same identity
    let id2 = provider.authenticate(&github_payload).await.unwrap();
    assert_eq!(id2.id, id1.id);
}
