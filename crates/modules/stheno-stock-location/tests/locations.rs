use stheno_stock_location::{CreateStockLocationInput, StockLocationService};

#[tokio::test]
async fn test_stock_location_creation_and_retrieval() {
    let service = StockLocationService::new();

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
}
