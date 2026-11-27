//! Notification templates survive the service instance that wrote them.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_notification::{CreateTemplateInput, NotificationService};

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = NotificationService::new(pool.clone());
    let written = writer
        .create_template(CreateTemplateInput {
            name: "outlive_template".to_string(),
            channel: "email".to_string(),
            subject_template: Some("Hello {{name}}".to_string()),
            body_template: "Body for {{name}}".to_string(),
        })
        .await
        .expect("create template");
    drop(writer);

    let reader = NotificationService::new(pool);
    let fetched = reader
        .get_template_by_name("outlive_template")
        .await
        .expect("get template");
    assert_eq!(fetched.id, written.id);
    assert_eq!(fetched.channel, "email");
    assert_eq!(fetched.subject_template.as_deref(), Some("Hello {{name}}"));
    assert_eq!(fetched.body_template, "Body for {{name}}");

    db.drop().await;
}
