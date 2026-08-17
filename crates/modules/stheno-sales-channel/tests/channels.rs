use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_sales_channel::{CreateSalesChannelInput, SalesChannelService};

#[tokio::test]
async fn test_sales_channel_creation_and_retrieval() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = SalesChannelService::new(pool);

    let sc = service
        .create_channel(CreateSalesChannelInput {
            name: "Default Web Store".to_string(),
            description: Some("Primary B2C online shop".to_string()),
            is_disabled: Some(false),
        })
        .await
        .expect("must create sales channel");

    assert_eq!(sc.name, "Default Web Store");
    assert!(!sc.is_disabled);

    let retrieved = service
        .get_channel(&sc.id.to_string())
        .await
        .expect("must retrieve sales channel");

    assert_eq!(retrieved.name, "Default Web Store");

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let sc_id = {
        let first = SalesChannelService::new(pool.clone());
        let sc = first
            .create_channel(CreateSalesChannelInput {
                name: "Persistent Channel".to_string(),
                description: None,
                is_disabled: None,
            })
            .await
            .expect("create");
        sc.id.to_string()
    };

    let second = SalesChannelService::new(pool);
    let found = second.get_channel(&sc_id).await.expect("found");
    assert_eq!(found.name, "Persistent Channel");

    db.drop().await;
}
