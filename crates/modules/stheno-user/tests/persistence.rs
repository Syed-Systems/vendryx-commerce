//! Users survive the service instance that wrote them.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_user::{CreateUserInput, UserService};

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = UserService::new(pool.clone());
    let created = writer
        .create_user(CreateUserInput {
            email: "outlive@stheno.store".to_string(),
            first_name: Some("Out".to_string()),
            last_name: Some("Live".to_string()),
            avatar_url: None,
            role: Some("admin".to_string()),
            metadata: None,
        })
        .await
        .expect("create");
    drop(writer);

    let reader = UserService::new(pool);
    let fetched = reader.get_user(&created.id).await.expect("get");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.email, "outlive@stheno.store");
    assert_eq!(fetched.role, "admin");

    db.drop().await;
}
