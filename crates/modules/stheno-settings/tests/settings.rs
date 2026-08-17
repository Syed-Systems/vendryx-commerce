use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_settings::{
    SaveLayoutPreferenceInput, SaveViewConfigInput, SetPropertyLabelInput, SetStoreSettingInput,
    SettingsService,
};

#[tokio::test]
async fn test_store_settings_view_configs_and_property_labels() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = SettingsService::new(pool);

    // 1. Store settings
    let setting = service
        .set_setting(SetStoreSettingInput {
            key: "store.name".to_string(),
            value: serde_json::json!("Stheno Commerce Flagship"),
            category: "general".to_string(),
            is_secret: Some(false),
        })
        .await
        .unwrap();

    assert_eq!(setting.key, "store.name");

    let fetched = service.get_setting("store.name").await.unwrap();
    assert_eq!(fetched.value, "Stheno Commerce Flagship");

    // 2. View configs
    let view = service
        .save_view_config(SaveViewConfigInput {
            user_id: Some("usr_priya".to_string()),
            scope: "admin_orders".to_string(),
            columns: vec!["id".to_string(), "total".to_string(), "status".to_string()],
            filters: Some(serde_json::json!({ "status": "pending" })),
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
            page_size: Some(50),
        })
        .await
        .unwrap();

    assert_eq!(view.scope, "admin_orders");
    assert_eq!(view.columns.len(), 3);

    let user_view = service
        .get_view_config(Some("usr_priya"), "admin_orders")
        .await
        .unwrap();
    assert_eq!(user_view.page_size, 50);

    // 3. Property label
    let prop = service
        .set_property_label(SetPropertyLabelInput {
            entity_type: "product".to_string(),
            property_key: "hs_code".to_string(),
            label: "Harmonized System Customs Code".to_string(),
            help_text: Some("6-10 digit international tariff code".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(prop.label, "Harmonized System Customs Code");

    // 4. Layout preference
    let layout = service
        .save_layout_preference(SaveLayoutPreferenceInput {
            user_id: "usr_priya".to_string(),
            layout_key: "dashboard_widgets".to_string(),
            config: serde_json::json!({ "sidebar_collapsed": true, "theme": "dark" }),
        })
        .await
        .unwrap();

    assert_eq!(layout.layout_key, "dashboard_widgets");

    db.drop().await;
}
