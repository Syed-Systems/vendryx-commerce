use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_stock_location::{CreateStockLocationInput, StockLocationService};

#[tokio::test]
async fn test_stock_location_creation_and_retrieval() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = StockLocationService::new(pool);

    let loc = service
        .create_location(CreateStockLocationInput {
            name: "Frankfurt Fulfillment Center".to_string(),
            address_id: None,
        })
        .await
        .expect("must create stock location");

    assert_eq!(loc.name, "Frankfurt Fulfillment Center");

    let retrieved = service
        .get_location(&loc.id.to_string())
        .await
        .expect("must retrieve location");

    assert_eq!(retrieved.name, "Frankfurt Fulfillment Center");

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let loc_id = {
        let first = StockLocationService::new(pool.clone());
        let loc = first
            .create_location(CreateStockLocationInput {
                name: "Persistent Hub".to_string(),
                address_id: None,
            })
            .await
            .expect("create");
        loc.id.to_string()
    };

    let second = StockLocationService::new(pool);
    let found = second.get_location(&loc_id).await.expect("found");
    assert_eq!(found.name, "Persistent Hub");

    db.drop().await;
}
