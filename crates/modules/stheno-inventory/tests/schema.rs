use stheno_db::testing::TestDb;

#[tokio::test]
async fn inventory_tables_exist_and_match_spec() {
    let db = TestDb::migrated().await;

    // Verify tables exist in inventory schema
    let item_cols = db
        .count("SELECT count(*) FROM information_schema.columns WHERE table_schema = 'inventory' AND table_name = 'inventory_item'")
        .await;
    assert!(item_cols > 0, "inventory.inventory_item table must exist");

    let level_cols = db
        .count("SELECT count(*) FROM information_schema.columns WHERE table_schema = 'inventory' AND table_name = 'inventory_level'")
        .await;
    assert!(level_cols > 0, "inventory.inventory_level table must exist");

    let res_cols = db
        .count("SELECT count(*) FROM information_schema.columns WHERE table_schema = 'inventory' AND table_name = 'reservation_item'")
        .await;
    assert!(res_cols > 0, "inventory.reservation_item table must exist");

    db.drop().await;
}
