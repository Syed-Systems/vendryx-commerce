//! Payment schema tests (TASKS P13-11).

use stheno_db::testing::TestDb;

#[tokio::test]
async fn every_payment_table_exists() {
    let db = TestDb::migrated().await;

    for table in [
        "payment_collection",
        "payment_provider",
        "payment_session",
        "payment",
        "refund_reason",
        "capture",
        "refund",
        "account_holder",
        "payment_provider_request",
        "payment_webhook_event",
        "payment_reconciliation_run",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'payment' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "payment.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    let db = TestDb::migrated().await;

    for table in [
        "payment_collection",
        "payment_provider",
        "payment_session",
        "payment",
        "refund_reason",
        "capture",
        "refund",
        "account_holder",
        "payment_provider_request",
        "payment_webhook_event",
        "payment_reconciliation_run",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'payment' AND table_name = '{table}' AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(exists, 1, "payment.{table}.deleted_at is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn payment_foreign_keys_cascade() {
    let db = TestDb::migrated().await;

    // Seed payment_collection, payment_session, payment, capture, refund
    TestDb::execute(
        &db,
        "INSERT INTO payment.payment_collection (id, currency_code, amount) \
         VALUES ('pcol_test_del', 'usd', 100.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO payment.payment_session (id, payment_collection_id, provider_id, amount, currency_code) \
         VALUES ('payses_test_del', 'pcol_test_del', 'pp_system_default', 100.00, 'usd')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO payment.payment (id, payment_collection_id, payment_session_id, provider_id, currency_code, amount) \
         VALUES ('pay_test_del', 'pcol_test_del', 'payses_test_del', 'pp_system_default', 'usd', 100.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO payment.capture (id, payment_id, amount) \
         VALUES ('capt_test_del', 'pay_test_del', 100.00)",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO payment.refund (id, payment_id, amount) \
         VALUES ('ref_test_del', 'pay_test_del', 50.00)",
    )
    .await;

    // Delete payment_collection must cascade
    TestDb::execute(
        &db,
        "DELETE FROM payment.payment_collection WHERE id = 'pcol_test_del'",
    )
    .await;

    assert_eq!(
        db.count("SELECT count(*) FROM payment.payment_session WHERE id = 'payses_test_del'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM payment.payment WHERE id = 'pay_test_del'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM payment.capture WHERE id = 'capt_test_del'")
            .await,
        0
    );
    assert_eq!(
        db.count("SELECT count(*) FROM payment.refund WHERE id = 'ref_test_del'")
            .await,
        0
    );

    db.drop().await;
}
