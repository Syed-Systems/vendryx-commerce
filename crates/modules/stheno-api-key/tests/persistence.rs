//! API keys survive the service instance that wrote them.

use stheno_api_key::{ApiKeyService, CreateApiKeyInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = ApiKeyService::new(pool.clone());
    let created = writer
        .create_api_key(CreateApiKeyInput {
            key_type: "publishable".to_string(),
            title: "Outlive Key".to_string(),
            sales_channel_ids: Some(vec!["sc_outlive".to_string()]),
            permissions: None,
            created_by: Some("user_admin".to_string()),
            metadata: None,
        })
        .await
        .expect("create");
    drop(writer);

    let reader = ApiKeyService::new(pool);
    let fetched = reader.get_api_key(&created.api_key.id).await.expect("get");
    assert_eq!(fetched.id, created.api_key.id);
    assert_eq!(fetched.title, "Outlive Key");
    assert_eq!(fetched.sales_channel_ids, vec!["sc_outlive"]);
    let valid = reader.validate_key(&created.token).await.expect("validate");
    assert!(valid.is_valid);

    db.drop().await;
}
