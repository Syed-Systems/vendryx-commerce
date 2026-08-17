//! P12-02: the pricing schema exists, with the constraints the calculation path
//! relies on.
//!
//! Money precision and the uniqueness rules are asserted here rather than in the
//! service, because they are the database's job — a service-side check can be
//! bypassed by the next writer, and by two concurrent ones today.

use stheno_db::testing::TestDb;

async fn insert(db: &TestDb, sql: &str) -> Result<usize, String> {
    use diesel_async::RunQueryDsl;
    let mut conn = db.conn().await;
    diesel::sql_query(sql)
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

async fn seed_set(db: &TestDb) {
    TestDb::execute(db, "INSERT INTO pricing.price_set (id) VALUES ('pset_1')").await;
}

#[tokio::test]
async fn every_pricing_table_exists() {
    let db = TestDb::migrated().await;

    for table in [
        "price_set",
        "price",
        "price_rule",
        "price_list",
        "price_list_rule",
        "price_preference",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'pricing' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "pricing.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    // `PgCrudRepository` reads and writes `deleted_at` by name, so a table
    // without it fails at runtime rather than at compile time.
    let db = TestDb::migrated().await;

    for table in [
        "price_set",
        "price",
        "price_rule",
        "price_list",
        "price_list_rule",
        "price_preference",
    ] {
        let has = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'pricing' AND table_name = '{table}' \
                 AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(has, 1, "pricing.{table} has no deleted_at column");
    }

    db.drop().await;
}

#[tokio::test]
async fn amount_is_numeric_not_float() {
    // ADR-006. A double precision column here would be a silent financial bug.
    let db = TestDb::migrated().await;

    let numeric = db
        .count(
            "SELECT count(*) FROM information_schema.columns \
             WHERE table_schema = 'pricing' AND table_name = 'price' \
             AND column_name = 'amount' AND data_type = 'numeric'",
        )
        .await;
    assert_eq!(numeric, 1, "price.amount must be numeric, never float");

    db.drop().await;
}

#[tokio::test]
async fn money_round_trips_at_every_iso_precision() {
    let db = TestDb::migrated().await;
    seed_set(&db).await;

    // JPY (0 digits), USD (2), KWD (3), and a sub-minor unit price.
    for (id, currency, amount) in [
        ("price_jpy", "jpy", "1234"),
        ("price_usd", "usd", "1234.56"),
        ("price_kwd", "kwd", "1234.567"),
        ("price_sub", "usd", "0.000001"),
    ] {
        TestDb::execute(
            &db,
            &format!(
                "INSERT INTO pricing.price (id, price_set_id, currency_code, amount) \
                 VALUES ('{id}', 'pset_1', '{currency}', {amount})"
            ),
        )
        .await;

        let stored = db
            .query_text(&format!(
                "SELECT amount FROM pricing.price WHERE id = '{id}'"
            ))
            .await;
        let stored: f64 = stored.parse().expect("numeric parses");
        let expected: f64 = amount.parse().expect("literal parses");
        assert!(
            (stored - expected).abs() < 1e-9,
            "{currency} amount {amount} came back as {stored}"
        );
    }

    db.drop().await;
}

#[tokio::test]
async fn an_uppercase_currency_code_is_rejected() {
    let db = TestDb::migrated().await;
    seed_set(&db).await;

    let err = insert(
        &db,
        "INSERT INTO pricing.price (id, price_set_id, currency_code, amount) \
         VALUES ('price_1', 'pset_1', 'USD', 100)",
    )
    .await
    .expect_err("an uppercase currency code must be rejected");
    assert!(
        err.contains("price_currency_is_lowercase"),
        "expected the CHECK constraint, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn an_inverted_quantity_tier_is_rejected() {
    let db = TestDb::migrated().await;
    seed_set(&db).await;

    let err = insert(
        &db,
        "INSERT INTO pricing.price (id, price_set_id, currency_code, amount, min_quantity, max_quantity) \
         VALUES ('price_1', 'pset_1', 'usd', 100, 10, 5)",
    )
    .await
    .expect_err("a tier whose minimum exceeds its maximum can never match");
    assert!(
        err.contains("price_quantity_tier_is_ordered"),
        "expected the CHECK constraint, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn an_inverted_price_list_window_is_rejected() {
    let db = TestDb::migrated().await;

    let err = insert(
        &db,
        "INSERT INTO pricing.price_list (id, title, starts_at, ends_at) \
         VALUES ('plist_1', 'Bad Window', '2026-02-01T00:00:00Z', '2026-01-01T00:00:00Z')",
    )
    .await
    .expect_err("a window that ends before it starts would silently never apply");
    assert!(
        err.contains("price_list_window_is_ordered"),
        "expected the CHECK constraint, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn two_rules_on_the_same_attribute_are_rejected() {
    // Specificity ranking counts matching rules; two rules on one attribute
    // would make the winner depend on row order.
    let db = TestDb::migrated().await;
    seed_set(&db).await;
    TestDb::execute(
        &db,
        "INSERT INTO pricing.price (id, price_set_id, currency_code, amount) \
         VALUES ('price_1', 'pset_1', 'usd', 100)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO pricing.price_rule (id, price_id, attribute, value) \
         VALUES ('prule_1', 'price_1', 'region_id', 'reg_eu')",
    )
    .await;

    let err = insert(
        &db,
        "INSERT INTO pricing.price_rule (id, price_id, attribute, value) \
         VALUES ('prule_2', 'price_1', 'region_id', 'reg_us')",
    )
    .await
    .expect_err("one value per attribute per price");
    assert!(
        err.contains("idx_price_rule_price_attribute"),
        "expected the unique index, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn duplicate_price_preferences_are_rejected() {
    let db = TestDb::migrated().await;

    TestDb::execute(
        &db,
        "INSERT INTO pricing.price_preference (id, attribute, value, is_tax_inclusive) \
         VALUES ('pricepref_1', 'region_id', 'reg_eu', true)",
    )
    .await;

    let err = insert(
        &db,
        "INSERT INTO pricing.price_preference (id, attribute, value, is_tax_inclusive) \
         VALUES ('pricepref_2', 'region_id', 'reg_eu', false)",
    )
    .await
    .expect_err("tax inclusivity must not depend on which row is read first");
    assert!(
        err.contains("idx_price_preference_attribute_value"),
        "expected the unique index, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn deleting_a_price_set_cascades_to_its_prices_and_rules() {
    let db = TestDb::migrated().await;
    seed_set(&db).await;
    TestDb::execute(
        &db,
        "INSERT INTO pricing.price (id, price_set_id, currency_code, amount) \
         VALUES ('price_1', 'pset_1', 'usd', 100)",
    )
    .await;
    TestDb::execute(
        &db,
        "INSERT INTO pricing.price_rule (id, price_id, attribute, value) \
         VALUES ('prule_1', 'price_1', 'region_id', 'reg_eu')",
    )
    .await;

    TestDb::execute(&db, "DELETE FROM pricing.price_set WHERE id = 'pset_1'").await;

    assert_eq!(db.count("SELECT count(*) FROM pricing.price").await, 0);
    assert_eq!(
        db.count("SELECT count(*) FROM pricing.price_rule").await,
        0,
        "rules were orphaned when their price was removed"
    );

    db.drop().await;
}

#[tokio::test]
async fn a_price_cannot_reference_a_missing_price_set() {
    let db = TestDb::migrated().await;

    let err = insert(
        &db,
        "INSERT INTO pricing.price (id, price_set_id, currency_code, amount) \
         VALUES ('price_1', 'pset_missing', 'usd', 100)",
    )
    .await
    .expect_err("a price must belong to a price set that exists");
    assert!(
        err.contains("foreign key") || err.contains("violates"),
        "got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_calculation_path_columns_are_indexed() {
    // P12-15 requires an index for every filterable column; these are the ones
    // the price calculation hot path selects on.
    let db = TestDb::migrated().await;

    for index in [
        "idx_price_price_set_id",
        "idx_price_price_list_id",
        "idx_price_set_currency",
        "idx_price_rule_price_id",
        "idx_price_list_rule_list_id",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM pg_indexes \
                 WHERE schemaname = 'pricing' AND indexname = '{index}'"
            ))
            .await;
        assert_eq!(exists, 1, "index {index} is missing");
    }

    db.drop().await;
}
