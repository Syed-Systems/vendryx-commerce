use stheno_inventory::{CreateInventoryItemInput, CreateInventoryLevelInput, InventoryService};

#[tokio::test]
async fn test_inventory_item_and_level_creation() {
    let service = InventoryService::new();

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
}
