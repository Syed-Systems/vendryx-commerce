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
        "payment_provider_credential",
        "payment_webhook_event",
        "payment_reconciliation_run",
        "courier_settlement_cycle",
        "courier_settlement_line",
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
        "payment_provider_credential",
        "payment_webhook_event",
        "payment_reconciliation_run",
        "courier_settlement_cycle",
        "courier_settlement_line",
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
async fn payment_provider_credentials_are_tenant_scoped_and_opaque() {
    let db = TestDb::migrated().await;

    for column in [
        "id",
        "tenant_id",
        "provider_id",
        "ciphertext",
        "nonce",
        "credential_version",
        "webhook_slug",
        "created_at",
        "updated_at",
        "deleted_at",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'payment' \
                   AND table_name = 'payment_provider_credential' \
                   AND column_name = '{column}'"
            ))
            .await;
        assert_eq!(
            exists, 1,
            "payment.payment_provider_credential.{column} is missing"
        );
    }

    let has_plaintext = db
        .count(
            "SELECT count(*) FROM information_schema.columns \
             WHERE table_schema = 'payment' \
               AND table_name = 'payment_provider_credential' \
               AND column_name IN ('secret', 'plaintext', 'api_key', 'webhook_secret')",
        )
        .await;
    assert_eq!(
        has_plaintext, 0,
        "credential table must not expose plaintext-shaped columns"
    );

    let tenant_provider_unique = db
        .count(
            "SELECT count(*) \
             FROM pg_indexes \
             WHERE schemaname = 'payment' \
               AND tablename = 'payment_provider_credential' \
               AND indexdef ILIKE '%UNIQUE%' \
               AND indexdef ILIKE '%tenant_id%' \
               AND indexdef ILIKE '%provider_id%'",
        )
        .await;
    assert_eq!(
        tenant_provider_unique, 1,
        "credential table needs a unique tenant/provider index"
    );

    let webhook_slug_unique = db
        .count(
            "SELECT count(*) \
             FROM pg_indexes \
             WHERE schemaname = 'payment' \
               AND tablename = 'payment_provider_credential' \
               AND indexdef ILIKE '%UNIQUE%' \
               AND indexdef ILIKE '%webhook_slug%'",
        )
        .await;
    assert_eq!(
        webhook_slug_unique, 1,
        "webhook_slug must be globally unique"
    );

    let rls_forced = db
        .count(
            "SELECT count(*) \
             FROM pg_class c \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = 'payment' \
               AND c.relname = 'payment_provider_credential' \
               AND c.relrowsecurity \
               AND c.relforcerowsecurity",
        )
        .await;
    assert_eq!(rls_forced, 1, "credential table must have forced RLS");

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
