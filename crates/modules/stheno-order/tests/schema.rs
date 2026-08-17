//! PostgreSQL schema verification tests for stheno-order (TASKS P13-14).

use stheno_db::testing::TestDb;

const EXPECTED_ORDER_TABLES: &[&str] = &[
    "address",
    "order",
    "line_item",
    "order_item",
    "line_item_adjustment",
    "line_item_tax_line",
    "shipping_method",
    "order_shipping_method",
    "shipping_method_adjustment",
    "shipping_method_tax_line",
    "order_summary",
    "order_change",
    "order_change_action",
    "transaction",
    "credit_line",
    "return_reason",
    "return",
    "return_item",
    "claim",
    "claim_item",
    "claim_item_image",
    "exchange",
    "exchange_item",
];

#[tokio::test]
async fn every_order_table_exists() {
    let db = TestDb::migrated().await;

    for table in EXPECTED_ORDER_TABLES {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'order' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "order.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    let db = TestDb::migrated().await;

    for table in EXPECTED_ORDER_TABLES {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'order' AND table_name = '{table}' AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(exists, 1, "order.{table} missing deleted_at column");
    }

    db.drop().await;
}

#[tokio::test]
async fn order_foreign_keys_cascade() {
    let db = TestDb::migrated().await;

    // 1. Insert order and child entities
    db.execute(
        r#"
        INSERT INTO "order"."order" (id, display_id, currency_code, status)
        VALUES ('ord_test_cascade', 9999, 'usd', 'pending');

        INSERT INTO "order"."line_item" (id, title, quantity, unit_price)
        VALUES ('item_test_cascade', 'Test Item', 1, 10.00);

        INSERT INTO "order"."order_item" (id, order_id, item_id, version, quantity)
        VALUES ('oi_test_cascade', 'ord_test_cascade', 'item_test_cascade', 1, 1);

        INSERT INTO "order"."line_item_adjustment" (id, item_id, amount)
        VALUES ('adj_test_cascade', 'item_test_cascade', 2.00);
        "#,
    )
    .await;

    // 2. Delete parent order -> cascades to order_item
    db.execute(r#"DELETE FROM "order"."order" WHERE id = 'ord_test_cascade'"#)
        .await;

    let count = db
        .count(r#"SELECT count(*) FROM "order"."order_item" WHERE id = 'oi_test_cascade'"#)
        .await;

    assert_eq!(
        count, 0,
        "order_item should be cascade deleted when order is deleted"
    );

    db.drop().await;
}
