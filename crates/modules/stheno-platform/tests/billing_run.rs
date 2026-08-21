//! P30-15: the scheduled billing run issues due invoices and is safe to repeat.

use chrono::{TimeZone, Utc};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{BillingPlanPrice, BillingRunInput, NewTenant, PlatformService};
use stheno_types::currency::CurrencyCode;
use stheno_types::Money;

async fn service() -> (TestDb, PlatformService) {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    (db, PlatformService::new(pool))
}

fn pro_price() -> BillingPlanPrice {
    BillingPlanPrice {
        tier: "pro".to_string(),
        amount: Money::from_minor_units(150_000, CurrencyCode::Bdt),
        interval_months: 1,
    }
}

#[tokio::test]
async fn billing_run_issues_due_invoices_once() {
    let (db, service) = service().await;
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap();

    let first = service
        .run_billing(BillingRunInput {
            now,
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("first billing run");
    let second = service
        .run_billing(BillingRunInput {
            now,
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("repeated billing run");

    assert_eq!(first.invoices_created, 1);
    assert_eq!(second.invoices_created, 0);
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice WHERE tenant_id = 'ten_test'")
            .await,
        1,
        "a repeated scheduled run must not double bill a merchant"
    );

    let sub = service
        .get_subscription(TestDb::TEST_TENANT)
        .await
        .expect("subscription");
    assert_eq!(
        sub.current_period_start,
        Some(Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap())
    );

    db.drop().await;
}

#[tokio::test]
async fn billing_run_skips_unsold_plans() {
    let (db, service) = service().await;
    service
        .create_tenant(NewTenant {
            id: "ten_free_run".to_string(),
            name: "Free Run".to_string(),
            tier: "free".to_string(),
            domain: None,
        })
        .await
        .expect("free tenant");

    let result = service
        .run_billing(BillingRunInput {
            now: Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap(),
            plan_prices: vec![pro_price()],
        })
        .await
        .expect("billing run");

    assert_eq!(result.invoices_created, 1);
    assert!(
        result.unsold_skipped >= 1,
        "the run must report unsold plans it skipped"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM platform.invoice WHERE tenant_id = 'ten_free_run'")
            .await,
        0,
        "an unpriced plan is not sold, not free"
    );

    db.drop().await;
}
