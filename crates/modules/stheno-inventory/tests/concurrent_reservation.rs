use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_inventory::{
    CreateInventoryItemInput, CreateInventoryLevelInput, CreateReservationInput, InventoryService,
};

#[tokio::test]
async fn test_50_concurrent_reservations_against_stock_of_10() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let service = Arc::new(InventoryService::new(pool));

    let item = service
        .create_item(CreateInventoryItemInput {
            sku: Some("SKU-HOT-DROP-1".to_string()),
            requires_shipping: Some(true),
        })
        .await
        .expect("must create item");

    let item_id = item.id.to_string();
    let location_id = "loc_east_1";

    service
        .create_level(CreateInventoryLevelInput {
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            stocked_quantity: 10,
        })
        .await
        .expect("must create level");

    // Spawn 50 concurrent tasks attempting to reserve 1 unit each
    let mut handles = Vec::new();
    for i in 0..50 {
        let svc = Arc::clone(&service);
        let it_id = item_id.clone();
        let loc_id = location_id.to_string();

        handles.push(tokio::spawn(async move {
            svc.reserve(CreateReservationInput {
                line_item_id: Some(format!("item_{i}")),
                inventory_item_id: it_id,
                location_id: loc_id,
                quantity: 1,
                description: Some(format!("Concurrent order #{i}")),
            })
            .await
        }));
    }

    let mut successes = 0;
    let mut failures = 0;

    for h in handles {
        match h.await.expect("task must not panic") {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    assert_eq!(
        successes, 10,
        "Exactly 10 reservations must succeed for 10 units in stock"
    );
    assert_eq!(failures, 40, "Exactly 40 reservations must fail");

    let remaining_avail = service
        .get_available_quantity(&item_id, location_id)
        .await
        .expect("must get available quantity");
    assert_eq!(remaining_avail, 0, "Available quantity must be exactly 0");

    db.drop().await;
}
