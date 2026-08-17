//! Fulfillment schema tests (TASKS P13-19).

use stheno_db::testing::TestDb;

#[tokio::test]
async fn every_fulfillment_table_exists() {
    let db = TestDb::migrated().await;

    for table in [
        "address",
        "fulfillment_provider",
        "shipping_profile",
        "shipping_option_type",
        "fulfillment_set",
        "service_zone",
        "geo_zone",
        "shipping_option",
        "shipping_option_rule",
        "fulfillment",
        "fulfillment_item",
        "fulfillment_label",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'fulfillment' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "fulfillment.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    let db = TestDb::migrated().await;

    for table in [
        "address",
        "fulfillment_provider",
        "shipping_profile",
        "shipping_option_type",
        "fulfillment_set",
        "service_zone",
        "geo_zone",
        "shipping_option",
        "shipping_option_rule",
        "fulfillment",
        "fulfillment_item",
        "fulfillment_label",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'fulfillment' AND table_name = '{table}' AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(exists, 1, "fulfillment.{table}.deleted_at is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn fulfillment_foreign_keys_cascade() {
    let db = TestDb::migrated().await;

    // Seed provider, set, service zone, shipping profile, shipping option, fulfillment, item, label
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) \
         VALUES ('fp_test_del', 'Manual Provider')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_set (id, name, type) \
         VALUES ('fuset_test_del', 'Default Set', 'shipping')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.service_zone (id, fulfillment_set_id, name) \
         VALUES ('serzo_test_del', 'fuset_test_del', 'US Zone')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.shipping_profile (id, name, type) \
         VALUES ('sp_test_del', 'Default Profile', 'default')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.shipping_option (id, name, price_type, service_zone_id, shipping_profile_id, provider_id) \
         VALUES ('so_test_del', 'Express', 'flat', 'serzo_test_del', 'sp_test_del', 'fp_test_del')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment (id, provider_id, shipping_option_id) \
         VALUES ('ful_test_del', 'fp_test_del', 'so_test_del')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_item (id, fulfillment_id, title, line_item_id, quantity) \
         VALUES ('fulit_test_del', 'ful_test_del', 'Test Product', 'item_123', 1)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_label (id, fulfillment_id, tracking_number) \
         VALUES ('fulla_test_del', 'ful_test_del', 'TRK123456')",
    )
    .await;

    // Verify row counts
    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.fulfillment WHERE id = 'ful_test_del'")
            .await,
        1
    );
    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.fulfillment_item WHERE fulfillment_id = 'ful_test_del'")
            .await,
        1
    );
    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.fulfillment_label WHERE fulfillment_id = 'ful_test_del'")
            .await,
        1
    );

    // Deleting fulfillment should cascade to items and labels
    TestDb::execute(
        &db,
        "DELETE FROM fulfillment.fulfillment WHERE id = 'ful_test_del'",
    )
    .await;

    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.fulfillment_item WHERE fulfillment_id = 'ful_test_del'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.fulfillment_label WHERE fulfillment_id = 'ful_test_del'")
            .await,
        0
    );

    db.drop().await;
}
