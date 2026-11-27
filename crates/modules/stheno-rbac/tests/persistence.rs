//! Roles survive the service instance that wrote them.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_rbac::{CreateRoleInput, RbacService};

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = RbacService::new(pool.clone());
    writer.seed_default_roles().await.expect("seed");
    writer
        .create_role(CreateRoleInput {
            name: "outlive_lead".to_string(),
            description: Some("Persists".to_string()),
            parent_roles: Some(vec!["member".to_string()]),
            is_system_role: Some(false),
            metadata: None,
        })
        .await
        .expect("create role");
    drop(writer);

    let reader = RbacService::new(pool);
    let role = reader.get_role("outlive_lead").await.expect("get role");
    assert_eq!(role.name, "outlive_lead");
    assert_eq!(role.description.as_deref(), Some("Persists"));
    assert_eq!(role.parent_roles, vec!["member"]);

    db.drop().await;
}
