use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_store::{CreateStoreInput, StoreService};

#[tokio::test]
async fn test_store_creation_and_currency_retrieval() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = StoreService::new(pool);

    let store = service
        .create_store(CreateStoreInput {
            name: "Flagship Store".to_string(),
            default_sales_channel_id: Some("sc_01default".to_string()),
            default_region_id: Some("reg_01us".to_string()),
            default_location_id: Some("loc_01main".to_string()),
            supported_currencies: Some(vec![
                "usd".to_string(),
                "eur".to_string(),
                "gbp".to_string(),
            ]),
            default_currency_code: Some("usd".to_string()),
            supported_locales: Some(vec!["en".to_string(), "fr".to_string()]),
            default_locale: Some("en".to_string()),
        })
        .await
        .expect("must create store");

    assert_eq!(store.name, "Flagship Store");
    assert_eq!(store.supported_currencies.len(), 3);
    assert_eq!(store.default_currency_code.as_deref(), Some("usd"));
    assert_eq!(store.supported_locales.len(), 2);

    let retrieved = service
        .get_store(&store.id.to_string())
        .await
        .expect("must retrieve store");

    assert_eq!(retrieved.name, "Flagship Store");
    assert_eq!(retrieved.supported_currencies[0], "usd");

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let store_id = {
        let first = StoreService::new(pool.clone());
        let store = first
            .create_store(CreateStoreInput {
                name: "Persistent Store".to_string(),
                default_sales_channel_id: None,
                default_region_id: None,
                default_location_id: None,
                supported_currencies: None,
                default_currency_code: None,
                supported_locales: None,
                default_locale: None,
            })
            .await
            .expect("create");
        store.id.to_string()
    };

    let second = StoreService::new(pool);
    let found = second.get_store(&store_id).await.expect("found");
    assert_eq!(found.name, "Persistent Store");

    db.drop().await;
}
