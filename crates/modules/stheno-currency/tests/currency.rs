//! P11-18: `currency` on Postgres.

use stheno_currency::CurrencyService;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

async fn service() -> (TestDb, CurrencyService) {
    let db = TestDb::migrated().await;
    let svc = CurrencyService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

#[tokio::test]
async fn the_iso_table_is_seeded_by_the_migration() {
    // Reference data ships with the schema, so a fresh deployment can price an
    // order before anyone has configured anything.
    let (db, service) = service().await;
    let all = service.list_currencies().await.expect("list");
    assert_eq!(all.len(), 20, "the ISO seed did not land");
    db.drop().await;
}

#[tokio::test]
async fn a_currency_is_retrievable_by_code() {
    let (db, service) = service().await;
    let usd = service.get_currency("usd").await.expect("usd");
    assert_eq!(usd.code, "usd");
    assert_eq!(usd.decimal_digits, 2);
    db.drop().await;
}

#[tokio::test]
async fn lookup_is_case_insensitive() {
    let (db, service) = service().await;
    let upper = service.get_currency("USD").await.expect("USD");
    assert_eq!(upper.code, "usd");
    db.drop().await;
}

#[tokio::test]
async fn non_two_digit_precisions_are_preserved() {
    // The reason precision is a lookup and not a constant: rounding JPY or KWD
    // to cents is a financial error (ADR-006).
    let (db, service) = service().await;
    assert_eq!(service.decimal_digits("jpy").await.expect("jpy"), 0);
    assert_eq!(service.decimal_digits("krw").await.expect("krw"), 0);
    assert_eq!(service.decimal_digits("kwd").await.expect("kwd"), 3);
    assert_eq!(service.decimal_digits("bhd").await.expect("bhd"), 3);
    assert_eq!(service.decimal_digits("omr").await.expect("omr"), 3);
    assert_eq!(service.decimal_digits("usd").await.expect("usd"), 2);
    db.drop().await;
}

#[tokio::test]
async fn an_unknown_currency_is_an_error_not_a_default() {
    let (db, service) = service().await;
    assert!(service.get_currency("xyz").await.is_err());
    assert!(
        service.decimal_digits("xyz").await.is_err(),
        "an unknown code must not silently round to 2 digits"
    );
    db.drop().await;
}

#[tokio::test]
async fn the_table_rejects_an_uppercase_code() {
    // Enforced by a CHECK constraint, not by service-side normalisation, so a
    // future writer cannot bypass it.
    let db = TestDb::migrated().await;
    let mut conn = db.conn().await;
    let result = {
        use diesel_async::RunQueryDsl;
        diesel::sql_query(
            "INSERT INTO currency.currency (code, symbol, symbol_native, name, decimal_digits) \
             VALUES ('USD2', '$', '$', 'Bad', 2)",
        )
        .execute(&mut conn)
        .await
    };
    let err = result.expect_err("an uppercase code must be rejected");
    assert!(
        err.to_string().contains("currency_code_is_lowercase"),
        "expected the CHECK constraint to reject it, got: {err}"
    );
    db.drop().await;
}

#[tokio::test]
async fn the_table_rejects_an_out_of_range_precision() {
    let db = TestDb::migrated().await;
    let mut conn = db.conn().await;
    let result = {
        use diesel_async::RunQueryDsl;
        diesel::sql_query(
            "INSERT INTO currency.currency (code, symbol, symbol_native, name, decimal_digits) \
             VALUES ('zzz', 'Z', 'Z', 'Bad', 9)",
        )
        .execute(&mut conn)
        .await
    };
    let err = result.expect_err("9 decimal digits is not an ISO precision");
    assert!(
        err.to_string().contains("currency_digits_in_iso_range"),
        "expected the CHECK constraint to reject it, got: {err}"
    );
    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let first = CurrencyService::new(pool.clone());
    let count_a = first.list_currencies().await.expect("list").len();

    let second = CurrencyService::new(pool);
    let count_b = second.list_currencies().await.expect("list").len();

    assert_eq!(count_a, count_b);
    assert_eq!(count_a, 20);

    db.drop().await;
}
