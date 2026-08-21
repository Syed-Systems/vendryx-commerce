//! P30-10: dunning is a reversible state machine.

use chrono::{Duration, TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{DunningInput, PlatformService};

async fn service() -> (TestDb, PlatformService) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    (db, PlatformService::new(pool))
}

#[tokio::test]
async fn unpaid_subscription_walks_active_to_past_due_to_grace_to_suspended() {
    let (db, service) = service().await;
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 10, 0, 0).unwrap();

    let first = service
        .advance_dunning(DunningInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            now,
        })
        .await
        .expect("first rung");
    assert_eq!(first.status, "past_due");
    assert_eq!(first.dunning_attempts, 1);
    assert_eq!(first.next_attempt_at, Some(now + Duration::days(1)));

    let second = service
        .advance_dunning(DunningInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            now: first.next_attempt_at.unwrap(),
        })
        .await
        .expect("second rung");
    assert_eq!(second.status, "grace");
    assert_eq!(second.dunning_attempts, 2);
    assert_eq!(
        second.next_attempt_at,
        Some(first.next_attempt_at.unwrap() + Duration::days(3))
    );

    let third = service
        .advance_dunning(DunningInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            now: second.next_attempt_at.unwrap(),
        })
        .await
        .expect("third rung");
    assert_eq!(third.status, "suspended");
    assert_eq!(third.dunning_attempts, 3);
    assert_eq!(third.next_attempt_at, None);

    db.drop().await;
}

#[tokio::test]
async fn payment_resets_dunning_to_active() {
    let (db, service) = service().await;
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 10, 0, 0).unwrap();

    service
        .advance_dunning(DunningInput {
            tenant_id: TestDb::TEST_TENANT.to_string(),
            now,
        })
        .await
        .expect("past due");

    let reset = service
        .mark_invoice_paid(TestDb::TEST_TENANT)
        .await
        .expect("payment resets dunning");

    assert_eq!(reset.status, "active");
    assert_eq!(reset.dunning_attempts, 0);
    assert_eq!(reset.next_attempt_at, None);

    db.drop().await;
}
