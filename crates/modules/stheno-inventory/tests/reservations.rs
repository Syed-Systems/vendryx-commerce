use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_inventory::{
    CreateInventoryItemInput, CreateInventoryLevelInput, CreateReservationInput, InventoryService,
};

#[tokio::test]
async fn test_inventory_reservation_release_and_fulfill() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = InventoryService::new(pool);

    let item = service
        .create_item(CreateInventoryItemInput {
            sku: Some("SKU-SNEAKER-1".to_string()),
            requires_shipping: Some(true),
        })
        .await
        .unwrap();

    let item_id = item.id.to_string();
    let location_id = "loc_primary";

    service
        .create_level(CreateInventoryLevelInput {
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            stocked_quantity: 10,
        })
        .await
        .unwrap();

    // 1. Reserve 3 units
    let res = service
        .reserve(CreateReservationInput {
            line_item_id: Some("item_123".to_string()),
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            quantity: 3,
            description: Some("Order #1001 checkout".to_string()),
        })
        .await
        .unwrap();

    let avail = service
        .get_available_quantity(&item_id, location_id)
        .await
        .unwrap();
    assert_eq!(avail, 7);

    let retry = service
        .reserve(CreateReservationInput {
            line_item_id: Some("item_123".to_string()),
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            quantity: 3,
            description: Some("Order #1001 retry".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(retry.id, res.id, "retry returns the existing reservation");
    assert_eq!(
        service
            .get_available_quantity(&item_id, location_id)
            .await
            .unwrap(),
        7,
        "retry must not reserve the quantity twice"
    );
    assert!(service
        .reserve(CreateReservationInput {
            line_item_id: Some("item_123".to_string()),
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            quantity: 4,
            description: None,
        })
        .await
        .is_err());

    // 2. Release reservation (e.g. cart expired or order cancelled)
    service
        .release_reservation(&res.id.to_string())
        .await
        .unwrap();
    service
        .release_reservation(&res.id.to_string())
        .await
        .expect("releasing a reservation twice is idempotent");

    let avail_after_release = service
        .get_available_quantity(&item_id, location_id)
        .await
        .unwrap();
    assert_eq!(avail_after_release, 10);

    // 3. Re-reserve and fulfill
    let res2 = service
        .reserve(CreateReservationInput {
            line_item_id: Some("item_123".to_string()),
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            quantity: 4,
            description: Some("Order #1002 completed".to_string()),
        })
        .await
        .unwrap();

    service
        .fulfill_reservation(&res2.id.to_string())
        .await
        .unwrap();

    let avail_after_fulfill = service
        .get_available_quantity(&item_id, location_id)
        .await
        .unwrap();
    assert_eq!(avail_after_fulfill, 6);

    db.drop().await;
}
