//! Store settings survive the service instance that wrote them.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_settings::{SetStoreSettingInput, SettingsService};

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = SettingsService::new(pool.clone());
    writer
        .set_setting(SetStoreSettingInput {
            key: "store.outlive".to_string(),
            value: serde_json::json!("kept"),
            category: "general".to_string(),
            is_secret: Some(false),
        })
        .await
        .expect("set");
    drop(writer);

    let reader = SettingsService::new(pool);
    let fetched = reader.get_setting("store.outlive").await.expect("get");
    assert_eq!(fetched.key, "store.outlive");
    assert_eq!(fetched.value, "kept");
    assert_eq!(fetched.category, "general");

    db.drop().await;
}
