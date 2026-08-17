use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_inventory::{CreateInventoryItemInput, CreateInventoryLevelInput, InventoryService};

#[tokio::test]
async fn test_inventory_item_and_level_creation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = InventoryService::new(pool);

    let item = service
        .create_item(CreateInventoryItemInput {
            sku: Some("SKU-TSHIRT-BLK".to_string()),
            requires_shipping: Some(true),
        })
        .await
        .expect("must create item");

    let level = service
        .create_level(CreateInventoryLevelInput {
            inventory_item_id: item.id.to_string(),
            location_id: "loc_warehouse_us_east".to_string(),
            stocked_quantity: 50,
        })
        .await
        .expect("must create level");

    assert_eq!(level.stocked_quantity, 50);
    assert_eq!(level.reserved_quantity, 0);
    assert_eq!(level.available_quantity, 50);

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let item_id = {
        let first = InventoryService::new(pool.clone());
        let item = first
            .create_item(CreateInventoryItemInput {
                sku: Some("SKU-SURVIVOR".to_string()),
                requires_shipping: Some(true),
            })
            .await
            .expect("create item");
        item.id.to_string()
    };

    let second = InventoryService::new(pool);
    let items = second.list_items().await.expect("list items");
    assert!(items.iter().any(|i| i.id.to_string() == item_id));

    db.drop().await;
}
