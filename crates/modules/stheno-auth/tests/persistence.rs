//! Auth identities survive the service instance that wrote them.

use stheno_auth::{AuthService, CreateIdentityInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = AuthService::new(pool.clone(), None);
    let written = writer
        .create_identity(CreateIdentityInput {
            provider_id: "emailpass".to_string(),
            entity_id: "outlive@example.com".to_string(),
            password: Some("SuperSecretP@ss123!".to_string()),
            actor_id: Some("usr_01outlive".to_string()),
            actor_type: Some("user".to_string()),
            provider_metadata: None,
            app_metadata: None,
            user_metadata: None,
        })
        .await
        .expect("create identity");
    drop(writer);

    let reader = AuthService::new(pool, None);
    let fetched = reader
        .get_identity(&written.id)
        .await
        .expect("get identity");
    assert_eq!(fetched.id, written.id);
    assert_eq!(fetched.provider_id, "emailpass");
    assert_eq!(fetched.entity_id, "outlive@example.com");
    assert_eq!(fetched.actor_id.as_deref(), Some("usr_01outlive"));
    assert_eq!(fetched.actor_type.as_deref(), Some("user"));

    db.drop().await;
}
