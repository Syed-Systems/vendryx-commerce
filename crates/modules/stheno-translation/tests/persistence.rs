//! Locales and translations survive the service instance that wrote them.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_translation::{CreateLocaleInput, SetTranslationInput, TranslationService};

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = TranslationService::new(pool.clone());
    writer
        .create_locale(CreateLocaleInput {
            code: "fr-FR".to_string(),
            name: "French (France)".to_string(),
            is_default: Some(false),
            is_enabled: Some(true),
        })
        .await
        .expect("locale");
    writer
        .set_translation(SetTranslationInput {
            locale_code: "fr-FR".to_string(),
            entity_type: "product".to_string(),
            entity_id: "prod_outlive".to_string(),
            field_name: "title".to_string(),
            value: "Sweat noir".to_string(),
        })
        .await
        .expect("translation");
    drop(writer);

    let reader = TranslationService::new(pool);
    let locale = reader.get_locale("fr-FR").await.expect("get locale");
    assert_eq!(locale.name, "French (France)");
    let resolved = reader
        .resolve_scoped_field(
            "fr-FR",
            "product",
            "prod_outlive",
            "title",
            None,
            None,
            "Black",
        )
        .await;
    assert_eq!(resolved, "Sweat noir");

    db.drop().await;
}
