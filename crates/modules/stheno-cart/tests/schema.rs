//! P13-06: the cart schema exists, with the constraints the cart engine relies on.

use stheno_db::testing::TestDb;

#[tokio::test]
async fn every_cart_table_exists() {
    let db = TestDb::migrated().await;

    for table in [
        "cart",
        "line_item",
        "line_item_adjustment",
        "line_item_tax_line",
        "shipping_method",
        "shipping_method_adjustment",
        "shipping_method_tax_line",
        "credit_line",
        "address",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'cart' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "cart.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    let db = TestDb::migrated().await;

    for table in [
        "cart",
        "line_item",
        "line_item_adjustment",
        "line_item_tax_line",
        "shipping_method",
        "shipping_method_adjustment",
        "shipping_method_tax_line",
        "credit_line",
        "address",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'cart' AND table_name = '{table}' AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(exists, 1, "cart.{table}.deleted_at is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn foreign_keys_cascade_deletes_properly() {
    let db = TestDb::migrated().await;

    // Seed a cart, line item, adjustment, tax line, shipping method, and credit line
    TestDb::execute(
        &db,
        "INSERT INTO cart.cart (id, currency_code, cart_secret) VALUES ('cart_1', 'usd', 'sec_1')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.line_item (id, cart_id, title, quantity, unit_price) \
         VALUES ('li_1', 'cart_1', 'Item 1', 1, 10.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.line_item_adjustment (id, item_id, amount) \
         VALUES ('liadj_1', 'li_1', 2.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.line_item_tax_line (id, item_id, rate, code) \
         VALUES ('litxl_1', 'li_1', 0.20, 'VAT')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.shipping_method (id, cart_id, name, amount) \
         VALUES ('sm_1', 'cart_1', 'Standard', 5.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.shipping_method_adjustment (id, shipping_method_id, amount) \
         VALUES ('smadj_1', 'sm_1', 1.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.shipping_method_tax_line (id, shipping_method_id, rate, code) \
         VALUES ('smtxl_1', 'sm_1', 0.20, 'VAT')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO cart.credit_line (id, cart_id, amount, reference, reference_id) \
         VALUES ('cl_1', 'cart_1', 5.00, 'gift_card', 'gc_123')",
    )
    .await;

    // Deleting cart must cascade
    TestDb::execute(&db, "DELETE FROM cart.cart WHERE id = 'cart_1'").await;

    assert_eq!(
        db.count("SELECT count(*) FROM cart.line_item WHERE cart_id = 'cart_1'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM cart.line_item_adjustment WHERE id = 'liadj_1'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM cart.shipping_method WHERE cart_id = 'cart_1'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM cart.credit_line WHERE cart_id = 'cart_1'")
            .await,
        0
    );

    db.drop().await;
}
