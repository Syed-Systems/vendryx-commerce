use std::sync::Arc;
use stheno_file::{CreatePresignedUploadInput, FileService, UploadFileInput};
use stheno_file_local::LocalFileProvider;
use stheno_flows::{
    BatchTranslateWorkflow, CreateLocaleWorkflow, CreatePresignedUploadWorkflow,
    DeleteFileWorkflow, GetSettingWorkflow, ResolveFieldTranslationWorkflow,
    SaveLayoutPreferenceWorkflow, SaveViewConfigWorkflow, SetPropertyLabelWorkflow,
    SetSettingWorkflow, SetTranslationOverrideWorkflow, SetTranslationWorkflow, UploadFileWorkflow,
};
use stheno_settings::{
    SaveLayoutPreferenceInput, SaveViewConfigInput, SetPropertyLabelInput, SetStoreSettingInput,
    SettingsService,
};
use stheno_translation::{
    CreateLocaleInput, SetTranslationInput, SetTranslationOverrideInput, TranslationService,
};

#[tokio::test]
async fn test_13_platform_workflows_settings_translation_file() {
    // ── File Workflows (3) ──────────────────────────────────────────────────
    let local_file_provider = Arc::new(LocalFileProvider::new("uploads", "/static"));
    let file_service = Arc::new(FileService::new(local_file_provider));

    let upload_wf = UploadFileWorkflow::new(file_service.clone());
    let presigned_wf = CreatePresignedUploadWorkflow::new(file_service.clone());
    let delete_wf = DeleteFileWorkflow::new(file_service.clone());

    // Valid JPEG bytes with header: [0xFF, 0xD8, 0xFF, ...]
    let mut jpeg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0];
    jpeg_bytes.extend(vec![0; 50]);

    let uploaded = upload_wf
        .run(UploadFileInput {
            filename: "catalog_cover.jpg".to_string(),
            bytes: jpeg_bytes,
            content_type: Some("image/jpeg".to_string()),
            metadata: None,
        })
        .await
        .unwrap();
    assert_eq!(uploaded.filename, "catalog_cover.jpg");

    let presigned = presigned_wf
        .run(CreatePresignedUploadInput {
            filename: "banner.png".to_string(),
            content_type: "image/png".to_string(),
            expires_in_seconds: Some(3600),
        })
        .await
        .unwrap();
    assert!(!presigned.upload_url.is_empty());

    delete_wf.run(&uploaded.id).await.unwrap();

    // ── Translation Workflows (5) ───────────────────────────────────────────
    let translation_service = Arc::new(TranslationService::new());
    let create_locale_wf = CreateLocaleWorkflow::new(translation_service.clone());
    let set_trans_wf = SetTranslationWorkflow::new(translation_service.clone());
    let set_override_wf = SetTranslationOverrideWorkflow::new(translation_service.clone());
    let resolve_wf = ResolveFieldTranslationWorkflow::new(translation_service.clone());
    let batch_wf = BatchTranslateWorkflow::new(translation_service.clone());

    let de = create_locale_wf
        .run(CreateLocaleInput {
            code: "de-DE".to_string(),
            name: "German (Germany)".to_string(),
            is_default: Some(false),
            is_enabled: Some(true),
        })
        .await
        .unwrap();
    assert_eq!(de.code, "de-DE");

    let tr = set_trans_wf
        .run(SetTranslationInput {
            locale_code: "de-DE".to_string(),
            entity_type: "product".to_string(),
            entity_id: "prod_coat".to_string(),
            field_name: "title".to_string(),
            value: "Wollmantel".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(tr.value, "Wollmantel");

    let _ov = set_override_wf
        .run(SetTranslationOverrideInput {
            translation_id: tr.id.clone(),
            sales_channel_id: Some("sc_berlin".to_string()),
            region_id: None,
            override_value: "Exklusiver Wollmantel Berlin".to_string(),
        })
        .await
        .unwrap();

    let resolved_ov = resolve_wf
        .run(
            "de-DE",
            "product",
            "prod_coat",
            "title",
            Some("sc_berlin"),
            None,
            "Wool Coat",
        )
        .await;
    assert_eq!(resolved_ov, "Exklusiver Wollmantel Berlin");

    let batch_res = batch_wf
        .run(vec![
            SetTranslationInput {
                locale_code: "de-DE".to_string(),
                entity_type: "category".to_string(),
                entity_id: "cat_apparel".to_string(),
                field_name: "title".to_string(),
                value: "Bekleidung".to_string(),
            },
            SetTranslationInput {
                locale_code: "de-DE".to_string(),
                entity_type: "category".to_string(),
                entity_id: "cat_footwear".to_string(),
                field_name: "title".to_string(),
                value: "Schuhe".to_string(),
            },
        ])
        .await
        .unwrap();
    assert_eq!(batch_res.len(), 2);

    // ── Settings Workflows (5) ──────────────────────────────────────────────
    let settings_service = Arc::new(SettingsService::new());
    let set_setting_wf = SetSettingWorkflow::new(settings_service.clone());
    let get_setting_wf = GetSettingWorkflow::new(settings_service.clone());
    let save_view_wf = SaveViewConfigWorkflow::new(settings_service.clone());
    let set_prop_wf = SetPropertyLabelWorkflow::new(settings_service.clone());
    let save_layout_wf = SaveLayoutPreferenceWorkflow::new(settings_service.clone());

    let _setting = set_setting_wf
        .run(SetStoreSettingInput {
            key: "checkout.allow_guest".to_string(),
            value: serde_json::json!(true),
            category: "checkout".to_string(),
            is_secret: Some(false),
        })
        .await
        .unwrap();

    let fetched_setting = get_setting_wf.run("checkout.allow_guest").await.unwrap();
    assert_eq!(fetched_setting.value, serde_json::json!(true));

    let view = save_view_wf
        .run(SaveViewConfigInput {
            user_id: None,
            scope: "admin_inventory".to_string(),
            columns: vec![
                "sku".to_string(),
                "stocked".to_string(),
                "available".to_string(),
            ],
            filters: None,
            sort_by: Some("stocked".to_string()),
            sort_order: Some("desc".to_string()),
            page_size: Some(100),
        })
        .await
        .unwrap();
    assert_eq!(view.scope, "admin_inventory");

    let prop = set_prop_wf
        .run(SetPropertyLabelInput {
            entity_type: "product_variant".to_string(),
            property_key: "upc".to_string(),
            label: "Universal Product Code".to_string(),
            help_text: Some("12-digit barcode".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(prop.label, "Universal Product Code");

    let layout = save_layout_wf
        .run(SaveLayoutPreferenceInput {
            user_id: "usr_admin_1".to_string(),
            layout_key: "sidebar_order".to_string(),
            config: serde_json::json!(["orders", "products", "customers", "settings"]),
        })
        .await
        .unwrap();
    assert_eq!(layout.layout_key, "sidebar_order");
}
