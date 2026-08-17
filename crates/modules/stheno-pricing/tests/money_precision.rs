//! ADR-006 regression guard: money must not travel through f64.
//!
//! `PgCrudRepository` moves entities as JSON. serde_json parses numeric literals
//! into f64 unless `arbitrary_precision` is enabled, which silently rounded
//! 12345678901234.567891 to 12345678901234.566 -- on every money column of every
//! entity. This test fails if that feature is ever dropped.
use stheno_db::testing::TestDb;

#[tokio::test]
async fn money_is_not_routed_through_f64() {
    let db = TestDb::migrated().await;
    TestDb::execute(&db, "INSERT INTO pricing.price_set (id) VALUES ('pset_1')").await;
    // 20 significant digits: f64 has ~15-17, so a value beyond that proves the path.
    TestDb::execute(
        &db,
        "INSERT INTO pricing.price (id, price_set_id, currency_code, amount) \
         VALUES ('price_1', 'pset_1', 'usd', 12345678901234.567891)",
    )
    .await;

    let raw = db
        .query_text("SELECT row_to_json(t)::text FROM pricing.price t WHERE t.id = 'price_1'")
        .await;
    let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    let d: rust_decimal::Decimal = serde_json::from_value(v["amount"].clone()).expect("decimal");
    assert_eq!(
        d.to_string(),
        "12345678901234.567891",
        "precision lost through the JSON bridge"
    );
    db.drop().await;
}
