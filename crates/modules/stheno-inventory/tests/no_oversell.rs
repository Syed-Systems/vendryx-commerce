use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_inventory::{
    CreateInventoryItemInput, CreateInventoryLevelInput, CreateReservationInput, InventoryService,
};

#[tokio::test]
async fn test_no_oversell_100_concurrent_reservations_vs_1_unit() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 20).expect("pool");
    let service = Arc::new(InventoryService::new(pool));

    let item = service
        .create_item(CreateInventoryItemInput {
            sku: Some("SKU-LIMITED-EDITION".to_string()),
            requires_shipping: Some(true),
        })
        .await
        .unwrap();

    let item_id = item.id.to_string();
    let location_id = "loc_vault";

    // Stock exactly 1 unit!
    service
        .create_level(CreateInventoryLevelInput {
            inventory_item_id: item_id.clone(),
            location_id: location_id.to_string(),
            stocked_quantity: 1,
        })
        .await
        .unwrap();

    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for i in 0..100 {
        let s = service.clone();
        let iid = item_id.clone();
        let sc = success_count.clone();
        let fc = failure_count.clone();

        handles.push(tokio::spawn(async move {
            let res = s
                .reserve(CreateReservationInput {
                    line_item_id: Some(format!("item_{i}")),
                    inventory_item_id: iid,
                    location_id: location_id.to_string(),
                    quantity: 1,
                    description: Some("flash sale request".to_string()),
                })
                .await;

            match res {
                Ok(_) => {
                    sc.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    fc.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    // Critical assertion for Task P2-13 / P12-08: Exactly 1 reservation wins, 99 fail with insufficient inventory
    assert_eq!(
        success_count.load(Ordering::SeqCst),
        1,
        "Exactly 1 concurrent reservation must succeed"
    );
    assert_eq!(
        failure_count.load(Ordering::SeqCst),
        99,
        "99 concurrent reservations must fail"
    );

    let final_avail = service
        .get_available_quantity(&item_id, location_id)
        .await
        .unwrap();
    assert_eq!(final_avail, 0);

    db.drop().await;
}
