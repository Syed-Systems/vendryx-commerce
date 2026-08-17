use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_translation::{
    CreateLocaleInput, SetTranslationInput, SetTranslationOverrideInput, TranslationService,
};

#[tokio::test]
async fn test_locale_and_scoped_translation_overrides() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = TranslationService::new(pool);

    // 1. Create Locale
    let fr = service
        .create_locale(CreateLocaleInput {
            code: "fr-FR".to_string(),
            name: "French (France)".to_string(),
            is_default: Some(false),
            is_enabled: Some(true),
        })
        .await
        .unwrap();

    assert_eq!(fr.code, "fr-FR");

    // 2. Set base translation
    let tr = service
        .set_translation(SetTranslationInput {
            locale_code: "fr-FR".to_string(),
            entity_type: "product".to_string(),
            entity_id: "prod_001".to_string(),
            field_name: "title".to_string(),
            value: "Sweat à capuche noir".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(tr.value, "Sweat à capuche noir");

    // 3. Resolve without override
    let resolved_base = service
        .resolve_scoped_field(
            "fr-FR",
            "product",
            "prod_001",
            "title",
            None,
            None,
            "Black Hoodie",
        )
        .await;

    assert_eq!(resolved_base, "Sweat à capuche noir");

    // 4. Set sales-channel specific override
    let _ov = service
        .set_translation_override(SetTranslationOverrideInput {
            translation_id: tr.id.clone(),
            sales_channel_id: Some("sc_vip_paris".to_string()),
            region_id: None,
            override_value: "Sweat à capuche Édition VIP Paris".to_string(),
        })
        .await
        .unwrap();

    // 5. Resolve with VIP sales channel override
    let resolved_override = service
        .resolve_scoped_field(
            "fr-FR",
            "product",
            "prod_001",
            "title",
            Some("sc_vip_paris"),
            None,
            "Black Hoodie",
        )
        .await;

    assert_eq!(resolved_override, "Sweat à capuche Édition VIP Paris");

    // 6. Fallback when translation not found
    let fallback = service
        .resolve_scoped_field(
            "es-ES",
            "product",
            "prod_001",
            "title",
            None,
            None,
            "Black Hoodie Default",
        )
        .await;

    assert_eq!(fallback, "Black Hoodie Default");

    db.drop().await;
}
