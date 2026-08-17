//! P13-01: the workflow schema exists, with tables for executions, step states, and leases.

use diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;

async fn insert(db: &TestDb, sql: &str) -> Result<usize, String> {
    let mut conn = db.conn().await;
    diesel::sql_query(sql)
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn every_workflow_table_exists() {
    let db = TestDb::migrated().await;

    for table in ["execution", "step_state", "lease"] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.tables \
                 WHERE table_schema = 'workflow' AND table_name = '{table}'"
            ))
            .await;
        assert_eq!(exists, 1, "workflow.{table} is missing");
    }

    db.drop().await;
}

#[tokio::test]
async fn every_table_carries_deleted_at() {
    let db = TestDb::migrated().await;

    for table in ["execution", "step_state", "lease"] {
        let has = db
            .count(&format!(
                "SELECT count(*) FROM information_schema.columns \
                 WHERE table_schema = 'workflow' AND table_name = '{table}' \
                 AND column_name = 'deleted_at'"
            ))
            .await;
        assert_eq!(has, 1, "workflow.{table} has no deleted_at column");
    }

    db.drop().await;
}

#[tokio::test]
async fn duplicate_step_state_is_rejected() {
    let db = TestDb::migrated().await;

    TestDb::execute(
        &db,
        "INSERT INTO workflow.execution (id, workflow_name, state) \
         VALUES ('tx_100', 'apply-discount', 'Invoking')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO workflow.step_state (id, transaction_id, step_key, step_name, step_index, state) \
         VALUES ('st_1', 'tx_100', 'step_0_validate', 'validate', 0, 'Pending')",
    )
    .await;

    let err = insert(
        &db,
        "INSERT INTO workflow.step_state (id, transaction_id, step_key, step_name, step_index, state) \
         VALUES ('st_2', 'tx_100', 'step_0_validate', 'validate', 0, 'Running')",
    )
    .await
    .expect_err("duplicate step key in same transaction must be rejected by unique index");

    assert!(
        err.contains("idx_workflow_step_state_tx_key")
            || err.contains("unique")
            || err.contains("duplicate"),
        "expected unique constraint error, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn execution_deletion_cascades() {
    let db = TestDb::migrated().await;

    TestDb::execute(
        &db,
        "INSERT INTO workflow.execution (id, workflow_name, state) \
         VALUES ('tx_200', 'checkout', 'Invoking')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO workflow.step_state (id, transaction_id, step_key, step_name, step_index, state) \
         VALUES ('st_201', 'tx_200', 'step_0_auth', 'auth', 0, 'Completed')",
    )
    .await;

    TestDb::execute(
        &db,
        "INSERT INTO workflow.lease (id, transaction_id, owner_id, lease_expires_at) \
         VALUES ('lease_200', 'tx_200', 'worker_1', now() + interval '1 hour')",
    )
    .await;

    TestDb::execute(&db, "DELETE FROM workflow.execution WHERE id = 'tx_200'").await;

    assert_eq!(
        db.count("SELECT count(*) FROM workflow.step_state WHERE transaction_id = 'tx_200'")
            .await,
        0,
        "step_state rows should cascade on execution deletion"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM workflow.lease WHERE transaction_id = 'tx_200'")
            .await,
        0,
        "lease rows should cascade on execution deletion"
    );

    db.drop().await;
}

#[tokio::test]
async fn missing_execution_foreign_key_rejected() {
    let db = TestDb::migrated().await;

    let err = insert(
        &db,
        "INSERT INTO workflow.step_state (id, transaction_id, step_key, step_name, step_index, state) \
         VALUES ('st_999', 'tx_nonexistent', 'step_0_none', 'none', 0, 'Pending')",
    )
    .await
    .expect_err("foreign key violation expected");

    assert!(
        err.contains("foreign key") || err.contains("violates"),
        "expected foreign key violation, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn workflow_indexes_exist() {
    let db = TestDb::migrated().await;

    for index in [
        "idx_workflow_execution_state",
        "idx_workflow_execution_lease",
        "idx_workflow_step_state_tx_key",
        "idx_workflow_lease_expiry",
    ] {
        let exists = db
            .count(&format!(
                "SELECT count(*) FROM pg_indexes \
                 WHERE schemaname = 'workflow' AND indexname = '{index}'"
            ))
            .await;
        assert_eq!(exists, 1, "index {index} is missing");
    }

    db.drop().await;
}
