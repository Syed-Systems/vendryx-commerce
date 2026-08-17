use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_inventory::{CreateInventoryItemInput, CreateInventoryLevelInput, InventoryService};

#[tokio::test]
async fn test_inventory_and_stock_workflows() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = InventoryService::new(pool);

    let item = service
        .create_item(CreateInventoryItemInput {
            sku: Some("SKU-FLOW-ITEM-1".to_string()),
            requires_shipping: Some(true),
        })
        .await
        .unwrap();

    let level = service
        .create_level(CreateInventoryLevelInput {
            inventory_item_id: item.id.to_string(),
            location_id: "loc_berlin".to_string(),
            stocked_quantity: 100,
        })
        .await
        .unwrap();

    assert_eq!(level.available_quantity, 100);

    db.drop().await;
}
